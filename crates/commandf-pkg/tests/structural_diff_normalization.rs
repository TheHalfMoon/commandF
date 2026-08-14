use std::io::Cursor;

use commandf_pkg::{
    diff_package_archives, ElementView, PackageCache, StructuralChangeKind, StructuralDiffError,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

fn archive(resource: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let mut header = Header::new_gnu();
        header
            .set_path("package/StructureDefinition-example.json")
            .unwrap();
        header.set_size(resource.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, Cursor::new(resource)).unwrap();
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn diff(
    before: &[u8],
    after: &[u8],
) -> Result<commandf_pkg::StructuralDiffReport, StructuralDiffError> {
    diff_package_archives(
        "example.pkg",
        "1.0.0",
        PackageCache::digest(before),
        before,
        "1.0.1",
        PackageCache::digest(after),
        after,
    )
}

#[test]
fn type_and_nested_set_reordering_is_not_a_structural_change() {
    let before = archive(
        br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "snapshot":{"element":[{
        "id":"Observation.value[x]",
        "path":"Observation.value[x]",
        "type":[
          {
            "code":"CodeableConcept",
            "profile":["https://example.org/b","https://example.org/a"],
            "targetProfile":["https://example.org/d","https://example.org/c"],
            "aggregation":["referenced","contained"]
          },
          {"code":"Quantity"}
        ]
      }]}
    }"#,
    );
    let after = archive(
        br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "snapshot":{"element":[{
        "id":"Observation.value[x]",
        "path":"Observation.value[x]",
        "type":[
          {"code":"Quantity"},
          {
            "code":"CodeableConcept",
            "profile":["https://example.org/a","https://example.org/b"],
            "targetProfile":["https://example.org/c","https://example.org/d"],
            "aggregation":["contained","referenced"]
          }
        ]
      }]}
    }"#,
    );

    let report = diff(&before, &after).unwrap();

    assert!(report
        .changes
        .iter()
        .any(|change| change.kind == StructuralChangeKind::ResourceBytesChanged));
    assert!(!report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementFieldChanged
            && change.field.as_deref() == Some("type")
    }));
}

#[test]
fn extension_reordering_remains_a_structural_change() {
    let before = archive(
        br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "snapshot":{"element":[{
        "id":"Observation.value[x]",
        "path":"Observation.value[x]",
        "extension":[
          {"url":"https://example.org/a","valueString":"a"},
          {"url":"https://example.org/b","valueString":"b"}
        ]
      }]}
    }"#,
    );
    let after = archive(
        br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "snapshot":{"element":[{
        "id":"Observation.value[x]",
        "path":"Observation.value[x]",
        "extension":[
          {"url":"https://example.org/b","valueString":"b"},
          {"url":"https://example.org/a","valueString":"a"}
        ]
      }]}
    }"#,
    );

    let report = diff(&before, &after).unwrap();

    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementFieldChanged
            && change.field.as_deref() == Some("extension")
    }));
}

#[test]
fn view_and_element_removals_are_explicit() {
    let before = archive(
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
    );
    let after = archive(
        br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "snapshot":{"element":[{"id":"Observation","path":"Observation"}]}
    }"#,
    );

    let report = diff(&before, &after).unwrap();

    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ViewRemoved
            && change.view == Some(ElementView::Differential)
    }));
    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementRemoved
            && change.view == Some(ElementView::Snapshot)
            && change.element_id.as_deref() == Some("Observation.status")
    }));
    assert!(report.changes.iter().any(|change| {
        change.kind == StructuralChangeKind::ElementRemoved
            && change.view == Some(ElementView::Differential)
            && change.element_id.as_deref() == Some("Observation.status")
    }));
}
