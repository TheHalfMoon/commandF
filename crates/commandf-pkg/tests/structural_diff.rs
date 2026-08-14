use std::collections::BTreeSet;
use std::io::Cursor;

use commandf_pkg::{
    diff_package_archives, ElementView, PackageCache, ResourceKeyKind, StructuralChangeKind,
    StructuralDiffError,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

fn archive_with_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for (path, body) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).unwrap();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, Cursor::new(*body)).unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn diff(
    before_version: &str,
    before: &[u8],
    after_version: &str,
    after: &[u8],
) -> Result<commandf_pkg::StructuralDiffReport, StructuralDiffError> {
    diff_package_archives(
        "example.pkg",
        before_version,
        PackageCache::digest(before),
        before,
        after_version,
        PackageCache::digest(after),
        after,
    )
}

#[test]
fn self_diff_is_empty_and_serialization_is_stable() {
    let archive = archive_with_entries(&[(
        "package/Patient-example.json",
        br#"{"resourceType":"Patient","id":"example"}"#,
    )]);

    let first = diff("1.0.0", &archive, "1.0.0", &archive).unwrap();
    let second = diff("1.0.0", &archive, "1.0.0", &archive).unwrap();

    assert!(first.changes.is_empty());
    assert_eq!(
        first.to_json_bytes().unwrap(),
        second.to_json_bytes().unwrap()
    );
}

#[test]
fn unique_canonical_url_matches_across_version_change() {
    let before = archive_with_entries(&[(
        "package/ValueSet-example.json",
        br#"{"resourceType":"ValueSet","id":"example","url":"https://example.org/ValueSet/example","version":"1.0.0"}"#,
    )]);
    let after = archive_with_entries(&[(
        "package/ValueSet-example.json",
        br#"{"resourceType":"ValueSet","id":"example","url":"https://example.org/ValueSet/example","version":"2.0.0"}"#,
    )]);

    let report = diff("1.0.0", &before, "2.0.0", &after).unwrap();
    let kinds = report
        .changes
        .iter()
        .map(|change| change.kind)
        .collect::<BTreeSet<_>>();

    assert!(kinds.contains(&StructuralChangeKind::ResourceVersionChanged));
    assert!(kinds.contains(&StructuralChangeKind::ResourceBytesChanged));
    assert!(!kinds.contains(&StructuralChangeKind::ResourceAdded));
    assert!(!kinds.contains(&StructuralChangeKind::ResourceRemoved));
    assert!(report
        .changes
        .iter()
        .all(|change| change.resource.kind == ResourceKeyKind::Canonical));
}

#[test]
fn multi_version_canonical_group_uses_versioned_keys() {
    let before = archive_with_entries(&[
        (
            "package/ValueSet-v1.json",
            br#"{"resourceType":"ValueSet","id":"v1","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#,
        ),
        (
            "package/ValueSet-v2.json",
            br#"{"resourceType":"ValueSet","id":"v2","url":"https://example.org/ValueSet/shared","version":"2.0.0"}"#,
        ),
    ]);
    let after = archive_with_entries(&[
        (
            "package/ValueSet-v1.json",
            br#"{"resourceType":"ValueSet","id":"v1","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#,
        ),
        (
            "package/ValueSet-v3.json",
            br#"{"resourceType":"ValueSet","id":"v3","url":"https://example.org/ValueSet/shared","version":"3.0.0"}"#,
        ),
    ]);

    let report = diff("1.0.0", &before, "2.0.0", &after).unwrap();
    let removed = report
        .changes
        .iter()
        .find(|change| change.kind == StructuralChangeKind::ResourceRemoved)
        .unwrap();
    let added = report
        .changes
        .iter()
        .find(|change| change.kind == StructuralChangeKind::ResourceAdded)
        .unwrap();

    assert_eq!(
        removed.resource.value,
        "https://example.org/ValueSet/shared|2.0.0"
    );
    assert_eq!(
        added.resource.value,
        "https://example.org/ValueSet/shared|3.0.0"
    );
}

