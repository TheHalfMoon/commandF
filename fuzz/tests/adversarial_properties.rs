use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use commandf_pkg::{
    build_context_graph, build_source_mapped_check_report, evaluate_compatibility_policy,
    evaluate_quality_gate, finding_fingerprint_v1, validate_quality_gate_report,
    CanonicalReferenceRelation, CanonicalResolutionStatus, CheckDirection, CheckFailOn,
    CheckPolicy, CompatibilityDirection, CompatibilityFinding, CompatibilityReport,
    CompatibilitySeverity, ElementView, FindingFingerprint, GateSuppression, GateSuppressions,
    LockedPackage, Lockfile, PackageCache, PackageEvidence, QualityGateDisposition,
    ResolvedDependency, ResourceKey, ResourceKeyKind, StructuralChangeKind,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use proptest::prelude::*;

#[path = "support/property_runner.rs"]
mod property_runner;
use serde_json::json;
use sha2::{Digest, Sha256};
use tar::{Builder, Header};

#[path = "../../tests/assurance/af02_models/canonical_reference.rs"]
mod canonical_reference_model;
#[path = "../../tests/assurance/af02_models/context_graph_order.rs"]
mod context_graph_order_model;
#[path = "../../tests/assurance/af02_models/gate_truth_table.rs"]
mod gate_truth_table_model;
#[path = "../../tests/assurance/af02_models/portable_path.rs"]
mod portable_path_model;

const CANONICAL_REFERENCE_PROPERTY: &str = "PROP-CANONICAL-REFERENCE-001";
const CONTEXT_GRAPH_PROPERTY: &str = "PROP-CONTEXT-GRAPH-ORDER-001";
const GATE_PROPERTY: &str = "PROP-GATE-FINGERPRINT-SUPPRESSION-001";
const PORTABLE_PATH_PROPERTY: &str = "PROP-PORTABLE-PATH-001";
const CANONICAL_REFERENCE_SEED_HEX: &str =
    "460617d7c267dbea82a6571424601e1c12d468d07b55842e01cd967e9ca57f84";
const CONTEXT_GRAPH_SEED_HEX: &str =
    "8961a39186ec93f00f2757102a78d6ba457bc2837c132ad551e96f5970cc2e4a";
const GATE_SEED_HEX: &str = "7aee90d41e14cd0d4f509e11eadace4ce6823f4c214246761a1f89752dd4a48a";
const PORTABLE_PATH_SEED_HEX: &str =
    "c15f7011fce0482f9839861a576d9c108142c08bc95d4bfb444981c25b874c3f";
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(label: &str) -> PathBuf {
    let counter = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "commandf-af02-t033-{label}-{}-{counter}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create T033 scratch directory");
    path
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn append_entry(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, body: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_path(path).expect("set archive path");
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append(&header, Cursor::new(body))
        .expect("append archive entry");
}

fn package_archive(name: &str, version: &str, resources: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let manifest = format!("{{\"name\":\"{name}\",\"version\":\"{version}\"}}");
        append_entry(&mut builder, "package/package.json", manifest.as_bytes());
        for (path, body) in resources {
            append_entry(&mut builder, path, body);
        }
        builder.finish().expect("finish tar archive");
    }
    encoder.finish().expect("finish gzip archive")
}

fn locked_package(name: &str, version: &str, digest: &str) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: digest.to_owned(),
        source: "af02:model".to_owned(),
        dependencies: BTreeMap::new(),
    }
}

fn check_finding(rule_id: &str, after_filename: Option<&str>) -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: rule_id.to_owned(),
        severity: CompatibilitySeverity::Breaking,
        direction: CompatibilityDirection::Producer,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: format!("{rule_id} synthetic finding"),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "https://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: after_filename.map(str::to_owned),
        view: Some(ElementView::Snapshot),
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: None,
        after: None,
    }
}

