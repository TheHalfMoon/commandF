use std::io::{self, Cursor, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::{model::PackageManifest, PackageError};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MIN_MANIFEST_SCAN_DECOMPRESSED_BYTES: u64 = 512 * 1024 * 1024;
const MAX_MANIFEST_SCAN_DECOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;
const MANIFEST_SCAN_EXPANSION_RATIO: u64 = 16;
const MAX_ARCHIVE_ENTRIES: usize = 50_000;

struct BoundedReader<R> {
    inner: R,
    read: u64,
    max: u64,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, max: u64) -> Self {
        Self {
            inner,
            read: 0,
            max,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if self.read == self.max {
            let mut probe = [0_u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "package archive exceeds the maximum decompressed size",
                )),
            };
        }

        let remaining = (self.max - self.read) as usize;
        let allowed = remaining.min(buf.len());
        let count = self.inner.read(&mut buf[..allowed])?;
        self.read += count as u64;
        Ok(count)
    }
}

pub(crate) fn read_manifest(bytes: &[u8]) -> Result<PackageManifest, PackageError> {
    read_manifest_with_limits(
        bytes,
        manifest_scan_decompressed_limit(bytes.len()),
        MAX_ARCHIVE_ENTRIES,
    )
}

fn manifest_scan_decompressed_limit(compressed_bytes: usize) -> u64 {
    let compressed_bytes = u64::try_from(compressed_bytes).unwrap_or(u64::MAX);
    compressed_bytes
        .saturating_mul(MANIFEST_SCAN_EXPANSION_RATIO)
        .clamp(
            MIN_MANIFEST_SCAN_DECOMPRESSED_BYTES,
            MAX_MANIFEST_SCAN_DECOMPRESSED_BYTES,
        )
}

fn read_manifest_with_limits(
    bytes: &[u8],
    max_decompressed_bytes: u64,
    max_entries: usize,
) -> Result<PackageManifest, PackageError> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let bounded = BoundedReader::new(decoder, max_decompressed_bytes);
    let mut archive = Archive::new(bounded);
    let mut entry_count = 0_usize;

    for entry in archive.entries()? {
        entry_count += 1;
        if entry_count > max_entries {
            return Err(PackageError::InvalidRequest(format!(
                "package archive exceeds the maximum entry count of {max_entries}"
            )));
        }

        let entry = entry?;
        let path = entry.path()?;
        let normalized = path.strip_prefix(Path::new("./")).unwrap_or(path.as_ref());
        if normalized != Path::new("package/package.json") {
            continue;
        }
        if entry.size() > MAX_MANIFEST_BYTES {
            return Err(PackageError::ManifestTooLarge);
        }
        let mut body = String::new();
        entry
            .take(MAX_MANIFEST_BYTES + 1)
            .read_to_string(&mut body)?;
        if body.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(PackageError::ManifestTooLarge);
        }
        return Ok(serde_json::from_str(&body)?);
    }

    Err(PackageError::MissingManifest)
}

#[cfg(test)]
mod tests {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};

    use super::*;

    fn archive_with_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = Builder::new(&mut encoder);
            for (path, body) in entries {
                let mut header = Header::new_gnu();
                header.set_path(path).unwrap();
                header.set_size(body.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append(&header, Cursor::new(*body)).unwrap();
            }
            builder.finish().unwrap();
        }
        encoder.finish().unwrap()
    }

    #[test]
    fn manifest_scan_budget_preserves_floor_scales_and_caps() {
        assert_eq!(
            manifest_scan_decompressed_limit(1),
            MIN_MANIFEST_SCAN_DECOMPRESSED_BYTES
        );
        assert_eq!(
            manifest_scan_decompressed_limit(40 * 1024 * 1024),
            640 * 1024 * 1024
        );
        assert_eq!(
            manifest_scan_decompressed_limit(78_238_082),
            MAX_MANIFEST_SCAN_DECOMPRESSED_BYTES
        );
        assert_eq!(
            manifest_scan_decompressed_limit(usize::MAX),
            MAX_MANIFEST_SCAN_DECOMPRESSED_BYTES
        );
    }

    #[test]
    fn rejects_excessive_entry_count_before_manifest() {
        let bytes = archive_with_entries(&[("one", b""), ("two", b""), ("three", b"")]);
        let error = read_manifest_with_limits(&bytes, 1024 * 1024, 2).unwrap_err();
        assert!(matches!(error, PackageError::InvalidRequest(_)));
    }

    #[test]
    fn rejects_excessive_decompressed_archive_bytes() {
        let body = vec![b'x'; 4096];
        let bytes = archive_with_entries(&[("large", body.as_slice())]);
        let error = read_manifest_with_limits(&bytes, 1024, 100).unwrap_err();
        assert!(matches!(error, PackageError::Io(_)));
    }
}
