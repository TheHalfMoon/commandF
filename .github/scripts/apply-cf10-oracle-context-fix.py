from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one replacement, found {count}: {old[:80]!r}")
    file.write_text(text.replace(old, new, 1))


# ---------------------------------------------------------------------------
# Rust process boundary: pass deterministic verified dependency context paths.
# ---------------------------------------------------------------------------
replace_once(
    "crates/commandf-pkg/src/oracle_process.rs",
    "use std::path::Path;",
    "use std::path::{Path, PathBuf};",
)
replace_once(
    "crates/commandf-pkg/src/oracle_process.rs",
    '''pub struct Hl7OracleInvocation<'a> {
    pub core_package: &'a Path,
    pub left_package: &'a Path,
    pub right_package: &'a Path,
    pub left_url: &'a str,
    pub left_version: Option<&'a str>,
    pub right_url: &'a str,
    pub right_version: Option<&'a str>,
}''',
    '''pub struct Hl7OracleInvocation<'a> {
    pub core_package: &'a Path,
    pub left_package: &'a Path,
    pub right_package: &'a Path,
    pub left_context_packages: &'a [PathBuf],
    pub right_context_packages: &'a [PathBuf],
    pub left_url: &'a str,
    pub left_version: Option<&'a str>,
    pub right_url: &'a str,
    pub right_version: Option<&'a str>,
}''',
)
replace_once(
    "crates/commandf-pkg/src/oracle_process.rs",
    '''    command
        .arg("--core-package")
        .arg(invocation.core_package)
        .arg("--left-package")
        .arg(invocation.left_package)
        .arg("--right-package")
        .arg(invocation.right_package)
        .arg("--left-url")
        .arg(invocation.left_url)
        .arg("--right-url")
        .arg(invocation.right_url);
''',
    '''    command
        .arg("--core-package")
        .arg(invocation.core_package)
        .arg("--left-package")
        .arg(invocation.left_package)
        .arg("--right-package")
        .arg(invocation.right_package);

    for context_package in invocation.left_context_packages {
        command
            .arg("--left-context-package")
            .arg(context_package);
    }
    for context_package in invocation.right_context_packages {
        command
            .arg("--right-context-package")
            .arg(context_package);
    }

    command
        .arg("--left-url")
        .arg(invocation.left_url)
        .arg("--right-url")
        .arg(invocation.right_url);
''',
)

# ---------------------------------------------------------------------------
# Rust process tests: keep old callers empty-context and prove repeat argv order.
# ---------------------------------------------------------------------------
replace_once(
    "crates/commandf-pkg/tests/oracle_process.rs",
    '''    let invocation = Hl7OracleInvocation {
        core_package: core,
        left_package: left,
        right_package: right,
        left_url: "http://example.org/StructureDefinition/test",''',
    '''    let context_packages = Vec::<PathBuf>::new();
    let invocation = Hl7OracleInvocation {
        core_package: core,
        left_package: left,
        right_package: right,
        left_context_packages: &context_packages,
        right_context_packages: &context_packages,
        left_url: "http://example.org/StructureDefinition/test",''',
)
replace_once(
    "crates/commandf-pkg/tests/oracle_process.rs",
    '''#[test]
fn jar_adapter_requires_explicit_java_path() {''',
    '''#[test]
fn context_package_arguments_are_forwarded_in_input_order() {
    let root = unique_temp_dir("contexts");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.sh");
    let capture = root.join("args.txt");
    write_executable(
        &adapter,
        &format!(
            "printf '%s\\n' \\\"$@\\\" > '{}'; printf '%s\\n' '{}'",
            capture.display(),
            GOOD_REPORT
        ),
    );
    let (core, left, right) = package_inputs(&root);
    let left_contexts = vec![root.join("left-a.tgz"), root.join("left-b.tgz")];
    let right_contexts = vec![root.join("right-a.tgz")];
    for path in left_contexts.iter().chain(right_contexts.iter()) {
        fs::write(path, b"fixture").expect("write context fixture");
    }

    let invocation = Hl7OracleInvocation {
        core_package: &core,
        left_package: &left,
        right_package: &right,
        left_context_packages: &left_contexts,
        right_context_packages: &right_contexts,
        left_url: "http://example.org/StructureDefinition/test",
        left_version: None,
        right_url: "http://example.org/StructureDefinition/test",
        right_version: None,
    };
    run_hl7_oracle_adapter(&adapter, None, &invocation, Duration::from_secs(1))
        .expect("context argv adapter report");

    let args = fs::read_to_string(&capture)
        .expect("read captured args")
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let context_args = args
        .windows(2)
        .filter(|pair| pair[0] == "--left-context-package" || pair[0] == "--right-context-package")
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        context_args,
        vec![
            (
                "--left-context-package".to_owned(),
                left_contexts[0].display().to_string(),
            ),
            (
                "--left-context-package".to_owned(),
                left_contexts[1].display().to_string(),
            ),
            (
                "--right-context-package".to_owned(),
                right_contexts[0].display().to_string(),
            ),
        ]
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn jar_adapter_requires_explicit_java_path() {''',
)

