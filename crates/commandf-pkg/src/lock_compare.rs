//! Bounded, offline CF11 lockfile attribution; never print transport sources.
//! Compares full validated v2 package identities, raw SHA-256, declarations,
//! roots, and resolved edges. This is not an archive or cache verification.

use std::collections::{BTreeMap, BTreeSet};

use crate::{LockedPackage, Lockfile, PackageError};
use serde_json::{json, Value};

const MAX_REPORTED_PACKAGE_DIFFERENCES: usize = 32;

/// Compare validated lockfile-v2 semantic identities and produce source-redacted JSON.
pub fn compare_cf11_lockfiles(
    first: &Lockfile,
    second: &Lockfile,
) -> Result<(Vec<u8>, bool), PackageError> {
    if first.schema != Lockfile::SCHEMA_V2 || second.schema != Lockfile::SCHEMA_V2 {
        return Err(PackageError::InvalidLockfile(
            "CF11 lock comparison requires two schema-v2 lockfiles".to_owned(),
        ));
    }

    // Also validate programmatically built inputs, not only persisted lockfiles.
    first.validate_v2()?;
    second.validate_v2()?;
    let first_by_identity = index_packages(first);
    let second_by_identity = index_packages(second);
    let identities: BTreeSet<_> = first_by_identity
        .keys()
        .chain(second_by_identity.keys())
        .copied()
        .collect();

    let roots_equal = first.roots == second.roots;
    let resolved_edges_equal = first.resolved_dependencies == second.resolved_dependencies;
    let mut package_difference_count: usize = 0;
    let mut package_differences: Vec<Value> = Vec::new();

    for (name, version) in identities {
        let left = first_by_identity.get(&(name, version));
        let right = second_by_identity.get(&(name, version));
        if let (Some(left), Some(right)) = (left, right) {
            if left.sha256 == right.sha256 && left.dependencies == right.dependencies {
                // Transport source URLs are intentionally excluded.
                continue;
            }
        }
        package_difference_count += 1;
        if package_differences.len() < MAX_REPORTED_PACKAGE_DIFFERENCES {
            package_differences.push(json!({
                "name": name,
                "version": version,
                "presence": match (left, right) {
                    (Some(_), Some(_)) => "both",
                    (Some(_), None) => "first_only",
                    (None, Some(_)) => "second_only",
                    (None, None) => unreachable!("set union"),
                },
                "sha256_first": left.map(|entry| entry.sha256.as_str()),
                "sha256_second": right.map(|entry| entry.sha256.as_str()),
                "declared_dependencies_equal": match (left, right) {
                    (Some(left), Some(right)) => Some(left.dependencies == right.dependencies),
                    _ => None,
                },
            }));
        }
    }
    let equivalent = roots_equal && resolved_edges_equal && package_difference_count == 0;
    let report = json!({
        "schema": "commandf.cf11-lock-comparison.v1",
        "equivalent": equivalent,
        "roots_equal": roots_equal,
        "resolved_edges_equal": resolved_edges_equal,
        "package_count_first": first.packages.len(),
        "package_count_second": second.packages.len(),
        "package_difference_count": package_difference_count,
        "package_differences_truncated": package_difference_count > MAX_REPORTED_PACKAGE_DIFFERENCES,
        "package_differences": package_differences,
    });
    let mut bytes = serde_json::to_vec_pretty(&report)?;
    bytes.push(10_u8);
    Ok((bytes, equivalent))
}