fn check_report(
    findings: Vec<CompatibilityFinding>,
    policy: CheckPolicy,
) -> commandf_pkg::CheckReport {
    let compatibility = CompatibilityReport {
        schema: CompatibilityReport::SCHEMA_V1,
        ruleset: CompatibilityReport::RULESET_V1.to_owned(),
        package_name: "example.package".to_owned(),
        before: PackageEvidence {
            version: "1.0.0".to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: "1.1.0".to_owned(),
            archive_sha256: "b".repeat(64),
        },
        findings,
    };
    evaluate_compatibility_policy(&compatibility, policy).expect("valid generated check report")
}

fn product_portable_path_accepts(value: &str) -> bool {
    let root = scratch("path");
    let report = check_report(
        vec![check_finding(
            "CF04-AF02-PATH",
            Some("StructureDefinition-example.json"),
        )],
        CheckPolicy::default(),
    );

    let model = portable_path_model::normalize(value, false);
    if let Ok(normalized) = &model {
        let file = root.join(normalized);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).expect("create generated source parent");
        }
        fs::write(&file, "Profile: Example\nParent: Observation\n")
            .expect("write generated source");
    }

    let index = serde_json::to_vec(&json!([{
        "outputFile": "StructureDefinition-example.json",
        "fshFile": value,
        "fshName": "Example",
        "fshType": "Profile",
        "startLine": 1,
        "endLine": 2
    }]))
    .expect("serialize generated SUSHI index");
    let accepted = build_source_mapped_check_report(&report, &index, &root, Path::new(".")).is_ok();
    cleanup(&root);
    accepted
}

fn model_gate_finding(
    severity: gate_truth_table_model::Severity,
    direction: gate_truth_table_model::Direction,
    before: serde_json::Value,
    after: serde_json::Value,
) -> gate_truth_table_model::Finding {
    gate_truth_table_model::Finding {
        rule_id: "CF04-AF02-GATE".to_owned(),
        severity,
        direction,
        resource_value: "https://example.org/StructureDefinition/example".to_owned(),
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: Some(before),
        after: Some(after),
    }
}

fn product_gate_finding(model: &gate_truth_table_model::Finding) -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: model.rule_id.clone(),
        severity: match model.severity {
            gate_truth_table_model::Severity::Breaking => CompatibilitySeverity::Breaking,
            gate_truth_table_model::Severity::Risky => CompatibilitySeverity::Risky,
            gate_truth_table_model::Severity::Additive => CompatibilitySeverity::Additive,
        },
        direction: match model.direction {
            gate_truth_table_model::Direction::Producer => CompatibilityDirection::Producer,
            gate_truth_table_model::Direction::Consumer => CompatibilityDirection::Consumer,
        },
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: "presentation-only message".to_owned(),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: model.resource_value.clone(),
        },
        before_filename: model.before_filename.clone(),
        after_filename: model.after_filename.clone(),
        view: Some(ElementView::Snapshot),
        element_id: model.element_id.clone(),
        field: model.field.clone(),
        before: model.before.clone(),
        after: model.after.clone(),
    }
}

fn selected_direction(value: u8) -> (gate_truth_table_model::SelectedDirection, CheckDirection) {
    match value % 3 {
        0 => (
            gate_truth_table_model::SelectedDirection::Both,
            CheckDirection::Both,
        ),
        1 => (
            gate_truth_table_model::SelectedDirection::Producer,
            CheckDirection::Producer,
        ),
        _ => (
            gate_truth_table_model::SelectedDirection::Consumer,
            CheckDirection::Consumer,
        ),
    }
}

fn fail_on(value: u8) -> (gate_truth_table_model::FailOn, CheckFailOn) {
    match value % 3 {
        0 => (
            gate_truth_table_model::FailOn::Breaking,
            CheckFailOn::Breaking,
        ),
        1 => (gate_truth_table_model::FailOn::Risky, CheckFailOn::Risky),
        _ => (gate_truth_table_model::FailOn::None, CheckFailOn::None),
    }
}

fn severity(value: u8) -> gate_truth_table_model::Severity {
    match value % 3 {
        0 => gate_truth_table_model::Severity::Breaking,
        1 => gate_truth_table_model::Severity::Risky,
        _ => gate_truth_table_model::Severity::Additive,
    }
}

fn direction(value: bool) -> gate_truth_table_model::Direction {
    if value {
        gate_truth_table_model::Direction::Producer
    } else {
        gate_truth_table_model::Direction::Consumer
    }
}

