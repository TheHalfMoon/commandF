use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use commandf_pkg::{
    LocalMirrorSource, Lockfile, PackageCache, PackageError, PackageRequest, Resolver,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use proptest::prelude::*;
use serde_json::json;
use tar::{Builder, Header};

#[path = "support/property_runner.rs"]
mod property_runner;

#[path = "../../tests/assurance/af02_models/archive_manifest.rs"]
mod archive_manifest_model;

use archive_manifest_model::{Limits, LogicalEntry, Outcome};

const PROPERTY_ID: &str = "PROP-ARCHIVE-MANIFEST-001";
const SEED_HEX: &str = "9f2ff50e2d1382752f79e92585f4fec473081f43d327cf810cc594cf28689dd8";
const PRODUCT_MAX_ENTRIES: usize = 50_000;
const PRODUCT_MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const PRODUCT_MIN_DECOMPRESSED_BUDGET: u64 = 512 * 1024 * 1024;
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(label: &str) -> PathBuf {
    let counter = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "commandf-af02-t034-{label}-{}-{counter}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create T034 scratch directory");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn tar_footprint(body_len: usize) -> u64 {
    let body_blocks = body_len.div_ceil(512) as u64;
    512 + body_blocks * 512
}

fn entry(path: impl Into<String>, body: Vec<u8>) -> LogicalEntry {
    LogicalEntry {
        path: path.into(),
        decompressed_bytes: tar_footprint(body.len()),
        body,
    }
}

fn package_manifest(marker: u16) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "name": "af02.archive",
        "version": "1.0.0",
        "dependencies": {},
        "marker": marker
    }))
    .expect("serialize generated package manifest")
}

fn archive(entries: &[LogicalEntry]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for logical in entries {
            let mut header = Header::new_gnu();
            header
                .set_path(&logical.path)
                .expect("set generated tar path");
            header.set_size(logical.body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append(&header, Cursor::new(&logical.body))
                .expect("append generated tar entry");
        }
        builder.finish().expect("finish generated tar archive");
    }
    encoder.finish().expect("finish generated gzip archive")
}

fn product_resolve(entries: &[LogicalEntry]) -> Result<Lockfile, PackageError> {
    let root = scratch("archive-product");
    let mirror = root.join("mirror");
    let package_dir = mirror.join("af02.archive").join("1.0.0");
    fs::create_dir_all(&package_dir).expect("create generated local mirror package directory");
    fs::write(package_dir.join("package.tgz"), archive(entries))
        .expect("write generated package archive");

    let source = LocalMirrorSource::new(&mirror);
    let cache = PackageCache::new(root.join("cache"));
    let result = Resolver::new(&source, &cache)
        .resolve(vec![
            PackageRequest::parse("af02.archive@1.0.0").expect("parse frozen package request")
        ]);
    cleanup(&root);
    result
}

fn production_limits() -> Limits {
    Limits {
        max_entries: PRODUCT_MAX_ENTRIES,
        max_manifest_bytes: PRODUCT_MAX_MANIFEST_BYTES,
        max_decompressed_bytes: PRODUCT_MIN_DECOMPRESSED_BUDGET,
    }
}

