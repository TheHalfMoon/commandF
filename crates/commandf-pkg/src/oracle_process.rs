use std::io::{self, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

pub fn run_hl7_oracle_adapter(
    adapter: &Path,
    java: Option<&Path>,
    invocation: &Hl7OracleInvocation<'_>,
    timeout: Duration,
) -> Result<Hl7OracleReport, OracleError> {
    if !adapter.is_file() {
        return Err(OracleError::AdapterPath {
            path: adapter.display().to_string(),
        });
    }

    let is_jar = adapter
        .extension()
        .map(|extension| extension.to_string_lossy().eq_ignore_ascii_case("jar"))
        .unwrap_or(false);

    let mut command = if is_jar {
        let java = java.ok_or(OracleError::JavaRequiredForJar)?;
        if !java.is_file() {
            return Err(OracleError::JavaPath {
                path: java.display().to_string(),
            });
        }
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
            let _ = child.kill();
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