fn product_canonical_resolution(
    canonical: &str,
    candidates: &[canonical_reference_model::Candidate],
) -> Result<canonical_reference_model::Resolution, String> {
    let root = scratch("canonical-reference");
    let source_resource = serde_json::to_vec(&json!({
        "resourceType": "ValueSet",
        "id": "source",
        "url": "https://example.org/ValueSet/source",
        "version": "1.0.0",
        "compose": {
            "include": [{
                "valueSet": [canonical]
            }]
        }
    }))
    .expect("serialize source ValueSet");

    let source_archive = package_archive(
        "af02.source",
        "1.0.0",
        &[("package/ValueSet-source.json".to_owned(), source_resource)],
    );
    let target_resources = candidates
        .iter()
        .map(|candidate| {
            let mut value = json!({
                "resourceType": "ValueSet",
                "id": candidate.identity,
                "url": candidate.url,
            });
            if let Some(version) = &candidate.version {
                value
                    .as_object_mut()
                    .expect("resource object")
                    .insert("version".to_owned(), json!(version));
            }
            (
                format!("package/ValueSet-{}.json", candidate.identity),
                serde_json::to_vec(&value).expect("serialize candidate ValueSet"),
            )
        })
        .collect::<Vec<_>>();
    let target_archive = package_archive("af02.targets", "1.0.0", &target_resources);

    let cache = PackageCache::new(&root);
    let source_digest = cache
        .put(&source_archive)
        .map_err(|error| error.to_string())?;
    let target_digest = cache
        .put(&target_archive)
        .map_err(|error| error.to_string())?;
    let lock = Lockfile::new_v2(
        vec![
            "af02.source@1.0.0".to_owned(),
            "af02.targets@1.0.0".to_owned(),
        ],
        vec![
            locked_package("af02.source", "1.0.0", &source_digest),
            locked_package("af02.targets", "1.0.0", &target_digest),
        ],
        Vec::new(),
    );

    let result = build_context_graph(&lock, &cache);
    cleanup(&root);
    let report = result.map_err(|error| error.to_string())?;
    let edge = report
        .canonical_reference_edges
        .iter()
        .find(|edge| edge.relation == CanonicalReferenceRelation::ValueSetIncludeValueSet)
        .ok_or_else(|| "generated canonical edge missing".to_owned())?;
    let mut identities = edge
        .candidates
        .iter()
        .map(|candidate| {
            candidate
                .filename
                .strip_prefix("ValueSet-")
                .and_then(|value| value.strip_suffix(".json"))
                .unwrap_or(&candidate.filename)
                .to_owned()
        })
        .collect::<Vec<_>>();
    identities.sort();
    identities.dedup();

    Ok(match edge.resolution {
        CanonicalResolutionStatus::External => canonical_reference_model::Resolution::External,
        CanonicalResolutionStatus::Resolved => {
            canonical_reference_model::Resolution::Resolved(identities[0].clone())
        }
        CanonicalResolutionStatus::Ambiguous => {
            canonical_reference_model::Resolution::Ambiguous(identities)
        }
    })
}

