use commandf_pkg::{
    compare_complete_code_systems, compare_value_set_expansions, ResourceKey, ResourceKeyKind,
    TerminologyIndeterminateReason, TerminologyRelation,
};
use serde_json::{json, Value};

fn key(url: &str) -> ResourceKey {
    ResourceKey {
        kind: ResourceKeyKind::Canonical,
        value: url.to_owned(),
    }
}

fn code_system(codes: &[&str]) -> Value {
    json!({
        "resourceType": "CodeSystem",
        "url": "http://example.org/CodeSystem/test",
        "version": "1",
        "caseSensitive": true,
        "content": "complete",
        "count": codes.len(),
        "concept": codes.iter().map(|code| json!({"code": code})).collect::<Vec<_>>()
    })
}

fn expansion(codes: &[&str]) -> Value {
    json!({
        "resourceType": "ValueSet",
        "url": "http://example.org/ValueSet/test",
        "version": "1",
        "expansion": {
            "identifier": "urn:uuid:ignored-for-membership",
            "timestamp": "2026-08-14T00:00:00Z",
            "total": codes.len(),
            "contains": codes.iter().map(|code| json!({
                "system": "http://example.org/CodeSystem/test",
                "version": "1",
                "code": code
            })).collect::<Vec<_>>()
        }
    })
}

#[test]
fn complete_code_system_relations_are_directional() {
    let resource = key("http://example.org/CodeSystem/test");

    let equal = compare_complete_code_systems(
        resource.clone(),
        &code_system(&["a", "b"]),
        &code_system(&["a", "b"]),
    )
    .unwrap();
    assert_eq!(equal.relation, TerminologyRelation::Equal);
    assert!(equal.added.is_empty());
    assert!(equal.removed.is_empty());

    let narrowed = compare_complete_code_systems(
        resource.clone(),
        &code_system(&["a", "b"]),
        &code_system(&["a"]),
    )
    .unwrap();
    assert_eq!(narrowed.relation, TerminologyRelation::Narrowed);
    assert_eq!(narrowed.removed[0].code, "b");

    let widened = compare_complete_code_systems(
        resource.clone(),
        &code_system(&["a"]),
        &code_system(&["a", "b"]),
    )
    .unwrap();
    assert_eq!(widened.relation, TerminologyRelation::Widened);
    assert_eq!(widened.added[0].code, "b");

    let incomparable = compare_complete_code_systems(
        resource,
        &code_system(&["a", "b"]),
        &code_system(&["a", "c"]),
    )
    .unwrap();
    assert_eq!(incomparable.relation, TerminologyRelation::Incomparable);
}

#[test]
fn incomplete_code_system_modes_never_produce_finite_proof() {
    let mut after = code_system(&["a"]);
    after["content"] = json!("fragment");
    let result = compare_complete_code_systems(
        key("http://example.org/CodeSystem/test"),
        &code_system(&["a"]),
        &after,
    )
    .unwrap();
    assert_eq!(result.relation, TerminologyRelation::Indeterminate);
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::CodeSystemNotComplete)
    );

    let mut after = code_system(&["a"]);
    after["compositional"] = json!(true);
    let result = compare_complete_code_systems(
        key("http://example.org/CodeSystem/test"),
        &code_system(&["a"]),
        &after,
    )
    .unwrap();
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::CodeSystemCompositional)
    );

    let mut after = code_system(&["a"]);
    after["caseSensitive"] = json!(false);
    let result = compare_complete_code_systems(
        key("http://example.org/CodeSystem/test"),
        &code_system(&["a"]),
        &after,
    )
    .unwrap();
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::CodeSystemCaseSensitivityChanged)
    );
}

#[test]
fn duplicate_complete_code_system_codes_fail_closed() {
    let mut value = code_system(&["a", "b"]);
    value["concept"] = json!([
        {"code": "a", "concept": [{"code": "b"}]},
        {"code": "b"}
    ]);
    let error =
        compare_complete_code_systems(key("http://example.org/CodeSystem/test"), &value, &value)
            .unwrap_err()
            .to_string();
    assert!(error.contains("duplicate complete CodeSystem concept code b"));
}

