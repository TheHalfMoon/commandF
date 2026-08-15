use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use commandf_pkg::{
    diff_package_archives, matched_structure_definition_pairs, reconcile_hl7_oracle,
    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation, LockedPackage,
    Lockfile, OracleDivergenceReport, PackageCache, PackageName, PackageRequest, ResourceKey,
    ResourceKeyKind, VersionConstraint, DEFAULT_ORACLE_TIMEOUT_SECS,
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
    run_report_inner(
        package,
        before_lock,
        before_cache,
        after_lock,
        after_cache,
        oracle_adapter,
        oracle_java,
        false,
    )
}

pub fn run_changed_report(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
    oracle_adapter: PathBuf,
    oracle_java: Option<PathBuf>,
) -> Result<OracleDivergenceReport, Box<dyn std::error::Error>> {
    run_report_inner(
        package,
        before_lock,
        before_cache,
        after_lock,
        after_cache,
        oracle_adapter,
        oracle_java,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_report_inner(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
    oracle_adapter: PathBuf,
    oracle_java: Option<PathBuf>,
    changed_only: bool,
) -> Result<OracleDivergenceReport, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    validate_hl7_oracle_adapter(&oracle_adapter, oracle_java.as_deref())?;

    let before_lockfile = Lockfile::from_slice(&fs::read(&before_lock)?)?;
    let after_lockfile = Lockfile::from_slice(&fs::read(&after_lock)?)?;
    let before_locked = select_root_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_root_locked_package(&after_lockfile, package_name.as_str())?;
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
    before_lockfile.verify_cache(&before_cache)?;
    after_lockfile.verify_cache(&after_cache)?;

    let before_archive = archive_path(&before_cache, before_locked);
    let after_archive = archive_path(&after_cache, after_locked);
    let core_archive = archive_path(&before_cache, before_core);
    let before_context_archives =
        dependency_context_archives(&before_lockfile, &before_cache, before_locked)?;
    let after_context_archives =
        dependency_context_archives(&after_lockfile, &after_cache, after_locked)?;
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
            if changed_only
                && !structural_diff
                    .changes
                    .iter()
                    .any(|change| change.resource == pair.resource)
            {
                continue;
            }
            let (url, version) = canonical_parts(&pair.resource)?;
            let invocation = Hl7OracleInvocation {
                core_package: &core_archive,
                left_package: &before_archive,
                right_package: &after_archive,
                left_context_packages: &before_context_archives,
                right_context_packages: &after_context_archives,
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
    let request = parse_package_request(&format!("{ORACLE_CORE_PACKAGE}@{ORACLE_CORE_VERSION}"))?;
    select_matching_locked_package(lockfile, &request, "oracle core")
}

fn select_root_locked_package<'a>(
    lockfile: &'a Lockfile,
    package_name: &str,
) -> Result<&'a LockedPackage, io::Error> {
    let mut requests = lockfile
        .roots
        .iter()
        .map(|root| parse_package_request(root))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|request| request.name.as_str() == package_name);
    let request = requests.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("root package {package_name} is not present in the lockfile roots"),
        )
    })?;
    if requests.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("root package {package_name} appears more than once in lockfile roots"),
        ));
    }
    select_matching_locked_package(lockfile, &request, "root")
}

fn dependency_context_archives(
    lockfile: &Lockfile,
    cache: &PackageCache,
    root: &LockedPackage,
) -> Result<Vec<PathBuf>, io::Error> {
    dependency_context_packages(lockfile, root).map(|packages| {
        packages
            .into_iter()
            .map(|package| archive_path(cache, package))
            .collect()
    })
}

fn dependency_context_packages<'a>(
    lockfile: &'a Lockfile,
    root: &'a LockedPackage,
) -> Result<Vec<&'a LockedPackage>, io::Error> {
    let mut visited = BTreeSet::new();
    visited.insert(package_identity(root));
    let mut packages = Vec::new();
    collect_dependency_context_packages(lockfile, root, &mut visited, &mut packages)?;
    Ok(packages)
}

