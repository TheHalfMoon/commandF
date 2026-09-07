use std::collections::BTreeSet;
use std::ffi::{c_char, c_int, CString};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{CandidateInput, InputGuardError};

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(1);

pub(crate) struct CandidateSnapshot {
    root: PathBuf,
}

impl CandidateSnapshot {
    pub(crate) fn capture(
        candidate_root: &Path,
        inputs: &[CandidateInput],
        max_single_file_bytes: u64,
        max_aggregate_bytes: u64,
        max_files: u64,
    ) -> Result<Self, InputGuardError> {
        let files = u64::try_from(inputs.len()).map_err(|_| {
            InputGuardError::Violation("candidate input file count overflows u64".to_owned())
        })?;
        if files == 0 {
            return violation("candidate authority input set must not be empty");
        }
        if files > max_files {
            return violation("candidate input file count exceeds policy");
        }

        let root_fd = open_candidate_root(candidate_root)?;
        let snapshot_root = create_private_snapshot_root()?;
        let snapshot = Self {
            root: snapshot_root,
        };
        let mut seen = BTreeSet::<PathBuf>::new();
        let mut aggregate_bytes = 0_u64;

        for input in inputs {
            validate_relative_path(&input.relative_path)?;
            if !seen.insert(input.relative_path.clone()) {
                return violation(format!(
                    "candidate input path {} is duplicated",
                    input.relative_path.display()
                ));
            }

            let mut file = open_candidate_file(&root_fd, &input.relative_path)?;
            let metadata = file
                .metadata()
                .map_err(|error| io_error(&input.relative_path, error))?;
            if !metadata.is_file() {
                return violation(format!(
                    "candidate path {} is not a regular non-symlink file",
                    input.relative_path.display()
                ));
            }
            if metadata.len() > max_single_file_bytes {
                return violation(format!(
                    "candidate path {} exceeds single-file byte policy before parse",
                    input.relative_path.display()
                ));
            }

            let mut bytes = Vec::new();
            file.by_ref()
                .take(max_single_file_bytes.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(|error| io_error(&input.relative_path, error))?;
            let byte_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
            if byte_len > max_single_file_bytes {
                return violation(format!(
                    "candidate path {} exceeds single-file byte policy before parse",
                    input.relative_path.display()
                ));
            }
            aggregate_bytes = aggregate_bytes.checked_add(byte_len).ok_or_else(|| {
                InputGuardError::Violation("candidate input aggregate bytes overflow".to_owned())
            })?;
            if aggregate_bytes > max_aggregate_bytes {
                return violation("candidate input aggregate bytes exceed policy before parse");
            }

            let destination = snapshot.root.join(&input.relative_path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
            }
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|error| io_error(&destination, error))?;
            std::io::Write::write_all(&mut output, &bytes)
                .map_err(|error| io_error(&destination, error))?;
        }

        Ok(snapshot)
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for CandidateSnapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn create_private_snapshot_root() -> Result<PathBuf, InputGuardError> {
    use std::os::unix::fs::PermissionsExt;

    for _ in 0..128 {
        let nonce = NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "commandf-af02-input-snapshot-{}-{nonce}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
                    .map_err(|error| io_error(&path, error))?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_error(&path, error)),
        }
    }
    violation("unable to allocate private candidate input snapshot")
}