#[test]
fn af02_archive_manifest_valid_inventory_matches_independent_model() {
    let strategy = (any::<bool>(), 0_usize..=8, 0_usize..=8, any::<u16>());
    let mut runner = property_runner::runner(SEED_HEX);
    runner
        .run(&strategy, |(prefix, before_count, after_count, marker)| {
            let mut entries = (0..before_count)
                .map(|index| {
                    entry(
                        format!("package/noise-before-{index}.bin"),
                        vec![index as u8; index],
                    )
                })
                .collect::<Vec<_>>();
            let manifest_index = entries.len();
            let manifest_path = if prefix {
                "./package/package.json"
            } else {
                "package/package.json"
            };
            entries.push(entry(manifest_path, package_manifest(marker)));
            entries.extend((0..after_count).map(|index| {
                entry(
                    format!("package/noise-after-{index}.bin"),
                    vec![marker as u8; index],
                )
            }));

            let expected = archive_manifest_model::evaluate(&entries, production_limits());
            let Outcome::AcceptManifest {
                entry_index,
                manifest,
                ..
            } = expected
            else {
                return Err(TestCaseError::fail(
                    "bounded valid archive model did not accept",
                ));
            };
            prop_assert_eq!(entry_index, manifest_index);
            prop_assert_eq!(manifest.name, "af02.archive");
            prop_assert_eq!(manifest.version, "1.0.0");
            prop_assert!(manifest.dependencies.is_empty());

            let lock = product_resolve(&entries)
                .map_err(|error| TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(lock.packages.len(), 1);
            prop_assert_eq!(lock.packages[0].name.as_str(), "af02.archive");
            prop_assert_eq!(lock.packages[0].version.as_str(), "1.0.0");
            prop_assert!(lock.packages[0].dependencies.is_empty());
            Ok(())
        })
        .expect("frozen archive-manifest property must match the independent model");
}

#[test]
fn af02_archive_manifest_missing_invalid_prefix_and_duplicate_cases_are_closed() {
    let missing = vec![entry("package/readme.txt", b"noise".to_vec())];
    assert_eq!(
        archive_manifest_model::evaluate(&missing, production_limits()),
        Outcome::MissingManifest
    );
    assert!(matches!(
        product_resolve(&missing),
        Err(PackageError::MissingManifest)
    ));

    let invalid = vec![entry("package/package.json", b"{".to_vec())];
    assert_eq!(
        archive_manifest_model::evaluate(&invalid, production_limits()),
        Outcome::InvalidManifestJson
    );
    assert!(matches!(
        product_resolve(&invalid),
        Err(PackageError::Json(_))
    ));

    let prefixed = vec![entry("./package/package.json", package_manifest(7))];
    assert!(matches!(
        archive_manifest_model::evaluate(&prefixed, production_limits()),
        Outcome::AcceptManifest { entry_index: 0, .. }
    ));
    assert!(product_resolve(&prefixed).is_ok());

    let duplicate = vec![
        entry("package/package.json", package_manifest(11)),
        entry(
            "package/package.json",
            serde_json::to_vec(&json!({
                "name": "wrong.identity",
                "version": "9.9.9"
            }))
            .expect("serialize second duplicate manifest"),
        ),
    ];
    assert!(matches!(
        archive_manifest_model::evaluate(&duplicate, production_limits()),
        Outcome::AcceptManifest { entry_index: 0, .. }
    ));
    assert!(
        product_resolve(&duplicate).is_ok(),
        "first duplicate manifest must remain authoritative"
    );
}

#[test]
fn af02_archive_manifest_model_limit_mutations_are_deterministic() {
    let manifest = entry("package/package.json", package_manifest(1));
    assert_eq!(
        archive_manifest_model::evaluate(
            std::slice::from_ref(&manifest),
            Limits {
                max_entries: 8,
                max_manifest_bytes: 4,
                max_decompressed_bytes: u64::MAX,
            },
        ),
        Outcome::ManifestTooLarge
    );

    let two_entries = vec![
        entry("package/a", Vec::new()),
        entry("package/b", Vec::new()),
    ];
    assert_eq!(
        archive_manifest_model::evaluate(
            &two_entries,
            Limits {
                max_entries: 1,
                max_manifest_bytes: u64::MAX,
                max_decompressed_bytes: u64::MAX,
            },
        ),
        Outcome::EntryLimit
    );

    let decompressed = vec![LogicalEntry {
        path: "package/noise".to_owned(),
        body: Vec::new(),
        decompressed_bytes: 10,
    }];
    assert_eq!(
        archive_manifest_model::evaluate(
            &decompressed,
            Limits {
                max_entries: 8,
                max_manifest_bytes: u64::MAX,
                max_decompressed_bytes: 9,
            },
        ),
        Outcome::DecompressedLimit
    );
}

#[test]
fn af02_archive_manifest_property_identity_is_frozen() {
    assert_eq!(PROPERTY_ID, "PROP-ARCHIVE-MANIFEST-001");
    assert_eq!(property_runner::CASE_COUNT, 256);
    assert_eq!(property_runner::MAX_SHRINK_ITERS, 4096);
    assert_eq!(
        SEED_HEX,
        "9f2ff50e2d1382752f79e92585f4fec473081f43d327cf810cc594cf28689dd8"
    );
    assert_eq!(archive_manifest_model::INVALIDITY_CLASSES.len(), 7);
}
