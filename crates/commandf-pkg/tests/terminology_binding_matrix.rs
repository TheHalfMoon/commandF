use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{
    build_terminology_diff_report, classify_structural_diff, diff_package_archives,
    CompatibilityDirection, CompatibilitySeverity, LockedPackage, Lockfile, PackageCache,
    TerminologyDiffReport, TerminologyIndeterminateReason, TerminologyPackageState,
    TerminologyRelation,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use tar::{Builder, Header};
use tempfile::tempdir;

fn archive(name: &str, version: &str, resources: &[(&str, Value)]) -> Vec<u8> {
    let mut entries = vec![(
        "package/package.json".to_owned(),
        serde_json::to_vec(&json!({
            "name": name,
            "version": version,
            "dependencies": if name == "example.root" {
                json!({"example.term": if version == "1.0.0" { "1.0.0" } else { "2.0.0" }})
            } else {
                json!({})
            },
        }))
        .unwrap(),
    )];
    for (filename, resource) in resources {
        entries.push((
            format!("package/{filename}"),
            serde_json::to_vec(resource).unwrap(),
        ));
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for (path, body) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).unwrap();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, Cursor::new(body)).unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn profile(strength: &str, value_set: &str) -> Value {
    json!({
        "resourceType": "StructureDefinition",
        "id": "test-profile",
        "url": "http://example.org/StructureDefinition/test-profile",
        "version": "1",
        "name": "TestProfile",
        "status": "active",
        "kind": "resource",
        "abstract": false,
        "type": "Patient",
        "baseDefinition": "http://hl7.org/fhir/StructureDefinition/Patient",
        "derivation": "constraint",
        "snapshot": {
            "element": [
                {"id": "Patient", "path": "Patient", "min": 0, "max": "1"},
                {
                    "id": "Patient.gender",
                    "path": "Patient.gender",
                    "min": 0,
                    "max": "1",
                    "type": [{"code": "code"}],
                    "binding": {"strength": strength, "valueSet": value_set}
                }
            ]
        }
    })
}

fn value_set(id: &str, url: &str, version: &str, codes: &[&str]) -> Value {
    json!({
        "resourceType": "ValueSet",
        "id": id,
        "url": url,
        "version": version,
        "name": id,
        "status": "active",
        "expansion": {
            "total": codes.len(),
            "contains": codes.iter().map(|code| json!({
                "system": "http://example.org/CodeSystem/gender",
                "version": "1",
                "code": code
            })).collect::<Vec<_>>()
        }
    })
}

fn locked(name: &str, version: &str, sha256: String) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256,
        source: "fixture".to_owned(),
        dependencies: if name == "example.root" {
            BTreeMap::from([(
                "example.term".to_owned(),
                if version == "1.0.0" { "1.0.0" } else { "2.0.0" }.to_owned(),
            )])
        } else {
            BTreeMap::new()
        },
    }
}

fn report_case(
    before_strength: &str,
    before_reference: &str,
    before_value_sets: &[(&str, Value)],
    after_strength: &str,
    after_reference: &str,
    after_value_sets: &[(&str, Value)],
) -> Result<TerminologyDiffReport, String> {
    let before_dir = tempdir().unwrap();
    let after_dir = tempdir().unwrap();
    let before_cache = PackageCache::new(before_dir.path());
    let after_cache = PackageCache::new(after_dir.path());

    let before_root = archive(
        "example.root",
        "1.0.0",
        &[(
            "StructureDefinition-test-profile.json",
            profile(before_strength, before_reference),
        )],
    );
    let after_root = archive(
        "example.root",
        "2.0.0",
        &[(
            "StructureDefinition-test-profile.json",
            profile(after_strength, after_reference),
        )],
    );
    let before_term = archive("example.term", "1.0.0", before_value_sets);
    let after_term = archive("example.term", "2.0.0", after_value_sets);

    let before_root_digest = before_cache.put(&before_root).unwrap();
    let after_root_digest = after_cache.put(&after_root).unwrap();
    let before_term_digest = before_cache.put(&before_term).unwrap();
    let after_term_digest = after_cache.put(&after_term).unwrap();

    let before_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked("example.root", "1.0.0", before_root_digest.clone()),
            locked("example.term", "1.0.0", before_term_digest),
        ],
    );
    let after_lock = Lockfile::new(
        vec!["example.root@2.0.0".to_owned()],
        vec![
            locked("example.root", "2.0.0", after_root_digest.clone()),
            locked("example.term", "2.0.0", after_term_digest),
        ],
    );

    let structural = diff_package_archives(
        "example.root",
        "1.0.0",
        &before_root_digest,
        &before_root,
        "2.0.0",
        &after_root_digest,
        &after_root,
    )
    .unwrap();
    let compatibility = classify_structural_diff(&structural).unwrap();
    build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: &before_lock,
            cache: &before_cache,
            root_bytes: &before_root,
        },
        TerminologyPackageState {
            lockfile: &after_lock,
            cache: &after_cache,
            root_bytes: &after_root,
        },
        &structural,
        &compatibility,
    )
    .map_err(|error| error.to_string())
}

