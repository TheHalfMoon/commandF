use commandf_pkg::{
    parse_hl7_oracle_report, reconcile_hl7_oracle, Hl7OracleReport, OracleChangeState,
    OracleIdentity, OracleMessage, OracleMessageLevel, OracleResourceStatus, OracleStates,
    PackageEvidence, ResourceKey, ResourceKeyKind, StructuralChange, StructuralChangeKind,
    StructuralDiffReport,
};

fn key(value: &str) -> ResourceKey {
    ResourceKey {
        kind: ResourceKeyKind::Canonical,
        value: value.to_owned(),
    }
}

fn diff(changes: Vec<StructuralChange>) -> StructuralDiffReport {
    StructuralDiffReport {
        schema: StructuralDiffReport::SCHEMA_V1,
        package_name: "example.fhir.ig".to_owned(),
        before: PackageEvidence {
            version: "1.0.0".to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: "1.1.0".to_owned(),
            archive_sha256: "b".repeat(64),
        },
        changes,
    }
}

fn change(resource: &ResourceKey, kind: StructuralChangeKind) -> StructuralChange {
    StructuralChange {
        kind,
        resource: resource.clone(),
        before_filename: Some("before.json".to_owned()),
        after_filename: Some("after.json".to_owned()),
        view: None,
        element_id: None,
        field: None,
        before: None,
        after: None,
    }
}

fn oracle(resource: &str, changed: bool) -> Hl7OracleReport {
    Hl7OracleReport {
        schema: Hl7OracleReport::SCHEMA_V1,
        oracle: OracleIdentity::pinned_hl7(),
        left_identity: resource.to_owned(),
        right_identity: resource.to_owned(),
        states: OracleStates {
            metadata: OracleChangeState::NotChanged,
            definitions: if changed {
                OracleChangeState::Changed
            } else {
                OracleChangeState::NotChanged
            },
            content: OracleChangeState::Unknown,
            content_interpretation: OracleChangeState::Unknown,
        },
        messages: if changed {
            vec![OracleMessage {
                level: OracleMessageLevel::Warning,
                location: "Patient.active".to_owned(),
                message: "Elements differ".to_owned(),
            }]
        } else {
            vec![]
        },
    }
}

#[test]
fn exact_identity_and_schema_are_required() {
    let mut report = oracle("http://example.org/StructureDefinition/example", false);
    report.oracle.release = "future".to_owned();
    let bytes = serde_json::to_vec(&report).unwrap();
    let error = parse_hl7_oracle_report(&bytes).unwrap_err().to_string();
    assert!(error.contains("oracle identity mismatch: release"));

    let mut report = oracle("http://example.org/StructureDefinition/example", false);
    report.schema = 2;
    let bytes = serde_json::to_vec(&report).unwrap();
    assert!(parse_hl7_oracle_report(&bytes)
        .unwrap_err()
        .to_string()
        .contains("unsupported oracle report schema"));
}

#[test]
fn reconciliation_distinguishes_all_evidence_relationships() {
    let agreement = key("http://example.org/StructureDefinition/agreement");
    let commandf_only = key("http://example.org/StructureDefinition/commandf-only");
    let authority_only = key("http://example.org/StructureDefinition/authority-only");
    let both = key("http://example.org/StructureDefinition/both");
    let removed = key("http://example.org/StructureDefinition/removed");

    let report = reconcile_hl7_oracle(
        diff(vec![
            change(&commandf_only, StructuralChangeKind::ResourceBytesChanged),
            change(&both, StructuralChangeKind::ResourceBytesChanged),
            change(&removed, StructuralChangeKind::ResourceRemoved),
        ]),
        vec![
            (agreement.clone(), oracle(&agreement.value, false)),
            (commandf_only.clone(), oracle(&commandf_only.value, false)),
            (authority_only.clone(), oracle(&authority_only.value, true)),
            (both.clone(), oracle(&both.value, true)),
        ],
    )
    .unwrap();

    let status = |resource: &ResourceKey| {
        report
            .resources
            .iter()
            .find(|result| &result.resource == resource)
            .unwrap()
            .status
    };
    assert_eq!(status(&agreement), OracleResourceStatus::Agreement);
    assert_eq!(status(&commandf_only), OracleResourceStatus::CommandfOnly);
    assert_eq!(status(&authority_only), OracleResourceStatus::AuthorityOnly);
    assert_eq!(status(&both), OracleResourceStatus::BothChanged);
    assert_eq!(status(&removed), OracleResourceStatus::Uncomparable);
    assert_eq!(report.structural_diff.changes.len(), 3);
}

#[test]
fn missing_observation_never_becomes_false_agreement() {
    let resource = key("http://example.org/StructureDefinition/example");
    let report = reconcile_hl7_oracle(
        diff(vec![change(
            &resource,
            StructuralChangeKind::ResourceBytesChanged,
        )]),
        vec![],
    )
    .unwrap();
    assert_eq!(report.resources[0].status, OracleResourceStatus::Uncomparable);
}

#[test]
fn observation_identity_mismatch_and_duplicates_fail_closed() {
    let resource = key("http://example.org/StructureDefinition/example");
    let mismatch = oracle("http://example.org/StructureDefinition/other", false);
    assert!(reconcile_hl7_oracle(diff(vec![]), vec![(resource.clone(), mismatch)])
        .unwrap_err()
        .to_string()
        .contains("observation identity mismatch"));

    let observation = oracle(&resource.value, false);
    assert!(reconcile_hl7_oracle(
        diff(vec![]),
        vec![
            (resource.clone(), observation.clone()),
            (resource, observation),
        ],
    )
    .unwrap_err()
    .to_string()
    .contains("duplicate oracle observation"));
}

#[test]
fn repeated_reconciliation_is_byte_deterministic() {
    let resource = key("http://example.org/StructureDefinition/example");
    let input = diff(vec![change(
        &resource,
        StructuralChangeKind::ResourceBytesChanged,
    )]);
    let observation = oracle(&resource.value, true);
    let first = reconcile_hl7_oracle(
        input.clone(),
        vec![(resource.clone(), observation.clone())],
    )
    .unwrap();
    let second = reconcile_hl7_oracle(input, vec![(resource, observation)]).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.to_json_bytes().unwrap(), second.to_json_bytes().unwrap());
}