fn graph_fixture(
    reverse_packages: bool,
    reverse_resources: bool,
) -> (
    Lockfile,
    PackageCache,
    PathBuf,
    context_graph_order_model::Graph,
) {
    let root = scratch("graph-order");
    let resource_a = serde_json::to_vec(&json!({
        "resourceType": "StructureDefinition",
        "id": "alpha",
        "url": "https://example.org/StructureDefinition/alpha",
        "version": "1.0.0"
    }))
    .expect("serialize alpha resource");
    let resource_b = serde_json::to_vec(&json!({
        "resourceType": "Patient",
        "id": "patient"
    }))
    .expect("serialize patient resource");

    let mut first_resources = vec![
        (
            "package/StructureDefinition-alpha.json".to_owned(),
            resource_a.clone(),
        ),
        (
            "package/Patient-patient.json".to_owned(),
            resource_b.clone(),
        ),
    ];
    if reverse_resources {
        first_resources.reverse();
    }
    let second_resources = vec![(
        "package/ValueSet-beta.json".to_owned(),
        serde_json::to_vec(&json!({
            "resourceType": "ValueSet",
            "id": "beta",
            "url": "https://example.org/ValueSet/beta",
            "version": "1.0.0"
        }))
        .expect("serialize beta resource"),
    )];

    let first_archive = package_archive("af02.alpha", "1.0.0", &first_resources);
    let second_archive = package_archive("af02.beta", "1.0.0", &second_resources);
    let first_digest = sha256_hex(&first_archive);
    let second_digest = sha256_hex(&second_archive);
    let cache = PackageCache::new(&root);
    assert_eq!(
        cache.put(&first_archive).expect("cache alpha"),
        first_digest
    );
    assert_eq!(
        cache.put(&second_archive).expect("cache beta"),
        second_digest
    );

    let first_package = locked_package("af02.alpha", "1.0.0", &first_digest);
    let second_package = locked_package("af02.beta", "1.0.0", &second_digest);
    let mut packages = vec![first_package, second_package];
    if reverse_packages {
        packages.reverse();
    }
    let lock = Lockfile::new_v2(
        vec!["af02.beta@1.0.0".to_owned(), "af02.alpha@1.0.0".to_owned()],
        packages,
        Vec::<ResolvedDependency>::new(),
    );

    let model_packages = vec![
        context_graph_order_model::PackageNode {
            identity: context_graph_order_model::PackageIdentity {
                name: "af02.alpha".to_owned(),
                version: "1.0.0".to_owned(),
                sha256: first_digest.clone(),
            },
            source: "af02:model".to_owned(),
        },
        context_graph_order_model::PackageNode {
            identity: context_graph_order_model::PackageIdentity {
                name: "af02.beta".to_owned(),
                version: "1.0.0".to_owned(),
                sha256: second_digest.clone(),
            },
            source: "af02:model".to_owned(),
        },
    ];
    let alpha_identity = model_packages[0].identity.clone();
    let beta_identity = model_packages[1].identity.clone();
    let model_artifacts = vec![
        context_graph_order_model::ArtifactNode {
            package: alpha_identity.clone(),
            filename: "Patient-patient.json".to_owned(),
            sha256: sha256_hex(&resource_b),
            resource_type: "Patient".to_owned(),
            id: Some("patient".to_owned()),
            canonical_url: None,
            canonical_version: None,
        },
        context_graph_order_model::ArtifactNode {
            package: alpha_identity,
            filename: "StructureDefinition-alpha.json".to_owned(),
            sha256: sha256_hex(&resource_a),
            resource_type: "StructureDefinition".to_owned(),
            id: Some("alpha".to_owned()),
            canonical_url: Some("https://example.org/StructureDefinition/alpha".to_owned()),
            canonical_version: Some("1.0.0".to_owned()),
        },
        context_graph_order_model::ArtifactNode {
            package: beta_identity,
            filename: "ValueSet-beta.json".to_owned(),
            sha256: sha256_hex(&second_resources[0].1),
            resource_type: "ValueSet".to_owned(),
            id: Some("beta".to_owned()),
            canonical_url: Some("https://example.org/ValueSet/beta".to_owned()),
            canonical_version: Some("1.0.0".to_owned()),
        },
    ];
    let expected = context_graph_order_model::canonicalize(
        vec!["af02.alpha@1.0.0".to_owned(), "af02.beta@1.0.0".to_owned()],
        model_packages,
        model_artifacts,
        Vec::new(),
        Vec::new(),
    );
    (lock, cache, root, expected)
}