fn index_packages(lock: &Lockfile) -> BTreeMap<(&str, &str), &LockedPackage> {
    lock.packages
        .iter()
        .map(|package| ((package.name.as_str(), package.version.as_str()), package))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolvedDependency;

    fn pkg(name: &str, digest: &str, source: &str) -> LockedPackage {
        LockedPackage {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest.to_owned(),
            source: source.to_owned(),
            dependencies: BTreeMap::new(),
        }
    }

    fn lock(digest: &str, source: &str) -> Lockfile {
        Lockfile::new_v2(
            vec!["acme.root@1.0.0".to_owned()],
            vec![pkg("acme.root", digest, source)],
            Vec::new(),
        )
    }

    fn report(first: &Lockfile, second: &Lockfile) -> (Value, bool) {
        let (bytes, equivalent) = compare_cf11_lockfiles(first, second).unwrap();
        assert_eq!(bytes.last(), Some(&10_u8));
        (serde_json::from_slice(&bytes).unwrap(), equivalent)
    }

    #[test]
    fn source_only_drift_is_not_a_semantic_lock_conflict_and_is_not_emitted() {
        let first = lock(&"a".repeat(64), "https://secret.example.test/token=private");
        let second = lock(&"a".repeat(64), "https://other.example.test/secret");
        let (bytes, equivalent) = compare_cf11_lockfiles(&first, &second).unwrap();
        assert!(equivalent);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("\"equivalent\": true"));
        assert!(!text.contains("secret.example"));
        assert!(!text.contains("other.example"));
        assert!(!text.contains("private"));
    }

    #[test]
    fn divergent_raw_digests_are_reported_with_exact_package_identity() {
        let first = lock(&"a".repeat(64), "primary");
        let second = lock(&"b".repeat(64), "secondary");
        let (v, equivalent) = report(&first, &second);
        assert!(!equivalent);
        assert_eq!(v["package_difference_count"], 1);
        assert_eq!(v["package_differences"][0]["name"], "acme.root");
        assert_eq!(v["package_differences"][0]["version"], "1.0.0");
        assert_eq!(v["package_differences"][0]["sha256_first"], "a".repeat(64));
        assert_eq!(v["package_differences"][0]["sha256_second"], "b".repeat(64));
        assert_eq!(
            v["package_differences"][0]["declared_dependencies_equal"],
            true
        );
        assert_eq!(v["resolved_edges_equal"], true);
    }

    #[test]
    fn declared_dependency_and_resolved_edge_drift_is_not_hidden() {
        let digest = "a".repeat(64);
        let first = Lockfile::new_v2(
            vec!["acme.root@1.0.0".to_owned()],
            vec![
                pkg("acme.child", &digest, "one"),
                pkg("acme.root", &digest, "one"),
            ],
            Vec::new(),
        );
        let mut root_with_dependency = pkg("acme.root", &digest, "two");
        root_with_dependency
            .dependencies
            .insert("acme.child".to_owned(), "1.0.0".to_owned());
        let second = Lockfile::new_v2(
            vec!["acme.root@1.0.0".to_owned()],
            vec![pkg("acme.child", &digest, "two"), root_with_dependency],
            vec![ResolvedDependency {
                from_name: "acme.root".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "acme.child".to_owned(),
                to_version: "1.0.0".to_owned(),
                declared_constraint: "1.0.0".to_owned(),
            }],
        );
        let (v, equivalent) = report(&first, &second);
        assert!(!equivalent);
        assert_eq!(v["resolved_edges_equal"], false);
        assert_eq!(v["package_difference_count"], 1);
        assert_eq!(
            v["package_differences"][0]["declared_dependencies_equal"],
            false
        );
    }

    #[test]
    fn missing_package_and_roots_are_reported_without_source() {
        let first = lock(&"a".repeat(64), "private-source");
        let second = Lockfile::new_v2(Vec::new(), Vec::new(), Vec::new());
        let (v, equivalent) = report(&first, &second);
        assert!(!equivalent);
        assert_eq!(v["roots_equal"], false);
        assert_eq!(v["package_differences"][0]["presence"], "first_only");
        assert!(v["package_differences"][0]["sha256_second"].is_null());
    }

    #[test]
    fn many_discrepancies_are_sorted_bounded_and_counted_without_elision_of_result() {
        let first = Lockfile::new_v2(
            Vec::new(),
            (0..35)
                .map(|i| pkg(&format!("acme.{i:03}"), &"a".repeat(64), "private"))
                .collect(),
            Vec::new(),
        );
        let second = Lockfile::new_v2(Vec::new(), Vec::new(), Vec::new());
        let (v, equivalent) = report(&first, &second);
        assert!(!equivalent);
        assert_eq!(v["package_difference_count"], 35);
        assert_eq!(v["package_differences"].as_array().unwrap().len(), 32);
        assert_eq!(v["package_differences_truncated"], true);
        assert_eq!(v["package_differences"][0]["name"], "acme.000");
        assert_eq!(v["package_differences"][31]["name"], "acme.031");
    }

    #[test]
    fn invalid_or_unsupported_lock_is_rejected_before_comparison() {
        let first = Lockfile::new(Vec::new(), Vec::new());
        let second = lock(&"a".repeat(64), "one");
        assert!(matches!(
            compare_cf11_lockfiles(&first, &second),
            Err(PackageError::InvalidLockfile(_))
        ));
        let mut invalid = second.clone();
        invalid.packages.push(invalid.packages[0].clone());
        assert!(matches!(
            compare_cf11_lockfiles(&invalid, &second),
            Err(PackageError::InvalidLockfile(_))
        ));
    }
}