#[test]
fn code_system_count_mismatch_is_indeterminate() {
    let mut after = code_system(&["a", "b"]);
    after["count"] = json!(3);
    let result = compare_complete_code_systems(
        key("http://example.org/CodeSystem/test"),
        &code_system(&["a", "b"]),
        &after,
    )
    .unwrap();
    assert_eq!(result.relation, TerminologyRelation::Indeterminate);
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::CodeSystemCountMismatch)
    );
}

#[test]
fn complete_value_set_expansion_relations_are_directional() {
    let resource = key("http://example.org/ValueSet/test");
    let narrowed = compare_value_set_expansions(
        resource.clone(),
        &expansion(&["a", "b"]),
        &expansion(&["a"]),
    )
    .unwrap();
    assert_eq!(narrowed.relation, TerminologyRelation::Narrowed);

    let widened = compare_value_set_expansions(
        resource.clone(),
        &expansion(&["a"]),
        &expansion(&["a", "b"]),
    )
    .unwrap();
    assert_eq!(widened.relation, TerminologyRelation::Widened);

    let incomparable =
        compare_value_set_expansions(resource, &expansion(&["a", "b"]), &expansion(&["a", "c"]))
            .unwrap();
    assert_eq!(incomparable.relation, TerminologyRelation::Incomparable);
}

#[test]
fn hierarchical_expansion_duplicate_is_deduplicated_for_membership() {
    let before = json!({
        "resourceType": "ValueSet",
        "url": "http://example.org/ValueSet/test",
        "version": "1",
        "expansion": {
            "total": 2,
            "contains": [
                {
                    "system": "http://example.org/CodeSystem/test",
                    "code": "a",
                    "contains": [
                        {"system": "http://example.org/CodeSystem/test", "code": "b"}
                    ]
                },
                {"system": "http://example.org/CodeSystem/test", "code": "b"}
            ]
        }
    });
    let after = expansion(&["a", "b"]);
    let result =
        compare_value_set_expansions(key("http://example.org/ValueSet/test"), &before, &after)
            .unwrap();
    assert_eq!(result.relation, TerminologyRelation::Equal);
    assert_eq!(result.before_count, Some(2));
}

#[test]
fn expansion_paging_and_context_mismatch_are_indeterminate() {
    let mut paged = expansion(&["a"]);
    paged["expansion"]["offset"] = json!(1);
    let result = compare_value_set_expansions(
        key("http://example.org/ValueSet/test"),
        &expansion(&["a"]),
        &paged,
    )
    .unwrap();
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::IncompleteOrPagedExpansion)
    );

    let mut before = expansion(&["a"]);
    before["expansion"]["parameter"] = json!([
        {"name": "system-version", "valueUri": "http://example.org/CodeSystem/test|1"},
        {"name": "displayLanguage", "valueCode": "en"}
    ]);
    let mut after = expansion(&["a"]);
    after["expansion"]["parameter"] = json!([
        {"name": "displayLanguage", "valueCode": "en"},
        {"name": "system-version", "valueUri": "http://example.org/CodeSystem/test|2"}
    ]);
    let result =
        compare_value_set_expansions(key("http://example.org/ValueSet/test"), &before, &after)
            .unwrap();
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::ExpansionContextMismatch)
    );
}

#[test]
fn abstract_coded_members_disable_hard_binding_proof() {
    let before = expansion(&["a"]);
    let mut after = expansion(&["a", "b"]);
    after["expansion"]["contains"][1]["abstract"] = json!(true);
    let result =
        compare_value_set_expansions(key("http://example.org/ValueSet/test"), &before, &after)
            .unwrap();
    assert_eq!(result.relation, TerminologyRelation::Widened);
    assert!(!result.binding_proof_eligible);
    assert_eq!(
        result.reason,
        Some(TerminologyIndeterminateReason::AbstractMemberPresent)
    );
}