# ---------------------------------------------------------------------------
# CLI oracle: exact root selection and fail-closed dependency closure traversal.
# ---------------------------------------------------------------------------
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    "use std::fs;\nuse std::io;",
    "use std::collections::BTreeSet;\nuse std::fs;\nuse std::io;",
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation, LockedPackage,
    Lockfile, OracleDivergenceReport, PackageCache, PackageName, ResourceKey, ResourceKeyKind,
    DEFAULT_ORACLE_TIMEOUT_SECS,
''',
    '''    run_hl7_oracle_adapter, validate_hl7_oracle_adapter, Hl7OracleInvocation, LockedPackage,
    Lockfile, OracleDivergenceReport, PackageCache, PackageName, PackageRequest, ResourceKey,
    ResourceKeyKind, VersionConstraint, DEFAULT_ORACLE_TIMEOUT_SECS,
''',
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;
    let before_core = select_oracle_core(&before_lockfile)?;
    let after_core = select_oracle_core(&after_lockfile)?;
''',
    '''    let before_locked = select_root_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_root_locked_package(&after_lockfile, package_name.as_str())?;
    let before_core = select_oracle_core(&before_lockfile)?;
    let after_core = select_oracle_core(&after_lockfile)?;
''',
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);
    before_cache.verify(&before_locked.sha256)?;
    after_cache.verify(&after_locked.sha256)?;
    before_cache.verify(&before_core.sha256)?;
    after_cache.verify(&after_core.sha256)?;

    let before_archive = archive_path(&before_cache, before_locked);
    let after_archive = archive_path(&after_cache, after_locked);
    let core_archive = archive_path(&before_cache, before_core);
