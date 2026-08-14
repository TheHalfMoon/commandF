use std::io::Cursor;

use commandf_pkg::{diff_package_archives, PackageCache, StructuralDiffError};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

fn duplicate_filename_archive() -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for body in [
            br#"{"resourceType":"Patient","id":"first"}"#.as_slice(),
            br#"{"resourceType":"Patient","id":"second"}"#.as_slice(),
        ] {
            let mut header = Header::new_gnu();
            header.set_path("package/Patient-example.json").unwrap();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, Cursor::new(body)).unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

#[test]
fn duplicate_package_resource_filename_fails_closed() {
    let archive = duplicate_filename_archive();
    let digest = PackageCache::digest(&archive);

    let error = diff_package_archives(
        "example.pkg",
        "1.0.0",
        &digest,
        &archive,
        "1.0.0",
        &digest,
        &archive,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        StructuralDiffError::DuplicateResourceFilename { .. }
    ));
}
