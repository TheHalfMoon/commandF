use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use commandf_pkg::Lockfile;

pub(crate) const MAX_LOCKFILE_INPUT_BYTES: u64 = Lockfile::MAX_PERSISTED_LOCKFILE_BYTES as u64;

pub(crate) fn read_lockfile(path: &Path) -> Result<Lockfile, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    // Reject a known-oversized file before reading; keep the +1-byte
    // streaming check to catch growth after the metadata observation.
    let oversized = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("lockfile exceeds the maximum supported size of {MAX_LOCKFILE_INPUT_BYTES} bytes"),
        )
    };
    if file.metadata()?.len() > MAX_LOCKFILE_INPUT_BYTES {
        return Err(oversized().into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_LOCKFILE_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_LOCKFILE_INPUT_BYTES {
        return Err(oversized().into());
    }
    Ok(Lockfile::from_bounded_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sparse_oversized_lockfile_has_stable_size_error() {
        let path = std::env::temp_dir().join(format!(
            "commandf-sparse-oversize-lock-{}",
            std::process::id()
        ));
        File::create(&path)
            .expect("create sparse lockfile")
            .set_len(MAX_LOCKFILE_INPUT_BYTES + 1)
            .expect("expand sparse lockfile");

        let error = read_lockfile(&path).expect_err("oversized lockfile");
        assert!(error
            .to_string()
            .contains("lockfile exceeds the maximum supported size"));
        assert!(!error.to_string().contains(&path.display().to_string()));
        assert!(!error.to_string().contains("JSON"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn oversized_lockfile_read_is_rejected_before_parsing() {
        let path =
            std::env::temp_dir().join(format!("commandf-lock-oversize-{}", std::process::id()));
        let mut file = File::create(&path).expect("create lockfile");
        file.write_all(&[b'x'; 64]).expect("write prefix");
        let padding = vec![b'x'; Lockfile::MAX_PERSISTED_LOCKFILE_BYTES];
        file.write_all(&padding).expect("write overflow");
        drop(file);

        let first = read_lockfile(&path).expect_err("oversized lockfile");
        let second = read_lockfile(&path).expect_err("repeated oversized lockfile");
        assert_eq!(first.to_string(), second.to_string());
        assert!(first
            .to_string()
            .contains("lockfile exceeds the maximum supported size"));
        assert!(!first.to_string().contains("JSON"));
        let _ = std::fs::remove_file(path);
    }
}