fn collect_dependency_context_packages<'a>(
    lockfile: &'a Lockfile,
    parent: &'a LockedPackage,
    visited: &mut BTreeSet<String>,
    output: &mut Vec<&'a LockedPackage>,
) -> Result<(), io::Error> {
    for (dependency_name, constraint) in &parent.dependencies {
        let request = parse_package_request(&format!("{dependency_name}@{constraint}"))?;
        let dependency = select_matching_locked_package(lockfile, &request, "dependency")?;
        let identity = package_identity(dependency);
        if !visited.insert(identity) {
            continue;
        }
        if dependency.name == ORACLE_CORE_PACKAGE && dependency.version == ORACLE_CORE_VERSION {
            continue;
        }
        collect_dependency_context_packages(lockfile, dependency, visited, output)?;
        output.push(dependency);
    }
    Ok(())
}

fn select_matching_locked_package<'a>(
    lockfile: &'a Lockfile,
    request: &PackageRequest,
    role: &str,
) -> Result<&'a LockedPackage, io::Error> {
    let mut matches = Vec::new();
    for candidate in &lockfile.packages {
        if candidate.name != request.name.as_str() {
            continue;
        }
        let candidate_request =
            parse_package_request(&format!("{}@{}", candidate.name, candidate.version))?;
        let VersionConstraint::Exact(candidate_version) = candidate_request.constraint else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "locked package version is not exact: {}@{}",
                    candidate.name, candidate.version
                ),
            ));
        };
        if request.constraint.matches(&candidate_version) {
            matches.push(candidate);
        }
    }

    match matches.as_slice() {
        [single] => Ok(*single),
        [] => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "{role} request {} has no matching locked package",
                request.display()
            ),
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{role} request {} matches {} locked packages",
                request.display(),
                matches.len()
            ),
        )),
    }
}

fn parse_package_request(raw: &str) -> Result<PackageRequest, io::Error> {
    PackageRequest::parse(raw).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid lock package request {raw}: {error}"),
        )
    })
}

fn package_identity(package: &LockedPackage) -> String {
    format!("{}@{}", package.name, package.version)
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn locked(name: &str, version: &str, dependencies: &[(&str, &str)]) -> LockedPackage {
        LockedPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            sha256: format!("{name}-{version}"),
            source: format!("https://example.test/{name}/{version}"),
            dependencies: dependencies
                .iter()
                .map(|(name, version)| ((*name).to_owned(), (*version).to_owned()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    #[test]
    fn root_selection_uses_root_request_not_name_only() {
        let lockfile = Lockfile::new(
            vec!["example.root@2.0.0".to_owned()],
            vec![
                locked("example.root", "1.0.0", &[]),
                locked("example.root", "2.0.0", &[]),
            ],
        );
        let selected = select_root_locked_package(&lockfile, "example.root").unwrap();
        assert_eq!(selected.version, "2.0.0");
    }

    #[test]
    fn dependency_context_is_deterministic_leaf_first_and_excludes_core() {
        let root = locked(
            "example.root",
            "1.0.0",
            &[
                ("example.a", "1.0.0"),
                (ORACLE_CORE_PACKAGE, ORACLE_CORE_VERSION),
            ],
        );
        let lockfile = Lockfile::new(
            vec!["example.root@1.0.0".to_owned()],
            vec![
                root,
                locked("example.a", "1.0.0", &[("example.b", "2.0.x")]),
                locked("example.b", "2.0.3", &[]),
                locked(ORACLE_CORE_PACKAGE, ORACLE_CORE_VERSION, &[]),
            ],
        );
        let root = select_root_locked_package(&lockfile, "example.root").unwrap();
        let contexts = dependency_context_packages(&lockfile, root).unwrap();
        assert_eq!(
            contexts
                .iter()
                .map(|package| package_identity(package))
                .collect::<Vec<_>>(),
            vec!["example.b@2.0.3", "example.a@1.0.0"]
        );
    }

    #[test]
    fn ambiguous_dependency_constraint_fails_closed() {
        let root = locked("example.root", "1.0.0", &[("example.dep", "1.0.x")]);
        let lockfile = Lockfile::new(
            vec!["example.root@1.0.0".to_owned()],
            vec![
                root,
                locked("example.dep", "1.0.1", &[]),
                locked("example.dep", "1.0.2", &[]),
            ],
        );
        let root = select_root_locked_package(&lockfile, "example.root").unwrap();
        let error = dependency_context_packages(&lockfile, root).unwrap_err();
        assert!(
            error.to_string().contains("matches 2 locked packages"),
            "{error}"
        );
    }
}
