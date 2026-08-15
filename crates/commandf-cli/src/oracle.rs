use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use commandf_pkg::{
    diff_package_archives, matched_structure_definition_pairs, reconcile_hl7_oracle,
    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation, LockedPackage,
    Lockfile, OracleDivergenceReport, PackageCache, PackageName, ResourceKey, ResourceKeyKind,
    DEFAULT_ORACLE_TIMEOUT_SECS,
};

const ORACLE_CORE_PACKAGE: &str = "hl7.fhir.r4.core";
const ORACLE_CORE_VERSION: &str = "4.0.1";

pub fn run(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
    oracle_adapter: PathBuf,
    oracle_java: Option<PathBuf>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(run_report(
        package,
        before_lock,
        before_cache,
        after_lock,
        after_cache,
        oracle_adapter,
        oracle_java,
    )?
    .to_json_bytes()?)
}

pub fn run_report(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
    oracle_adapter: PathBuf,
    oracle_java: Option<PathBuf>,
) -> Result<OracleDivergenceReport, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    validate_hl7_oracle_adapter(&oracle_adapter, oracle_java.as_deref())?;

    let before_lockfile = Lockfile::from_slice(&fs::read(&before_lock)?)?;
    let after_lockfile = Lockfile::from_slice(&fs::read(&after_lock)?)?;
    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;
    let before_core = select_oracle_core(&before_lockfile)?;
    let after_core = select_oracle_core(&after_lockfile)?;

    if before_core.sha256 != after_core.sha256 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "oracle core package digest differs between states: {} != {}",
                before_core.sha256, after_core.sha256
            ),
        )
        .into());
    }

    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);
    before_cache.verify(&before_locked.sha256)?;
    after_cache.verify(&after_locked.sha256)?;
    before_cache.verify(&before_core.sha256)?;
    after_cache.verify(&after_core.sha256)?;

    let before_archive = archive_path(&before_cache, before_locked);
    let after_archive = archive_path(&after_cache, after_locked);
    let core_archive = archive_path(&before_cache, before_core);
    let before_bytes = fs::read(&before_archive)?;
    let after_bytes = fs::read(&after_archive)?;

    let structural_diff = diff_package_archives(
        package_name.to_string(),
        &before_locked.version,
        &before_locked.sha256,
        &before_bytes,
        &after_locked.version,
        &after_locked.sha256,
        &after_bytes,
    )?;

    let mut observations = Vec::new();
    if before_locked.sha256 != after_locked.sha256 {
        let pairs = matched_structure_definition_pairs(
            package_name.as_str(),
            &before_locked.version,
            &before_locked.sha256,
            &before_bytes,
            &after_locked.version,
            &after_locked.sha256,
            &after_bytes,
        )?;

        for pair in pairs {
            if pair.resource.kind != ResourceKeyKind::Canonical {
                continue;
            }
            let (url, version) = canonical_parts(&pair.resource)?;
            let invocation = Hl7OracleInvocation {
                core_package: &core_archive,
                left_package: &before_archive,
                right_package: &after_archive,
                left_url: url,
                left_version: version,
                right_url: url,
                right_version: version,
            };
            let observation = run_hl7_oracle_adapter(
                &oracle_adapter,
                oracle_java.as_deref(),
                &invocation,
                Duration::from_secs(DEFAULT_ORACLE_TIMEOUT_SECS),
            )?;
            observations.push((pair.resource, observation));
        }
    }

    Ok(reconcile_hl7_oracle(structural_diff, observations)?)
}

fn select_oracle_core(lockfile: &Lockfile) -> Result<&LockedPackage, io::Error> {
    let core = select_locked_package(lockfile, ORACLE_CORE_PACKAGE)?;
    if core.version != ORACLE_CORE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "CF-06 requires {ORACLE_CORE_PACKAGE}@{ORACLE_CORE_VERSION}, found {}@{}",
                core.name, core.version
            ),
        ));
    }
    Ok(core)
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

fn archive_path(cache: &PackageCache, package: &LockedPackage) -> PathBuf {
    cache
        .root()
        .join("sha256")
        .join(format!("{}.tgz", package.sha256))
}

fn canonical_parts(resource: &ResourceKey) -> Result<(&str, Option<&str>), io::Error> {
    if resource.kind != ResourceKeyKind::Canonical {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "HL7 oracle comparison requires a canonical CF-03 resource key",
        ));
    }
    if let Some((url, version)) = resource.value.rsplit_once('|') {
        if url.is_empty() || version.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid qualified canonical resource key {}",
                    resource.value
                ),
            ));
        }
        Ok((url, Some(version)))
    } else {
        Ok((resource.value.as_str(), None))
    }
}
