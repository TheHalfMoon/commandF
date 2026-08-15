from pathlib import Path
import re

EXPECTED_HEAD = "e5391d8ce968d001e1f9e3434b9e310acce248f8"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def regex_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one regex match, found {count}")
    return updated


# ---------------------------------------------------------------------------
# CF-07: scope dependency ValueSet closure to the FHIR core family of the
# exact root package while preserving whole-lock verification.
# ---------------------------------------------------------------------------
path = Path("crates/commandf-pkg/src/terminology.rs")
text = path.read_text()
text = replace_once(
    text,
    "    ElementView, Lockfile, PackageCache, PackageEvidence, ResourceKeyKind, StructuralDiffReport,\n",
    "    ElementView, LockedPackage, Lockfile, PackageCache, PackageEvidence, ResourceKeyKind,\n    StructuralDiffReport,\n",
    "terminology import LockedPackage",
)
text = replace_once(
    text,
    "    validate_root_evidence(\n        before.lockfile,\n        &structural.package_name,\n        &structural.before,\n    )?;\n    validate_root_evidence(after.lockfile, &structural.package_name, &structural.after)?;\n\n    let before_closure = TerminologyClosure::load(before.lockfile, before.cache)?;\n    let after_closure = TerminologyClosure::load(after.lockfile, after.cache)?;\n",
    "    let before_root = validate_root_evidence(\n        before.lockfile,\n        &structural.package_name,\n        &structural.before,\n    )?;\n    let after_root =\n        validate_root_evidence(after.lockfile, &structural.package_name, &structural.after)?;\n\n    let before_closure =\n        TerminologyClosure::load_for_root(before.lockfile, before.cache, before_root)?;\n    let after_closure =\n        TerminologyClosure::load_for_root(after.lockfile, after.cache, after_root)?;\n",
    "terminology scoped closure calls",
)
text = regex_once(
    text,
    r"fn validate_root_evidence\(\n    lockfile: &Lockfile,\n    package_name: &str,\n    evidence: &PackageEvidence,\n\) -> Result<\(\), TerminologyError> \{.*?\n    Ok\(\(\)\)\n\}",
    '''fn validate_root_evidence<'a>(\n    lockfile: &'a Lockfile,\n    package_name: &str,\n    evidence: &PackageEvidence,\n) -> Result<&'a LockedPackage, TerminologyError> {\n    let mut matches = lockfile.packages.iter().filter(|package| {\n        package.name == package_name\n            && package.version == evidence.version\n            && package.sha256 == evidence.archive_sha256\n    });\n    let Some(package) = matches.next() else {\n        return Err(TerminologyError::InvalidField {\n            resource: package_name.to_owned(),\n            field: "lockfile".to_owned(),\n            message: "exact root package identity is not present in the lockfile".to_owned(),\n        });\n    };\n    if matches.next().is_some() {\n        return Err(TerminologyError::InvalidField {\n            resource: package_name.to_owned(),\n            field: "lockfile".to_owned(),\n            message: "exact root package identity appears more than once in the lockfile".to_owned(),\n        });\n    }\n    Ok(package)\n}''',
    "terminology exact root validation",
)
path.write_text(text)

path = Path("crates/commandf-pkg/src/terminology_index.rs")
text = path.read_text()
text = replace_once(
    text,
    "    compare_value_set_expansions, Lockfile, PackageCache, PackageError, ResourceKey,\n    ResourceKeyKind, TerminologyError, TerminologyProofMode, TerminologyRelation,\n",
    "    compare_value_set_expansions, LockedPackage, Lockfile, PackageCache, PackageError,\n    PackageRequest, ResourceKey, ResourceKeyKind, TerminologyError, TerminologyProofMode,\n    TerminologyRelation, VersionConstraint,\n",
    "terminology_index imports",
)
text = replace_once(
    text,
    "impl TerminologyClosure {\n    pub(crate) fn load(\n        lockfile: &Lockfile,\n        cache: &PackageCache,\n    ) -> Result<Self, TerminologyError> {\n        lockfile.verify_cache(cache)?;\n        let mut closure = Self::default();\n\n        for package in &lockfile.packages {\n",
    "impl TerminologyClosure {\n    pub(crate) fn load_for_root(\n        lockfile: &Lockfile,\n        cache: &PackageCache,\n        root: &LockedPackage,\n    ) -> Result<Self, TerminologyError> {\n        let target_core = root_core_family(root)?;\n        Self::load_scoped(lockfile, cache, Some(target_core))\n    }\n\n    #[cfg(test)]\n    pub(crate) fn load(\n        lockfile: &Lockfile,\n        cache: &PackageCache,\n    ) -> Result<Self, TerminologyError> {\n        Self::load_scoped(lockfile, cache, None)\n    }\n\n    fn load_scoped(\n        lockfile: &Lockfile,\n        cache: &PackageCache,\n        target_core: Option<&str>,\n    ) -> Result<Self, TerminologyError> {\n        lockfile.verify_cache(cache)?;\n        let mut closure = Self::default();\n\n        for package in &lockfile.packages {\n            if let Some(target_core) = target_core {\n                if !package_matches_core_family(lockfile, package, target_core)? {\n                    continue;\n                }\n            }\n",
    "terminology_index scoped loader",
)
marker = "fn required_string(\n"
if marker not in text:
    raise SystemExit("terminology_index helper insertion marker missing")