fn project_graph(report: &commandf_pkg::ContextGraphReport) -> context_graph_order_model::Graph {
    let packages = report
        .packages
        .iter()
        .map(|package| context_graph_order_model::PackageNode {
            identity: context_graph_order_model::PackageIdentity {
                name: package.identity.name.clone(),
                version: package.identity.version.clone(),
                sha256: package.identity.sha256.clone(),
            },
            source: package.source.clone(),
        })
        .collect();
    let artifacts = report
        .artifacts
        .iter()
        .map(|artifact| context_graph_order_model::ArtifactNode {
            package: context_graph_order_model::PackageIdentity {
                name: artifact.identity.package.name.clone(),
                version: artifact.identity.package.version.clone(),
                sha256: artifact.identity.package.sha256.clone(),
            },
            filename: artifact.identity.filename.clone(),
            sha256: artifact.identity.sha256.clone(),
            resource_type: artifact.resource_type.clone(),
            id: artifact.id.clone(),
            canonical_url: artifact.canonical_url.clone(),
            canonical_version: artifact.canonical_version.clone(),
        })
        .collect();
    let dependency_edges = report
        .package_dependency_edges
        .iter()
        .map(|edge| context_graph_order_model::DependencyEdge {
            from: context_graph_order_model::PackageIdentity {
                name: edge.from.name.clone(),
                version: edge.from.version.clone(),
                sha256: edge.from.sha256.clone(),
            },
            to: context_graph_order_model::PackageIdentity {
                name: edge.to.name.clone(),
                version: edge.to.version.clone(),
                sha256: edge.to.sha256.clone(),
            },
            declared_constraint: edge.declared_constraint.clone(),
        })
        .collect();
    let reference_edges = report
        .canonical_reference_edges
        .iter()
        .map(|edge| context_graph_order_model::ReferenceEdge {
            source: edge.source.filename.clone(),
            relation: format!("{:?}", edge.relation),
            source_path: edge.source_path.clone(),
            canonical: edge.canonical.clone(),
            candidates: edge
                .candidates
                .iter()
                .map(|candidate| candidate.filename.clone())
                .collect(),
        })
        .collect();

    context_graph_order_model::Graph {
        root_requests: report.root_requests.clone(),
        packages,
        artifacts,
        dependency_edges,
        reference_edges,
        unsupported_resource_types: report.coverage.unsupported_source_resource_types.clone(),
    }
}

#[test]
fn af02_portable_path_rejects_every_frozen_invalidity_class() {
    use portable_path_model::Invalidity;

    for invalidity in [
        Invalidity::Empty,
        Invalidity::LeadingSlash,
        Invalidity::UncPrefix,
        Invalidity::DrivePrefix,
        Invalidity::EmptyComponent,
        Invalidity::DotComponent,
        Invalidity::ParentComponent,
        Invalidity::DisallowedSingleDot,
    ] {
        let (value, allow_dot) = portable_path_model::invalid_case(invalidity);
        assert_eq!(
            portable_path_model::normalize(value, allow_dot),
            Err(invalidity)
        );
        assert!(
            !product_portable_path_accepts(value),
            "{PORTABLE_PATH_PROPERTY} unexpectedly accepted {invalidity:?}"
        );
    }
}

#[test]
fn af02_portable_valid_paths_match_independent_model() {
    let strategy = (
        prop::collection::vec("[a-z][a-z0-9]{0,7}", 1..=4),
        any::<bool>(),
    );
    let mut runner = property_runner::runner(PORTABLE_PATH_SEED_HEX);
    runner
        .run(&strategy, |(components, backslashes)| {
            let separator = if backslashes { "\\" } else { "/" };
            let value = components.join(separator);
            prop_assert!(portable_path_model::normalize(&value, false).is_ok());
            prop_assert!(product_portable_path_accepts(&value));
            Ok(())
        })
        .expect("frozen portable-path property must match the independent model");
}

