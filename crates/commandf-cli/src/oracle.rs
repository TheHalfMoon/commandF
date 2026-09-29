use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use commandf_pkg::{
    diff_package_archives, matched_structure_definition_pairs, reconcile_hl7_oracle,
    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation, LockedPackage,
    Lockfile, PackageCache, PackageName, ResourceKey, ResourceKeyKind, DEFAULT_ORACLE_TIMEOUT_SECS,
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
    let staged = stage_verified_archives(
        &before_cache,
        &after_cache,
        before_locked,
        after_locked,
        before_core,
        after_core,
    )?;

    let structural_diff = diff_package_archives(
        package_name.to_string(),
        &before_locked.version,
        &before_locked.sha256,
        &staged.before_bytes,
        &after_locked.version,
        &after_locked.sha256,
        &staged.after_bytes,
    )?;

    let mut observations = Vec::new();
    if before_locked.sha256 != after_locked.sha256 {
        let pairs = matched_structure_definition_pairs(
            package_name.as_str(),
            &before_locked.version,
            &before_locked.sha256,
            &staged.before_bytes,
            &after_locked.version,
            &after_locked.sha256,
            &staged.after_bytes,
        )?;

        for pair in pairs {
            if pair.resource.kind != ResourceKeyKind::Canonical {
                continue;
            }
            let (url, version) = canonical_parts(&pair.resource)?;
            let invocation = Hl7OracleInvocation {
                core_package: &staged.core_path,
                left_package: &staged.before_path,
                right_package: &staged.after_path,
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

    let report = reconcile_hl7_oracle(structural_diff, observations)?;
    Ok(report.to_json_bytes()?)
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

struct StagedOracleArchives {
    root: PathBuf,
    before_bytes: Vec<u8>,
    after_bytes: Vec<u8>,
    before_path: PathBuf,
    after_path: PathBuf,
    core_path: PathBuf,
}

impl Drop for StagedOracleArchives {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn stage_verified_archives(
    before_cache: &PackageCache,
    after_cache: &PackageCache,
    before_locked: &LockedPackage,
    after_locked: &LockedPackage,
    before_core: &LockedPackage,
    after_core: &LockedPackage,
) -> Result<StagedOracleArchives, Box<dyn std::error::Error>> {
    let before_bytes = before_cache.read_verified(&before_locked.sha256)?;
    let after_bytes = after_cache.read_verified(&after_locked.sha256)?;
    let core_bytes = before_cache.read_verified(&before_core.sha256)?;
    let after_core_bytes = after_cache.read_verified(&after_core.sha256)?;
    if after_core_bytes != core_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "oracle core package bytes differ between verified cache reads",
        )
        .into());
    }

    let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "commandf-oracle-stage-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    restrict_private_directory(&root)?;
    let before_path = root.join("before.tgz");
    let after_path = root.join("after.tgz");
    let core_path = root.join("core.tgz");
    write_private_file(&before_path, &before_bytes)?;
    write_private_file(&after_path, &after_bytes)?;
    write_private_file(&core_path, &core_bytes)?;
    Ok(StagedOracleArchives {
        root,
        before_bytes,
        after_bytes,
        before_path,
        after_path,
        core_path,
    })
}

fn restrict_private_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
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
    use super::*;

    #[test]
    fn staged_oracle_bytes_survive_cache_replacement() {
        let before_root = unique_dir("oracle-before");
        let after_root = unique_dir("oracle-after");
        let before_cache = PackageCache::new(&before_root);
        let after_cache = PackageCache::new(&after_root);
        let before_digest = before_cache.put(b"before-package").expect("before package");
        let after_digest = after_cache.put(b"after-package").expect("after package");
        let core_digest = before_cache.put(b"core-package").expect("before core");
        after_cache.put(b"core-package").expect("after core");

        let staged = stage_verified_archives(
            &before_cache,
            &after_cache,
            &locked("acme.subject", &before_digest),
            &locked("acme.subject", &after_digest),
            &locked(ORACLE_CORE_PACKAGE, &core_digest),
            &locked(ORACLE_CORE_PACKAGE, &core_digest),
        )
        .expect("stage verified archives");

        replace_cache(&before_cache, &before_digest, b"replaced-before");
        replace_cache(&after_cache, &after_digest, b"replaced-after");
        replace_cache(&before_cache, &core_digest, b"replaced-core");
        replace_cache(&after_cache, &core_digest, b"replaced-core-after");

        assert_eq!(
            fs::read(&staged.before_path).expect("staged before"),
            b"before-package"
        );
        assert_eq!(
            fs::read(&staged.after_path).expect("staged after"),
            b"after-package"
        );
        assert_eq!(
            fs::read(&staged.core_path).expect("staged core"),
            b"core-package"
        );
        assert!(!staged.before_path.starts_with(before_cache.root()));
        assert!(!staged.after_path.starts_with(after_cache.root()));
        assert_eq!(staged.before_bytes, b"before-package");
        assert_eq!(staged.after_bytes, b"after-package");

        let staged_root = staged.root.clone();
        drop(staged);
        assert!(!staged_root.exists());
        let _ = fs::remove_dir_all(before_root);
        let _ = fs::remove_dir_all(after_root);
    }

    fn locked(name: &str, digest: &str) -> LockedPackage {
        LockedPackage {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest.to_owned(),
            source: "memory".to_owned(),
            dependencies: Default::default(),
        }
    }

    fn replace_cache(cache: &PackageCache, digest: &str, bytes: &[u8]) {
        fs::write(
            cache.root().join("sha256").join(format!("{digest}.tgz")),
            bytes,
        )
        .expect("replace cache object");
    }

    fn unique_dir(label: &str) -> PathBuf {
        let sequence = STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "commandf-{label}-{}-{sequence}",
            std::process::id()
        ))
    }
}
