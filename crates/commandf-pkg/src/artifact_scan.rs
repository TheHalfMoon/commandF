use std::io::{self, Cursor, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::ArtifactError;

pub(crate) const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_ARCHIVE_ENTRIES: usize = 50_000;
pub(crate) const MAX_RESOURCE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScannedResource {
    pub filename: String,
    pub bytes: Vec<u8>,
}

struct BoundedReader<R> {
    inner: R,
    read: u64,
    max_bytes: u64,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, max_bytes: u64) -> Self {
        Self {
            inner,
            read: 0,
            max_bytes,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.read == self.max_bytes {
            let mut probe = [0_u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "archive exceeds decompressed limit",
                )),
            };
        }
        let remaining = (self.max_bytes - self.read) as usize;
        let allowed = buf.len().min(remaining);
        let count = self.inner.read(&mut buf[..allowed])?;
        self.read += count as u64;
        Ok(count)
    }
}

pub(crate) fn scan_package_resources(
    archive_bytes: &[u8],
) -> Result<Vec<ScannedResource>, ArtifactError> {
    scan_package_resources_with_limit(archive_bytes, MAX_ARCHIVE_BYTES)
}

pub(crate) fn scan_package_resources_with_limit(
    archive_bytes: &[u8],
    max_archive_bytes: u64,
) -> Result<Vec<ScannedResource>, ArtifactError> {
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(BoundedReader::new(decoder, max_archive_bytes));
    let mut resources = Vec::new();
    let mut entry_count = 0_usize;

    for entry in archive.entries()? {
        entry_count += 1;
        if entry_count > MAX_ARCHIVE_ENTRIES {
            return Err(ArtifactError::TooManyEntries);
        }
        let entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path()?.into_owned();
        let normalized = path.strip_prefix(Path::new("./")).unwrap_or(path.as_ref());
        if normalized.parent() != Some(Path::new("package")) {
            continue;
        }
        let Some(filename_os) = normalized.file_name() else {
            continue;
        };
        let Some(filename) = filename_os.to_str().map(str::to_owned) else {
            continue;
        };
        if filename == "package.json" || filename == ".index.json" || !filename.ends_with(".json") {
            continue;
        }
        if entry.size() > MAX_RESOURCE_BYTES {
            return Err(ArtifactError::ResourceTooLarge(filename));
        }
        let mut bytes = Vec::new();
        entry.take(MAX_RESOURCE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_RESOURCE_BYTES {
            return Err(ArtifactError::ResourceTooLarge(filename));
        }
        resources.push(ScannedResource { filename, bytes });
    }

    resources.sort_by(|left, right| left.filename.cmp(&right.filename));
    Ok(resources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_reader_accepts_exact_limit() {
        let mut reader = BoundedReader::new(Cursor::new(b"abcd"), 4);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"abcd");
    }

    #[test]
    fn bounded_reader_rejects_byte_beyond_limit() {
        let mut reader = BoundedReader::new(Cursor::new(b"abcde"), 4);
        let mut output = Vec::new();
        let error = reader.read_to_end(&mut output).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("decompressed limit"));
    }
}
