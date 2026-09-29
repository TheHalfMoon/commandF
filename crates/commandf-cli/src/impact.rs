use std::fs;
use std::io;
use std::path::PathBuf;

use commandf_pkg::{
    build_context_graph, build_impact_report, diff_package_archives, LockedPackage, Lockfile,
    PackageCache, PackageName,
};

pub fn run(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    let before_lockfile = Lockfile::from_slice(&fs::read(before_lock)?)?;
    let after_lockfile = Lockfile::from_slice(&fs::read(after_lock)?)?;
    require_lock_v2(&before_lockfile, "before")?;
    require_lock_v2(&after_lockfile, "after")?;
    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;

    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);
    let before_bytes = read_locked_archive(&before_cache, before_locked)?;
    let after_bytes = read_locked_archive(&after_cache, after_locked)?;
    let diff = diff_package_archives(
        package_name.to_string(),
        &before_locked.version,
        &before_locked.sha256,
        &before_bytes,
        &after_locked.version,
        &after_locked.sha256,
        &after_bytes,
    )?;
    let before_graph = build_context_graph(&before_lockfile, &before_cache)?;
    let after_graph = build_context_graph(&after_lockfile, &after_cache)?;
    let report = build_impact_report(&diff, &before_graph, &after_graph)?;
    Ok(report.to_json_bytes()?)
}

fn require_lock_v2(lockfile: &Lockfile, side: &'static str) -> io::Result<()> {
    if lockfile.schema == Lockfile::SCHEMA_V2 {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "impact requires commandf.lock schema 2 on {side}; found schema {}",
            lockfile.schema
        ),
    ))
}

fn read_locked_archive(
    cache: &PackageCache,
    locked: &LockedPackage,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(cache.read_verified(&locked.sha256)?)
}

fn select_locked_package<'a>(
    lockfile: &'a Lockfile,
    package_name: &str,
) -> Result<&'a LockedPackage, io::Error> {
    let mut matches = lockfile
        .packages
        .iter()
        .filter(|candidate| candidate.name == package_name);
    let selected = matches.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("package {package_name} is not present in the lockfile"),
        )
    })?;
    if matches.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("package {package_name} appears more than once in the lockfile"),
        ));
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impact_reader_returns_verified_bytes() {
        let directory = unique_dir("impact-verified");
        let cache = PackageCache::new(&directory);
        let digest = cache.put(b"impact-archive").expect("cache object");
        let bytes = read_locked_archive(&cache, &locked(&digest)).expect("verified bytes");
        assert_eq!(bytes, b"impact-archive");
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn replaced_impact_cache_object_is_rejected() {
        let directory = unique_dir("impact-replaced");
        let cache = PackageCache::new(&directory);
        let digest = cache.put(b"impact-archive").expect("cache object");
        fs::write(
            cache.root().join("sha256").join(format!("{digest}.tgz")),
            b"replaced-impact-archive",
        )
        .expect("replace cache object");
        let error = read_locked_archive(&cache, &locked(&digest))
            .expect_err("replaced cache object must fail closed");
        assert!(error.to_string().contains("cache object digest mismatch"));
        let _ = fs::remove_dir_all(directory);
    }

    fn locked(digest: &str) -> LockedPackage {
        LockedPackage {
            name: "acme.subject".to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest.to_owned(),
            source: "memory".to_owned(),
            dependencies: Default::default(),
        }
    }

    fn unique_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("commandf-{label}-{}", std::process::id()))
    }
}
