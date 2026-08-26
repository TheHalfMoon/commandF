use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::{parse_hl7_oracle_report, Hl7OracleReport, OracleError};

pub const DEFAULT_ORACLE_TIMEOUT_SECS: u64 = 60;
pub const MAX_ORACLE_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ORACLE_STDERR_BYTES: usize = 1024 * 1024;

pub struct Hl7OracleInvocation<'a> {
    pub core_package: &'a Path,
    pub left_package: &'a Path,
    pub right_package: &'a Path,
    pub left_url: &'a str,
    pub left_version: Option<&'a str>,
    pub right_url: &'a str,
    pub right_version: Option<&'a str>,
}

pub struct Hl7OracleStagedArchives {
    _directory: TempDir,
    core_package: PathBuf,
    left_package: PathBuf,
    right_package: PathBuf,
}

impl Hl7OracleStagedArchives {
    pub fn new(core: &[u8], left: &[u8], right: &[u8]) -> Result<Self, OracleError> {
        let directory = tempfile::tempdir().map_err(|source| OracleError::AdapterIo {
            operation: "creating staged oracle directory",
            source,
        })?;
        let core_package = stage_archive(directory.path(), "core.tgz", core)?;
        let left_package = stage_archive(directory.path(), "left.tgz", left)?;
        let right_package = stage_archive(directory.path(), "right.tgz", right)?;
        Ok(Self {
            _directory: directory,
            core_package,
            left_package,
            right_package,
        })
    }

    pub fn core_package(&self) -> &Path {
        &self.core_package
    }

    pub fn left_package(&self) -> &Path {
        &self.left_package
    }

    pub fn right_package(&self) -> &Path {
        &self.right_package
    }
}

fn stage_archive(root: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf, OracleError> {
    let path = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|source| OracleError::AdapterIo {
            operation: "creating staged oracle archive",
            source,
        })?;
    file.write_all(bytes)
        .map_err(|source| OracleError::AdapterIo {
            operation: "writing staged oracle archive",
            source,
        })?;
    file.sync_all().map_err(|source| OracleError::AdapterIo {
        operation: "syncing staged oracle archive",
        source,
    })?;
    drop(file);

    let mut permissions = fs::metadata(&path)
        .map_err(|source| OracleError::AdapterIo {
            operation: "reading staged oracle archive metadata",
            source,
        })?
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).map_err(|source| OracleError::AdapterIo {
        operation: "protecting staged oracle archive",
        source,
    })?;
    Ok(path)
}

pub fn validate_hl7_oracle_adapter(adapter: &Path, java: Option<&Path>) -> Result<(), OracleError> {
    if !adapter.is_file() {
        return Err(OracleError::AdapterPath {
            path: adapter.display().to_string(),
        });
    }
    if adapter_is_jar(adapter) {
        let java = java.ok_or(OracleError::JavaRequiredForJar)?;
        if !java.is_file() {
            return Err(OracleError::JavaPath {
                path: java.display().to_string(),
            });
        }
    }
    Ok(())
}

