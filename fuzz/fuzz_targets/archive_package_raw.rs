#![no_main]

use std::fs;
use std::path::{Path, PathBuf};

use commandf_pkg::{
    inspect_package, LocalMirrorSource, PackageCache, PackageRequest, Resolver,
};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 256 * 1024;
const PACKAGE_NAME: &str = "af02.synthetic";
const PACKAGE_VERSION: &str = "1.0.0";

fn scratch_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "commandf-af02-archive-fuzz-{}",
        std::process::id()
    ))
}

fn remove_scratch(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn exercise_archive_package(bytes: &[u8]) {
    if bytes.len() > MAX_INPUT_BYTES {
        return;
    }

    let _ = inspect_package(
        PACKAGE_NAME,
        PACKAGE_VERSION,
        PackageCache::digest(bytes),
        bytes,
    );

    let root = scratch_root();
    remove_scratch(&root);
    let archive_dir = root
        .join("mirror")
        .join(PACKAGE_NAME)
        .join(PACKAGE_VERSION);
    if fs::create_dir_all(&archive_dir).is_err() {
        remove_scratch(&root);
        return;
    }
    if fs::write(archive_dir.join("package.tgz"), bytes).is_err() {
        remove_scratch(&root);
        return;
    }

    let source = LocalMirrorSource::new(root.join("mirror"));
    let cache = PackageCache::new(root.join("cache"));
    let request = match PackageRequest::parse("af02.synthetic@1.0.0") {
        Ok(request) => request,
        Err(_) => {
            remove_scratch(&root);
            return;
        }
    };
    let _ = Resolver::new(&source, &cache).resolve(vec![request]);
    remove_scratch(&root);
}

fuzz_target!(|data: &[u8]| {
    exercise_archive_package(data);
});