fn validate_relative_path(path: &Path) -> Result<(), InputGuardError> {
    let text = path
        .to_str()
        .ok_or_else(|| InputGuardError::Violation("candidate path must be UTF-8".to_owned()))?;
    if text.is_empty()
        || text.starts_with('/')
        || text.contains('\\')
        || text.contains('\0')
        || text
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return violation(format!("candidate path {text:?} is not portable and relative"));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
const O_RDONLY: c_int = 0;
#[cfg(target_os = "linux")]
const O_DIRECTORY: c_int = 0o200000;
#[cfg(target_os = "linux")]
const O_NOFOLLOW: c_int = 0o400000;
#[cfg(target_os = "linux")]
const O_CLOEXEC: c_int = 0o2000000;
#[cfg(target_os = "linux")]
const AT_FDCWD: c_int = -100;

#[cfg(target_os = "linux")]
extern "C" {
    fn openat(dirfd: c_int, pathname: *const c_char, flags: c_int) -> c_int;
}

#[cfg(target_os = "linux")]
fn open_candidate_root(candidate_root: &Path) -> Result<std::os::fd::OwnedFd, InputGuardError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;

    let expected = fs::symlink_metadata(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;
    if expected.file_type().is_symlink() || !expected.is_dir() {
        return violation("candidate root must be a non-symlink directory");
    }
    let canonical = fs::canonicalize(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;

    let mut current = openat_owned(AT_FDCWD, Path::new("/"), directory_flags())?;
    for component in canonical.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(part) => {
                current = openat_owned(current.as_raw_fd(), Path::new(part), directory_flags())?;
            }
            _ => return violation("canonical candidate root contains an unsupported component"),
        }
    }

    let opened = File::from(
        current
            .try_clone()
            .map_err(|error| io_error(candidate_root, error))?,
    );
    let actual = opened
        .metadata()
        .map_err(|error| io_error(candidate_root, error))?;
    if expected.dev() != actual.dev() || expected.ino() != actual.ino() {
        return violation("candidate root changed identity during descriptor binding");
    }
    Ok(current)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn open_candidate_root(_candidate_root: &Path) -> Result<std::os::fd::OwnedFd, InputGuardError> {
    violation("secure descriptor-relative no-follow open is unavailable on this Unix platform")
}

#[cfg(target_os = "linux")]
fn open_candidate_file(
    root_fd: &std::os::fd::OwnedFd,
    relative: &Path,
) -> Result<File, InputGuardError> {
    use std::os::fd::AsRawFd;

    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(part) => Ok(PathBuf::from(part)),
            _ => violation("candidate path contains a non-normal component"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let Some((last, parents)) = components.split_last() else {
        return violation("candidate path must not be empty");
    };

    let mut current = root_fd
        .try_clone()
        .map_err(|error| io_error(relative, error))?;
    for component in parents {
        current = openat_owned(current.as_raw_fd(), component, directory_flags())
            .map_err(|error| normalize_open_error(relative, error))?;
    }
    let file_fd = openat_owned(current.as_raw_fd(), last, file_flags())
        .map_err(|error| normalize_open_error(relative, error))?;
    Ok(File::from(file_fd))
}

#[cfg(all(unix, not(target_os = "linux")))]
fn open_candidate_file(
    _root_fd: &std::os::fd::OwnedFd,
    _relative: &Path,
) -> Result<File, InputGuardError> {
    violation("secure descriptor-relative no-follow open is unavailable on this Unix platform")
}

#[cfg(target_os = "linux")]
fn directory_flags() -> c_int {
    O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC
}

#[cfg(target_os = "linux")]
fn file_flags() -> c_int {
    O_RDONLY | O_NOFOLLOW | O_CLOEXEC
}

#[cfg(target_os = "linux")]
fn openat_owned(
    dirfd: c_int,
    path: &Path,
    flags: c_int,
) -> Result<std::os::fd::OwnedFd, InputGuardError> {
    use std::os::fd::FromRawFd;
    use std::os::unix::ffi::OsStrExt;

    let path_bytes = path.as_os_str().as_bytes();
    let c_path = CString::new(path_bytes).map_err(|_| {
        InputGuardError::Violation(format!("candidate path {} contains NUL", path.display()))
    })?;
    let raw = unsafe { openat(dirfd, c_path.as_ptr(), flags) };
    if raw < 0 {
        return Err(InputGuardError::Io(format!(
            "{}: {}",
            path.display(),
            std::io::Error::last_os_error()
        )));
    }
    Ok(unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) })
}

fn normalize_open_error(path: &Path, error: InputGuardError) -> InputGuardError {
    match error {
        InputGuardError::Io(message) => InputGuardError::Violation(format!(
            "candidate path {} failed descriptor-relative no-follow open: {message}",
            path.display()
        )),
        other => other,
    }
}

fn io_error(path: &Path, error: std::io::Error) -> InputGuardError {
    InputGuardError::Io(format!("{}: {error}", path.display()))
}

fn violation<T>(message: impl Into<String>) -> Result<T, InputGuardError> {
    Err(InputGuardError::Violation(message.into()))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::io::Read;
    use std::os::unix::fs::symlink;

    use super::*;

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new(label: &str) -> Self {
            let nonce = NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "commandf-af02-secure-open-{label}-{}-{nonce}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn descriptor_relative_open_rejects_symlink_components_and_final_symlinks() {
        let root = TempRoot::new("root");
        let outside = TempRoot::new("outside");
        fs::write(outside.path.join("value.yml"), b"outside").unwrap();
        fs::create_dir(root.path.join("dir")).unwrap();
        symlink(&outside.path, root.path.join("link-dir")).unwrap();
        symlink(outside.path.join("value.yml"), root.path.join("link-file.yml")).unwrap();

        let root_fd = open_candidate_root(&root.path).unwrap();
        assert!(open_candidate_file(&root_fd, Path::new("link-dir/value.yml")).is_err());
        assert!(open_candidate_file(&root_fd, Path::new("link-file.yml")).is_err());
    }

    #[test]
    fn opened_descriptor_is_not_rebound_by_symlink_to_hard_link_replacement() {
        let root = TempRoot::new("root-race");
        let outside = TempRoot::new("outside-race");
        let target = root.path.join("value.yml");
        let saved = root.path.join("saved.yml");
        let outside_file = outside.path.join("value.yml");
        fs::write(&target, b"inside").unwrap();
        fs::write(&outside_file, b"outside").unwrap();

        let root_fd = open_candidate_root(&root.path).unwrap();
        let mut opened = open_candidate_file(&root_fd, Path::new("value.yml")).unwrap();
        fs::rename(&target, &saved).unwrap();
        symlink(&outside_file, &target).unwrap();
        fs::remove_file(&target).unwrap();
        fs::hard_link(&outside_file, &target).unwrap();

        let mut bytes = Vec::new();
        opened.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"inside");
    }
}