#[test]
fn ambiguous_noncanonical_resource_id_fails_closed() {
    let before = archive_with_entries(&[
        (
            "package/Patient-a.json",
            br#"{"resourceType":"Patient","id":"shared"}"#,
        ),
        (
            "package/Patient-b.json",
            br#"{"resourceType":"Patient","id":"shared"}"#,
        ),
    ]);
    let after = archive_with_entries(&[(
        "package/Patient-a.json",
        br#"{"resourceType":"Patient","id":"shared"}"#,
    )]);

    let error = diff("1.0.0", &before, "2.0.0", &after).unwrap_err();
    assert!(matches!(
        error,
        StructuralDiffError::AmbiguousResourceKey { .. }
    ));
}

#[test]
fn structure_diff_reports_structural_fields_without_editorial_noise() {
    let before = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{
          "resourceType":"StructureDefinition",
          "id":"example",
          "url":"https://example.org/StructureDefinition/example",
          "version":"1.0.0",
          "snapshot":{"element":[
            {
              "id":"Observation.value[x]",
              "path":"Observation.value[x]",
              "short":"before prose",
              "min":0,
              "representation":["typeAttr","xmlAttr"],
              "condition":["b","a"],
              "type":[
                {"code":"Quantity"},
                {"code":"CodeableConcept","profile":["https://example.org/b","https://example.org/a"]}
              ],
              "slicing":{"rules":"open"},
              "binding":{"strength":"preferred"},
              "fixedString":"before",
              "constraint":[
                {"key":"b","severity":"warning"},
                {"key":"a","severity":"error"}
              ]
            }
          ]}
        }"#,
    )]);
    let after = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{
          "resourceType":"StructureDefinition",
          "id":"example",
          "url":"https://example.org/StructureDefinition/example",
          "version":"1.0.0",
          "snapshot":{"element":[
            {
              "id":"Observation.value[x]",
              "path":"Observation.value[x]",
              "short":"after prose",
              "min":1,
              "representation":["xmlAttr","typeAttr"],
              "condition":["a","b"],
              "type":[{"code":"string"}],
              "slicing":{"rules":"closed"},
              "binding":{"strength":"required"},
              "fixedString":"after",
              "constraint":[
                {"key":"a","severity":"error"},
                {"key":"b","severity":"warning"}
              ]
            }
          ]}
        }"#,
    )]);

    let report = diff("1.0.0", &before, "1.0.1", &after).unwrap();
    let changed_fields = report
        .changes
        .iter()
        .filter(|change| change.kind == StructuralChangeKind::ElementFieldChanged)
        .filter_map(|change| change.field.as_deref())
        .collect::<BTreeSet<_>>();

    assert!(changed_fields.contains("min"));
    assert!(changed_fields.contains("type"));
    assert!(changed_fields.contains("slicing"));
    assert!(changed_fields.contains("binding"));
    assert!(changed_fields.contains("fixedString"));
    assert!(!changed_fields.contains("short"));
    assert!(!changed_fields.contains("representation"));
    assert!(!changed_fields.contains("condition"));
    assert!(!changed_fields.contains("constraint"));
}

#[test]
fn view_and_element_additions_are_explicit() {
    let before = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{
          "resourceType":"StructureDefinition",
          "id":"example",
          "url":"https://example.org/StructureDefinition/example",
          "snapshot":{"element":[{"id":"Observation","path":"Observation"}]}
        }"#,
    )]);
    let after = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{
          "resourceType":"StructureDefinition",
          "id":"example",
          "url":"https://example.org/StructureDefinition/example",
          "snapshot":{"element":[
            {"id":"Observation","path":"Observation"},
            {"id":"Observation.status","path":"Observation.status"}
          ]},
          "differential":{"element":[
            {"id":"Observation.status","path":"Observation.status","min":1}
          ]}
        }"#,
    )]);

    let report = diff("1.0.0", &before, "1.0.1", &after).unwrap();

    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ViewAdded
            && change.view == Some(ElementView::Differential)
    }));
    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementAdded
            && change.view == Some(ElementView::Snapshot)
            && change.element_id.as_deref() == Some("Observation.status")
    }));
    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementAdded
            && change.view == Some(ElementView::Differential)
            && change.element_id.as_deref() == Some("Observation.status")
    }));
}