#[test]
fn af02_canonical_reference_covers_every_frozen_invalidity_class() {
    for (canonical, expected) in canonical_reference_model::invalid_cases() {
        assert_eq!(canonical_reference_model::resolve(canonical, &[]), expected);
        assert!(
            product_canonical_resolution(canonical, &[]).is_err(),
            "{CANONICAL_REFERENCE_PROPERTY} unexpectedly accepted {canonical:?}"
        );
    }

    let candidate = canonical_reference_model::Candidate {
        identity: "candidate".to_owned(),
        url: "https://example.org/ValueSet/target".to_owned(),
        version: Some("1.0.0".to_owned()),
    };
    for canonical in [
        "https://example.org/ValueSet/unknown",
        "https://example.org/ValueSet/target|2.0.0",
    ] {
        let expected =
            canonical_reference_model::resolve(canonical, std::slice::from_ref(&candidate));
        assert_eq!(expected, canonical_reference_model::Resolution::External);
        assert_eq!(
            product_canonical_resolution(canonical, std::slice::from_ref(&candidate))
                .expect("external canonical resolution"),
            expected
        );
    }

    let duplicates = vec![candidate.clone(), candidate];
    let canonical = "https://example.org/ValueSet/target|1.0.0";
    let expected = canonical_reference_model::resolve(canonical, &duplicates);
    assert_eq!(
        expected,
        canonical_reference_model::Resolution::Resolved("candidate".to_owned())
    );
    assert!(
        product_canonical_resolution(canonical, &duplicates).is_err(),
        "{CANONICAL_REFERENCE_PROPERTY} duplicate candidate identity must fail closed at the public artifact boundary"
    );
}

#[test]
fn af02_canonical_reference_resolution_matches_independent_model() {
    let strategy = (0usize..=3, any::<bool>(), any::<bool>());
    let mut runner = property_runner::runner(CANONICAL_REFERENCE_SEED_HEX);
    runner
        .run(
            &strategy,
            |(candidate_count, explicit_version, fragment)| {
                let candidates = (0..candidate_count)
                    .map(|index| canonical_reference_model::Candidate {
                        identity: format!("candidate-{index}"),
                        url: "https://example.org/ValueSet/target".to_owned(),
                        version: Some(format!("{}.0.0", index + 1)),
                    })
                    .collect::<Vec<_>>();
                let mut canonical = "https://example.org/ValueSet/target".to_owned();
                if explicit_version {
                    canonical.push_str("|1.0.0");
                }
                if fragment {
                    canonical.push_str("#fragment");
                }

                let expected = canonical_reference_model::resolve(&canonical, &candidates);
                let actual = product_canonical_resolution(&canonical, &candidates)
                    .map_err(TestCaseError::fail)?;
                prop_assert_eq!(actual, expected);
                Ok(())
            },
        )
        .expect("frozen canonical-reference property must match the independent model");
}

#[test]
fn af02_context_graph_order_matches_independent_model() {
    let strategy = (any::<bool>(), any::<bool>());
    let mut runner = property_runner::runner(CONTEXT_GRAPH_SEED_HEX);
    runner
        .run(&strategy, |(reverse_packages, reverse_resources)| {
            let (lock, cache, root, expected) = graph_fixture(reverse_packages, reverse_resources);
            let report = build_context_graph(&lock, &cache)
                .map_err(|error| TestCaseError::fail(error.to_string()))?;
            let actual = project_graph(&report);
            cleanup(&root);
            prop_assert_eq!(actual, expected);
            Ok(())
        })
        .expect("frozen context-graph property must match the independent model");
}