helpers = r'''fn root_core_family(root: &LockedPackage) -> Result<&str, TerminologyError> {
    let cores = root
        .dependencies
        .keys()
        .filter(|name| is_fhir_core_package(name))
        .map(String::as_str)
        .collect::<Vec<_>>();
    match cores.as_slice() {
        [single] => Ok(*single),
        [] => Err(lock_graph_error(
            root,
            "root package does not declare a FHIR core dependency",
        )),
        _ => Err(lock_graph_error(
            root,
            "root package declares more than one FHIR core dependency",
        )),
    }
}

fn package_matches_core_family(
    lockfile: &Lockfile,
    package: &LockedPackage,
    target_core: &str,
) -> Result<bool, TerminologyError> {
    let mut visiting = BTreeSet::new();
    package_matches_core_family_inner(lockfile, package, target_core, &mut visiting)
}

fn package_matches_core_family_inner(
    lockfile: &Lockfile,
    package: &LockedPackage,
    target_core: &str,
    visiting: &mut BTreeSet<String>,
) -> Result<bool, TerminologyError> {
    if package.name == target_core {
        return Ok(true);
    }
    if is_fhir_core_package(&package.name) {
        return Ok(false);
    }
    if package.dependencies.contains_key(target_core) {
        return Ok(true);
    }
    if package
        .dependencies
        .keys()
        .any(|name| is_fhir_core_package(name))
    {
        return Ok(false);
    }

    let identity = format!("{}@{}", package.name, package.version);
    if !visiting.insert(identity.clone()) {
        return Err(lock_graph_error(package, "dependency cycle while scoping FHIR core family"));
    }

    for (dependency_name, constraint) in &package.dependencies {
        let dependency = select_locked_dependency(lockfile, package, dependency_name, constraint)?;
        if package_matches_core_family_inner(lockfile, dependency, target_core, visiting)? {
            visiting.remove(&identity);
            return Ok(true);
        }
    }
    visiting.remove(&identity);
    Ok(false)
}

fn select_locked_dependency<'a>(
    lockfile: &'a Lockfile,
    parent: &LockedPackage,
    dependency_name: &str,
    constraint: &str,
) -> Result<&'a LockedPackage, TerminologyError> {
    let raw = format!("{dependency_name}@{constraint}");
    let request = PackageRequest::parse(&raw).map_err(|error| TerminologyError::InvalidField {
        resource: format!("{}@{}", parent.name, parent.version),
        field: "lockfile".to_owned(),
        message: format!("invalid dependency request {raw}: {error}"),
    })?;

    let mut matches = Vec::new();
    for candidate in &lockfile.packages {
        if candidate.name != request.name.as_str() {
            continue;
        }
        let candidate_raw = format!("{}@{}", candidate.name, candidate.version);
        let candidate_request = PackageRequest::parse(&candidate_raw).map_err(|error| {
            TerminologyError::InvalidField {
                resource: candidate_raw.clone(),
                field: "lockfile".to_owned(),
                message: format!("invalid locked package identity: {error}"),
            }
        })?;
        let VersionConstraint::Exact(candidate_version) = candidate_request.constraint else {
            return Err(TerminologyError::InvalidField {
                resource: candidate_raw,
                field: "lockfile".to_owned(),
                message: "locked package version is not exact".to_owned(),
            });
        };
        if request.constraint.matches(&candidate_version) {
            matches.push(candidate);
        }
    }

    match matches.as_slice() {
        [single] => Ok(*single),
        [] => Err(lock_graph_error(
            parent,
            &format!("dependency request {raw} has no matching locked package"),
        )),
        _ => Err(lock_graph_error(
            parent,
            &format!(
                "dependency request {raw} matches {} locked packages",
                matches.len()
            ),
        )),
    }
}

fn is_fhir_core_package(name: &str) -> bool {
    name.starts_with("hl7.fhir.r") && name.ends_with(".core")
}

fn lock_graph_error(package: &LockedPackage, message: &str) -> TerminologyError {
    TerminologyError::InvalidField {
        resource: format!("{}@{}", package.name, package.version),
        field: "lockfile".to_owned(),
        message: message.to_owned(),
    }
}

'''
text = text.replace(marker, helpers + marker, 1)

# Add a pure lock-graph regression without changing archive fixtures.
test_marker = "    #[test]\n    fn canonical_reference_parser_is_exact_and_fail_closed() {\n"
if test_marker not in text:
    raise SystemExit("terminology_index test marker missing")