#[test]
fn required_widening_is_consumer_breaking() {
    let url = "http://example.org/ValueSet/gender";
    let before = value_set("gender", url, "1", &["a"]);
    let after = value_set("gender", url, "2", &["a", "b"]);
    let report = report_case(
        "required",
        url,
        &[("ValueSet-gender.json", before)],
        "required",
        url,
        &[("ValueSet-gender.json", after)],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Widened);
    assert_eq!(finding.rule_id.as_deref(), Some("CF07-BIND-002"));
    assert_eq!(finding.severity, Some(CompatibilitySeverity::Breaking));
    assert_eq!(finding.direction, Some(CompatibilityDirection::Consumer));
}

#[test]
fn required_incomparable_membership_breaks_both_directions() {
    let url = "http://example.org/ValueSet/gender";
    let before = value_set("gender", url, "1", &["a", "b"]);
    let after = value_set("gender", url, "2", &["a", "c"]);
    let report = report_case(
        "required",
        url,
        &[("ValueSet-gender.json", before)],
        "required",
        url,
        &[("ValueSet-gender.json", after)],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 2);
    assert_eq!(
        report.binding_refinements[0].relation,
        TerminologyRelation::Incomparable
    );
    assert_eq!(
        report.binding_refinements[1].relation,
        TerminologyRelation::Incomparable
    );
    let directions = report
        .binding_refinements
        .iter()
        .map(|finding| finding.direction)
        .collect::<Vec<_>>();
    assert!(directions.contains(&Some(CompatibilityDirection::Producer)));
    assert!(directions.contains(&Some(CompatibilityDirection::Consumer)));
    let rules = report
        .binding_refinements
        .iter()
        .filter_map(|finding| finding.rule_id.as_deref())
        .collect::<Vec<_>>();
    assert!(rules.contains(&"CF07-BIND-003"));
    assert!(rules.contains(&"CF07-BIND-004"));
}

#[test]
fn extensible_membership_change_is_evidence_not_hard_breaking() {
    let url = "http://example.org/ValueSet/gender";
    let before = value_set("gender", url, "1", &["a", "b"]);
    let after = value_set("gender", url, "2", &["a"]);
    let report = report_case(
        "extensible",
        url,
        &[("ValueSet-gender.json", before)],
        "extensible",
        url,
        &[("ValueSet-gender.json", after)],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Narrowed);
    assert_eq!(finding.rule_id, None);
    assert_eq!(finding.severity, None);
    assert_eq!(finding.direction, None);
}

#[test]
fn equal_membership_after_reference_change_does_not_invent_safe_or_breaking() {
    let before_url = "http://example.org/ValueSet/old";
    let after_url = "http://example.org/ValueSet/new";
    let before = value_set("old", before_url, "1", &["a", "b"]);
    let after = value_set("new", after_url, "1", &["a", "b"]);
    let report = report_case(
        "required",
        before_url,
        &[("ValueSet-old.json", before)],
        "required",
        after_url,
        &[("ValueSet-new.json", after)],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Equal);
    assert_eq!(finding.rule_id, None);
    assert_eq!(finding.severity, None);
    assert_eq!(finding.direction, None);
    assert!(report
        .compatibility
        .findings
        .iter()
        .any(|finding| finding.rule_id == "CF04-BIND-005"));
}

#[test]
fn strength_change_blocks_hard_membership_refinement() {
    let url = "http://example.org/ValueSet/gender";
    let before = value_set("gender", url, "1", &["a", "b"]);
    let after = value_set("gender", url, "2", &["a"]);
    let report = report_case(
        "required",
        url,
        &[("ValueSet-gender.json", before)],
        "extensible",
        url,
        &[("ValueSet-gender.json", after)],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Narrowed);
    assert_eq!(
        finding.reason,
        Some(TerminologyIndeterminateReason::UnsupportedBindingInteraction)
    );
    assert_eq!(finding.rule_id, None);
    assert_eq!(finding.severity, None);
    assert_eq!(finding.direction, None);
}

#[test]
fn changed_unresolved_reference_is_indeterminate() {
    let report = report_case(
        "required",
        "http://example.org/ValueSet/missing-before",
        &[],
        "required",
        "http://example.org/ValueSet/missing-after",
        &[],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Indeterminate);
    assert_eq!(
        finding.reason,
        Some(TerminologyIndeterminateReason::UnresolvedValueSet)
    );
    assert_eq!(finding.rule_id, None);
}

#[test]
fn ambiguous_bare_value_set_reference_is_indeterminate_not_hard_proof() {
    let url = "http://example.org/ValueSet/gender";
    let before_one = value_set("gender-1", url, "1", &["a"]);
    let before_two = value_set("gender-2", url, "2", &["a"]);
    let after_one = value_set("gender-1", url, "1", &["a"]);
    let after_two = value_set("gender-2", url, "2", &["a"]);
    let report = report_case(
        "required",
        url,
        &[
            ("ValueSet-gender-1.json", before_one),
            ("ValueSet-gender-2.json", before_two),
        ],
        "required",
        url,
        &[
            ("ValueSet-gender-1.json", after_one),
            ("ValueSet-gender-2.json", after_two),
        ],
    )
    .unwrap();
    assert_eq!(report.binding_refinements.len(), 1);
    let finding = &report.binding_refinements[0];
    assert_eq!(finding.relation, TerminologyRelation::Indeterminate);
    assert_eq!(
        finding.reason,
        Some(TerminologyIndeterminateReason::AmbiguousCanonical)
    );
    assert!(!finding.binding_proof_eligible);
    assert!(finding.proof_mode.is_none());
    assert!(finding.rule_id.is_none());
    assert!(finding.severity.is_none());
    assert!(finding.direction.is_none());
}