#[test]
fn af02_gate_truth_table_matches_independent_model() {
    let strategy = (
        0u8..3,
        any::<bool>(),
        0u8..3,
        0u8..3,
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
    );
    let mut runner = property_runner::runner(GATE_SEED_HEX);
    runner
        .run(
            &strategy,
            |(
                severity_code,
                producer,
                direction_code,
                fail_code,
                baseline_member,
                suppressed,
                object_order_flip,
            )| {
                let before = if object_order_flip {
                    serde_json::from_str(r#"{"z":1,"a":{"y":2,"x":3}}"#).expect("model JSON")
                } else {
                    serde_json::from_str(r#"{"a":{"x":3,"y":2},"z":1}"#).expect("model JSON")
                };
                let finding = model_gate_finding(
                    severity(severity_code),
                    direction(producer),
                    before,
                    json!({"items":[1,2,3]}),
                );
                let ruleset = CompatibilityReport::RULESET_V1;
                let fingerprint = gate_truth_table_model::fingerprint(ruleset, &finding);
                let baseline = if baseline_member {
                    vec![finding.clone()]
                } else {
                    Vec::new()
                };
                let suppressions = if suppressed {
                    vec![gate_truth_table_model::Suppression {
                        fingerprint: fingerprint.clone(),
                        rationale: "accepted".to_owned(),
                        reference: Some("AF02-T033".to_owned()),
                    }]
                } else {
                    Vec::new()
                };
                let (model_direction, product_direction) = selected_direction(direction_code);
                let (model_fail, product_fail) = fail_on(fail_code);
                let expected = gate_truth_table_model::evaluate(
                    ruleset,
                    std::slice::from_ref(&finding),
                    &baseline,
                    &suppressions,
                    model_direction,
                    model_fail,
                )
                .expect("generated model case must be valid");

                let current_product = check_report(
                    vec![product_gate_finding(&finding)],
                    CheckPolicy {
                        direction: product_direction,
                        fail_on: product_fail,
                    },
                );
                let baseline_product = baseline_member.then(|| {
                    check_report(
                        vec![product_gate_finding(&finding)],
                        CheckPolicy {
                            direction: product_direction,
                            fail_on: product_fail,
                        },
                    )
                });
                let suppression_product = if suppressed {
                    Some(GateSuppressions {
                        schema: GateSuppressions::SCHEMA_V1,
                        suppressions: vec![GateSuppression {
                            finding_fingerprint: FindingFingerprint {
                                schema: FindingFingerprint::SCHEMA_V1,
                                digest: fingerprint.clone(),
                            },
                            rationale: "accepted".to_owned(),
                            reference: Some("AF02-T033".to_owned()),
                        }],
                    })
                } else {
                    None
                };
                let actual = evaluate_quality_gate(
                    &current_product,
                    baseline_product.as_ref(),
                    suppression_product.as_ref(),
                )
                .map_err(|error| TestCaseError::fail(error.to_string()))?;

                prop_assert_eq!(actual.findings.len(), 1);
                prop_assert_eq!(actual.findings[0].fingerprint.digest.clone(), fingerprint);
                prop_assert_eq!(
                    actual.findings[0].disposition,
                    match expected.dispositions[0] {
                        gate_truth_table_model::Disposition::New => QualityGateDisposition::New,
                        gate_truth_table_model::Disposition::Baseline =>
                            QualityGateDisposition::Baseline,
                        gate_truth_table_model::Disposition::Suppressed =>
                            QualityGateDisposition::Suppressed,
                    }
                );
                prop_assert_eq!(
                    actual.decision.selected_findings,
                    expected.selected_findings
                );
                prop_assert_eq!(actual.decision.new_findings, expected.new_findings);
                prop_assert_eq!(
                    actual.decision.baseline_findings,
                    expected.baseline_findings
                );
                prop_assert_eq!(
                    actual.decision.suppressed_findings,
                    expected.suppressed_findings
                );
                prop_assert_eq!(
                    actual.decision.new_selected_breaking_findings,
                    expected.new_selected_breaking_findings
                );
                prop_assert_eq!(
                    actual.decision.new_selected_risky_findings,
                    expected.new_selected_risky_findings
                );
                prop_assert_eq!(
                    actual.decision.new_selected_additive_findings,
                    expected.new_selected_additive_findings
                );
                prop_assert_eq!(
                    actual.decision.blocking_findings,
                    expected.blocking_findings
                );
                prop_assert_eq!(actual.decision.passed, expected.passed);
                Ok(())
            },
        )
        .expect("frozen gate property must match the independent model");
}

#[test]
fn af02_gate_frozen_invalidity_classes_fail_closed() {
    let finding = model_gate_finding(
        gate_truth_table_model::Severity::Breaking,
        gate_truth_table_model::Direction::Producer,
        json!(0),
        json!(1),
    );
    let product = product_gate_finding(&finding);
    let policy = CheckPolicy::default();

    let duplicate_current = check_report(vec![product.clone(), product.clone()], policy);
    assert!(evaluate_quality_gate(&duplicate_current, None, None).is_err());

    let baseline_duplicate = check_report(vec![product.clone(), product.clone()], policy);
    let current = check_report(vec![product.clone()], policy);
    assert!(evaluate_quality_gate(&current, Some(&baseline_duplicate), None).is_err());

    let fingerprint = finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &product)
        .expect("product fingerprint");
    let duplicate_suppression = GateSuppression {
        finding_fingerprint: fingerprint.clone(),
        rationale: "duplicate".to_owned(),
        reference: None,
    };
    let suppressions = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![duplicate_suppression.clone(), duplicate_suppression],
    };
    assert!(evaluate_quality_gate(&current, None, Some(&suppressions)).is_err());

    let mut mismatched_package = check_report(vec![product.clone()], policy);
    mismatched_package.compatibility.package_name = "different.package".to_owned();
    assert!(evaluate_quality_gate(&current, Some(&mismatched_package), None).is_err());

    let mut mismatched_ruleset = check_report(vec![product.clone()], policy);
    mismatched_ruleset.compatibility.ruleset = "different-ruleset".to_owned();
    assert!(evaluate_quality_gate(&current, Some(&mismatched_ruleset), None).is_err());

    let matching_suppression = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: fingerprint.clone(),
            rationale: "accepted".to_owned(),
            reference: Some("AF02-T033".to_owned()),
        }],
    };
    let mut report = evaluate_quality_gate(&current, None, Some(&matching_suppression))
        .expect("valid suppression report");
    report.findings[0]
        .matched_suppression
        .as_mut()
        .expect("matched suppression")
        .rationale = "tampered".to_owned();
    assert!(validate_quality_gate_report(&report).is_err());

    let mut report = evaluate_quality_gate(&current, None, None).expect("valid gate report");
    report.findings[0].fingerprint.digest = format!("sha256:{}", "f".repeat(64));
    assert!(validate_quality_gate_report(&report).is_err());

    let mut report = evaluate_quality_gate(&current, None, None).expect("valid gate report");
    report.decision.blocking_findings += 1;
    assert!(validate_quality_gate_report(&report).is_err());

    let unused = FindingFingerprint {
        schema: FindingFingerprint::SCHEMA_V1,
        digest: format!("sha256:{}", "d".repeat(64)),
    };
    let suppressions = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: unused,
            rationale: "unused".to_owned(),
            reference: None,
        }],
    };
    let mut report = evaluate_quality_gate(&current, None, Some(&suppressions))
        .expect("valid unused suppression");
    report.unused_suppressions.clear();
    assert!(validate_quality_gate_report(&report).is_err());
}

