use std::io::{Cursor, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

use crate::{model::PackageManifest, PackageError};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

pub(crate) fn read_manifest(bytes: &[u8]) -> Result<PackageManifest, PackageError> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(decoder);

    for entry in archive.entries()? {
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