pub fn run_hl7_oracle_adapter(
    adapter: &Path,
    java: Option<&Path>,
    invocation: &Hl7OracleInvocation<'_>,
    timeout: Duration,
) -> Result<Hl7OracleReport, OracleError> {
    validate_hl7_oracle_adapter(adapter, java)?;

    let mut command = if adapter_is_jar(adapter) {
        let java = java.ok_or(OracleError::JavaRequiredForJar)?;
        let mut command = Command::new(java);
        command.arg("-jar").arg(adapter);
        command
    } else {
        Command::new(adapter)
    };

    command
        .arg("--core-package")
        .arg(invocation.core_package)
        .arg("--left-package")
        .arg(invocation.left_package)
        .arg("--right-package")
        .arg(invocation.right_package)
        .arg("--left-url")
        .arg(invocation.left_url)
        .arg("--right-url")
        .arg(invocation.right_url);

    if let Some(version) = invocation.left_version {
        command.arg("--left-version").arg(version);
    }
    if let Some(version) = invocation.right_version {
        command.arg("--right-version").arg(version);
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_tree(&mut command);

    let mut child = command.spawn().map_err(|source| OracleError::AdapterIo {
        operation: "spawning adapter",
        source,
    })?;

    let stdout = child.stdout.take().ok_or_else(|| OracleError::AdapterIo {
        operation: "opening adapter stdout",
        source: io::Error::other("adapter stdout was not piped"),
    })?;
    let stderr = child.stderr.take().ok_or_else(|| OracleError::AdapterIo {
        operation: "opening adapter stderr",
        source: io::Error::other("adapter stderr was not piped"),
    })?;

    let stdout_thread = thread::spawn(move || read_bounded(stdout, MAX_ORACLE_STDOUT_BYTES));
    let stderr_thread = thread::spawn(move || read_bounded(stderr, MAX_ORACLE_STDERR_BYTES));

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|source| OracleError::AdapterIo {
            operation: "polling adapter",
            source,
        })? {
            break status;
        }
        if started.elapsed() >= timeout {
            terminate_process_tree(&mut child);
            let _ = child.wait();
            let _ = join_capture(stdout_thread, "stdout");
            let _ = join_capture(stderr_thread, "stderr");
            return Err(OracleError::AdapterTimeout {
                millis: timeout.as_millis(),
            });
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = join_capture(stdout_thread, "stdout")?;
    let stderr = join_capture(stderr_thread, "stderr")?;
    validate_capture_limit("stdout", &stdout, MAX_ORACLE_STDOUT_BYTES)?;
    validate_capture_limit("stderr", &stderr, MAX_ORACLE_STDERR_BYTES)?;

    if !status.success() {
        return Err(OracleError::AdapterExit {
            code: status.code(),
            stderr: stderr_summary(&stderr),
        });
    }

    parse_hl7_oracle_report(&stdout)
}

#[cfg(unix)]
fn configure_process_tree(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_tree(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    const SIGKILL: i32 = 9;

    let Ok(pid) = i32::try_from(child.id()) else {
        let _ = child.kill();
        return;
    };
    // SAFETY: this child was spawned with PGID == PID, so the negative PID targets only
    // the dedicated adapter process group and its descendants.
    let result = unsafe { kill(-pid, SIGKILL) };
    if result != 0 {
        let _ = child.kill();
    }
}

#[cfg(windows)]
fn terminate_process_tree(child: &mut Child) {
    use std::ffi::OsString;
    use std::path::PathBuf;

    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| OsString::from(r"C:\Windows"));
    let taskkill = PathBuf::from(root).join("System32").join("taskkill.exe");
    let pid = child.id().to_string();
    let status = Command::new(taskkill)
        .args(["/PID", &pid, "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if !matches!(status, Ok(status) if status.success()) {
        let _ = child.kill();
    }
}

#[cfg(not(any(unix, windows)))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
}

fn adapter_is_jar(adapter: &Path) -> bool {
    adapter
        .extension()
        .map(|extension| extension.to_string_lossy().eq_ignore_ascii_case("jar"))
        .unwrap_or(false)
}

fn read_bounded<R: Read>(reader: R, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take((limit + 1) as u64).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_capture(
    handle: thread::JoinHandle<io::Result<Vec<u8>>>,
    stream: &'static str,
) -> Result<Vec<u8>, OracleError> {
    handle
        .join()
        .map_err(|_| OracleError::AdapterCaptureThread { stream })?
        .map_err(|source| OracleError::AdapterIo {
            operation: if stream == "stdout" {
                "reading adapter stdout"
            } else {
                "reading adapter stderr"
            },
            source,
        })
}

fn validate_capture_limit(
    stream: &'static str,
    bytes: &[u8],
    limit: usize,
) -> Result<(), OracleError> {
    if bytes.len() > limit {
        return Err(OracleError::AdapterOutputLimit {
            stream,
            actual: bytes.len(),
            limit,
        });
    }
    Ok(())
}

fn stderr_summary(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr)
        .trim()
        .chars()
        .take(4096)
        .collect()
}
