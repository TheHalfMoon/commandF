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
    let before_bytes = before_cache.read_verified(&before_locked.sha256)?;
    let after_bytes = after_cache.read_verified(&after_locked.sha256)?;
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