new_test = r'''    fn locked(name: &str, version: &str, dependencies: &[(&str, &str)]) -> LockedPackage {
        LockedPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            sha256: format!("{name}-{version}"),
            source: format!("https://example.test/{name}/{version}"),
            dependencies: dependencies
                .iter()
                .map(|(name, version)| ((*name).to_owned(), (*version).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn fhir_core_family_scope_excludes_cross_version_dependency_branch() {
        let root = locked(
            "example.root",
            "1.0.0",
            &[("hl7.fhir.r4.core", "4.0.1"), ("example.mixed", "1.0.0")],
        );
        let mixed = locked(
            "example.mixed",
            "1.0.0",
            &[("hl7.fhir.r4.core", "4.0.1"), ("example.r5", "1.0.0")],
        );
        let r5 = locked("example.r5", "1.0.0", &[("hl7.fhir.r5.core", "5.0.0")]);
        let lockfile = Lockfile::new(
            vec!["example.root@1.0.0".to_owned()],
            vec![
                root.clone(),
                mixed.clone(),
                r5.clone(),
                locked("hl7.fhir.r4.core", "4.0.1", &[]),
                locked("hl7.fhir.r5.core", "5.0.0", &[]),
            ],
        );

        assert_eq!(root_core_family(&root).unwrap(), "hl7.fhir.r4.core");
        assert!(package_matches_core_family(&lockfile, &mixed, "hl7.fhir.r4.core").unwrap());
        assert!(!package_matches_core_family(&lockfile, &r5, "hl7.fhir.r4.core").unwrap());
        let r5_core = lockfile
            .packages
            .iter()
            .find(|package| package.name == "hl7.fhir.r5.core")
            .unwrap();
        assert!(!package_matches_core_family(&lockfile, r5_core, "hl7.fhir.r4.core").unwrap());
    }

'''
text = text.replace(test_marker, new_test + test_marker, 1)
path.write_text(text)

# ---------------------------------------------------------------------------
# CF-06: additional context is only the root's direct declared dependencies.
# Transitive packages remain verified in the lock but are not bulk-loaded into
# the comparison context, preventing unrelated duplicate resources from
# poisoning profile comparison.
# ---------------------------------------------------------------------------
path = Path("crates/commandf-cli/src/oracle.rs")
text = path.read_text()
text = replace_once(text, "use std::collections::BTreeSet;\n", "", "oracle remove BTreeSet import")
text = regex_once(
    text,
    r"fn dependency_context_packages<'a>\(.*?\n\}\n\nfn select_matching_locked_package<'a>",
    '''fn dependency_context_packages<'a>(\n    lockfile: &'a Lockfile,\n    root: &'a LockedPackage,\n) -> Result<Vec<&'a LockedPackage>, io::Error> {\n    let mut packages = Vec::new();\n    for (dependency_name, constraint) in &root.dependencies {\n        let request = parse_package_request(&format!("{dependency_name}@{constraint}"))?;\n        let dependency = select_matching_locked_package(lockfile, &request, "dependency")?;\n        if dependency.name == ORACLE_CORE_PACKAGE && dependency.version == ORACLE_CORE_VERSION {\n            continue;\n        }\n        packages.push(dependency);\n    }\n    Ok(packages)\n}\n\nfn select_matching_locked_package<'a>''',
    "oracle direct dependency contexts",
)
text = replace_once(
    text,
    "fn package_identity(package: &LockedPackage) -> String {\n    format!(\"{}@{}\", package.name, package.version)\n}\n",
    "#[cfg(test)]\nfn package_identity(package: &LockedPackage) -> String {\n    format!(\"{}@{}\", package.name, package.version)\n}\n",
    "oracle package_identity test-only",
)
text = replace_once(
    text,
    "    fn dependency_context_is_deterministic_leaf_first_and_excludes_core() {\n",
    "    fn dependency_context_is_direct_deterministic_and_excludes_core() {\n",
    "oracle context test name",
)
text = replace_once(
    text,
    "            vec![\"example.b@2.0.3\", \"example.a@1.0.0\"]\n",
    "            vec![\"example.a@1.0.0\"]\n",
    "oracle direct context expected",
)
path.write_text(text)

# The pinned HL7 loader exposes getTypes(); use it to restrict additional
# dependency packages to StructureDefinitions. Core and the compared root
# package remain fully loaded.
path = Path("tools/hl7-oracle/src/main/java/dev/commandf/oracle/Main.java")
text = path.read_text()
text = replace_once(
    text,
    "import java.util.Objects;\nimport java.util.TreeSet;\n",
    "import java.util.Objects;\nimport java.util.Set;\nimport java.util.TreeSet;\n",
    "oracle java import Set",
)
text = replace_once(
    text,
    "      IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());\n      context.loadFromPackage(dependency, dependencyLoader, false);\n",
    "      IContextResourceLoader dependencyLoader = ValidatorUtils.loaderForVersion(dependency.fhirVersion());\n      dependencyLoader.getTypes().retainAll(Set.of(\"StructureDefinition\"));\n      context.loadFromPackage(dependency, dependencyLoader, false);\n",
    "oracle java context type filter",
)
path.write_text(text)

print("CF-10 context-scope fixes applied")