#[test]
fn af02_t033_property_identities_are_frozen() {
    assert_eq!(property_runner::CASE_COUNT, 256);
    assert_eq!(property_runner::MAX_SHRINK_ITERS, 4096);
    assert_eq!(CANONICAL_REFERENCE_PROPERTY, "PROP-CANONICAL-REFERENCE-001");
    assert_eq!(CONTEXT_GRAPH_PROPERTY, "PROP-CONTEXT-GRAPH-ORDER-001");
    assert_eq!(GATE_PROPERTY, "PROP-GATE-FINGERPRINT-SUPPRESSION-001");
    assert_eq!(PORTABLE_PATH_PROPERTY, "PROP-PORTABLE-PATH-001");
    assert_eq!(
        CANONICAL_REFERENCE_SEED_HEX,
        "460617d7c267dbea82a6571424601e1c12d468d07b55842e01cd967e9ca57f84"
    );
    assert_eq!(
        CONTEXT_GRAPH_SEED_HEX,
        "8961a39186ec93f00f2757102a78d6ba457bc2837c132ad551e96f5970cc2e4a"
    );
    assert_eq!(
        GATE_SEED_HEX,
        "7aee90d41e14cd0d4f509e11eadace4ce6823f4c214246761a1f89752dd4a48a"
    );
    assert_eq!(
        PORTABLE_PATH_SEED_HEX,
        "c15f7011fce0482f9839861a576d9c108142c08bc95d4bfb444981c25b874c3f"
    );
    assert_eq!(canonical_reference_model::INVALIDITY_CLASSES.len(), 6);
    assert_eq!(context_graph_order_model::INVALIDITY_CLASSES.len(), 7);
    assert_eq!(gate_truth_table_model::INVALIDITY_CLASSES.len(), 9);
    assert_eq!(portable_path_model::INVALIDITY_CLASSES.len(), 8);
}