''',
    '''    let before_cache = PackageCache::new(before_cache);
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
''',
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''            let invocation = Hl7OracleInvocation {
                core_package: &core_archive,
                left_package: &before_archive,
                right_package: &after_archive,
                left_url: url,''',
    '''            let invocation = Hl7OracleInvocation {
                core_package: &core_archive,
                left_package: &before_archive,
                right_package: &after_archive,
                left_context_packages: &before_context_archives,
                right_context_packages: &after_context_archives,
                left_url: url,''',
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''fn select_oracle_core(lockfile: &Lockfile) -> Result<&LockedPackage, io::Error> {
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
''',
    '''fn select_oracle_core(lockfile: &Lockfile) -> Result<&LockedPackage, io::Error> {
    let request = parse_package_request(&format!(
        "{ORACLE_CORE_PACKAGE}@{ORACLE_CORE_VERSION}"
    ))?;
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
        let candidate_request = parse_package_request(&format!(
            "{}@{}",
            candidate.name, candidate.version
        ))?;
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
            format!("{role} request {} has no matching locked package", request.display()),
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
''',
)
replace_once(
    "crates/commandf-cli/src/oracle.rs",
    '''fn canonical_parts(resource: &ResourceKey) -> Result<(&str, Option<&str>), io::Error> {''',
    '''#[cfg(test)]
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
            &[("example.a", "1.0.0"), (ORACLE_CORE_PACKAGE, ORACLE_CORE_VERSION)],
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

fn canonical_parts(resource: &ResourceKey) -> Result<(&str, Option<&str>), io::Error> {''',
)

# ---------------------------------------------------------------------------
# Java adapter: repeated verified context packages loaded before the side root.
# ---------------------------------------------------------------------------
replace_once(
    "tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java",
    '''    ContextAndPackage left = loadContext(args.corePackage(), args.leftPackage());
    ContextAndPackage right = loadContext(args.corePackage(), args.rightPackage());''',
    '''    ContextAndPackage left = loadContext(
        args.corePackage(), args.leftContextPackages(), args.leftPackage());
    ContextAndPackage right = loadContext(
        args.corePackage(), args.rightContextPackages(), args.rightPackage());''',
)
replace_once(
    "tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java",
    '''  private static ContextAndPackage loadContext(Path corePath, Path sidePath) throws Exception {
    NpmPackage core = loadPackage(corePath);
    requirePackage(core, CORE_PACKAGE_NAME, CORE_PACKAGE_VERSION, "core");

    IContextResourceLoader coreLoader = ValidatorUtils.loaderForVersion(core.fhirVersion());
    SimpleWorkerContext context = new SimpleWorkerContext.SimpleWorkerContextBuilder()
        .withAllowLoadingDuplicates(true)
        .fromPackage(core, coreLoader, false);
    context.setAllowLoadingDuplicates(false);
    context.setCanRunWithoutTerminology(true);

    NpmPackage side = loadPackage(sidePath);
    if (!samePackage(core, side)) {
      IContextResourceLoader sideLoader = ValidatorUtils.loaderForVersion(side.fhirVersion());
      context.loadFromPackage(side, sideLoader, false);
    }
    return new ContextAndPackage(context, side.name(), side.version());
  }''',
    '''  private static ContextAndPackage loadContext(
      Path corePath, List<Path> contextPaths, Path sidePath) throws Exception {
    NpmPackage core = loadPackage(corePath);
    requirePackage(core, CORE_PACKAGE_NAME, CORE_PACKAGE_VERSION, "core");

    IContextResourceLoader coreLoader = ValidatorUtils.loaderForVersion(core.fhirVersion());
    SimpleWorkerContext context = new SimpleWorkerContext.SimpleWorkerContextBuilder()
        .withAllowLoadingDuplicates(true)
        .fromPackage(core, coreLoader, false);
    context.setAllowLoadingDuplicates(false);
    context.setCanRunWithoutTerminology(true);

    NpmPackage side = loadPackage(sidePath);
    for (Path contextPath : contextPaths) {
      NpmPackage dependency = loadPackage(contextPath);
      if (samePackage(core, dependency) || samePackage(side, dependency)) {
        continue;
      }
      IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());
      context.loadFromPackage(dependency, dependencyLoader, false);
    }
    if (!samePackage(core, side)) {
      IContextResourceLoader sideLoader = ValidatorUtils.loaderForVersion(side.fhirVersion());
      context.loadFromPackage(side, sideLoader, false);
    }
    return new ContextAndPackage(context, side.name(), side.version());
  }''',
)
replace_once(
    "tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java",
    '''  record Arguments(
      Path corePackage,
      Path leftPackage,
      Path rightPackage,
      String leftUrl,
      String leftVersion,
      String rightUrl,
      String rightVersion) {

    static Arguments parse(String[] args) {
      Map<String, String> values = new LinkedHashMap<>();
      for (int index = 0; index < args.length; index += 2) {
        if (index + 1 >= args.length) {
          throw new IllegalArgumentException("missing value for " + args[index]);
        }
        String key = args[index];
        if (!key.startsWith("--")) {
          throw new IllegalArgumentException("unexpected positional argument: " + key);
        }
        if (values.put(key, args[index + 1]) != null) {
          throw new IllegalArgumentException("duplicate argument: " + key);
        }
      }

      List<String> allowed = List.of(
          "--core-package",
          "--left-package",
          "--right-package",
          "--left-url",
          "--left-version",
          "--right-url",
          "--right-version");
      List<String> unknown = new ArrayList<>();
      for (String key : values.keySet()) {
        if (!allowed.contains(key)) {
          unknown.add(key);
        }
      }
      if (!unknown.isEmpty()) {
        throw new IllegalArgumentException("unknown arguments: " + String.join(", ", unknown));
      }

      return new Arguments(
          Path.of(required(values, "--core-package")),
          Path.of(required(values, "--left-package")),
          Path.of(required(values, "--right-package")),
          required(values, "--left-url"),
          values.get("--left-version"),
          required(values, "--right-url"),
          values.get("--right-version"));
    }''',
    '''  record Arguments(
      Path corePackage,
      Path leftPackage,
      Path rightPackage,
      List<Path> leftContextPackages,
      List<Path> rightContextPackages,
      String leftUrl,
      String leftVersion,
      String rightUrl,
      String rightVersion) {

    static Arguments parse(String[] args) {
      Map<String, String> values = new LinkedHashMap<>();
      List<Path> leftContextPackages = new ArrayList<>();
      List<Path> rightContextPackages = new ArrayList<>();
      for (int index = 0; index < args.length; index += 2) {
        if (index + 1 >= args.length) {
          throw new IllegalArgumentException("missing value for " + args[index]);
        }
        String key = args[index];
        if (!key.startsWith("--")) {
          throw new IllegalArgumentException("unexpected positional argument: " + key);
        }
        String value = args[index + 1];
        if (key.equals("--left-context-package")) {
          leftContextPackages.add(Path.of(value));
        } else if (key.equals("--right-context-package")) {
          rightContextPackages.add(Path.of(value));
        } else if (values.put(key, value) != null) {
          throw new IllegalArgumentException("duplicate argument: " + key);
        }
      }

      List<String> allowed = List.of(
          "--core-package",
          "--left-package",
          "--right-package",
          "--left-url",
          "--left-version",
          "--right-url",
          "--right-version");
      List<String> unknown = new ArrayList<>();
      for (String key : values.keySet()) {
        if (!allowed.contains(key)) {
          unknown.add(key);
        }
      }
      if (!unknown.isEmpty()) {
        throw new IllegalArgumentException("unknown arguments: " + String.join(", ", unknown));
      }

      return new Arguments(
          Path.of(required(values, "--core-package")),
          Path.of(required(values, "--left-package")),
          Path.of(required(values, "--right-package")),
          List.copyOf(leftContextPackages),
          List.copyOf(rightContextPackages),
          required(values, "--left-url"),
          values.get("--left-version"),
          required(values, "--right-url"),
          values.get("--right-version"));
    }''',
)

print("CF-10 oracle context fix staged")
