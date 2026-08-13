use std::io::Cursor;

use commandf_pkg::{inspect_package, ArtifactError, ElementView, PackageCache};
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

fn inspect(bytes: &[u8]) -> Result<commandf_pkg::PackageInspection, ArtifactError> {
    inspect_package("example.pkg", "1.0.0", PackageCache::digest(bytes), bytes)
}

#[test]
fn rebuilds_inventory_from_resources_and_ignores_derived_index() {
    let b = br#"{"resourceType":"Patient","id":"b"}"#;
    let invalid_index = b"not-json";
    let a = br#"{"resourceType":"Patient","id":"a"}"#;
    let bytes = archive_with_entries(&[
        ("package/B.json", b),
        ("package/.index.json", invalid_index),
        ("package/A.json", a),
        ("package/examples/ignored.json", b),
    ]);

    let report = inspect(&bytes).unwrap();
    let names = report
        .resources
        .iter()
        .map(|resource| resource.filename.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["A.json", "B.json"]);
    assert_eq!(report.archive_sha256, PackageCache::digest(&bytes));
    assert!(report.resources.iter().all(|resource| resource.sha256.len() == 64));
}

#[test]
fn rejects_duplicate_versioned_canonical_identity() {
    let first = br#"{"resourceType":"ValueSet","id":"a","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#;
    let second = br#"{"resourceType":"CodeSystem","id":"b","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#;
    let bytes = archive_with_entries(&[("package/a.json", first), ("package/b.json", second)]);

    let error = inspect(&bytes).unwrap_err();
    assert!(matches!(error, ArtifactError::DuplicateCanonical { .. }));
}

#[test]
fn rejects_duplicate_element_id_within_structure_view() {
    let structure = br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "version":"1.0.0",
      "snapshot":{"element":[
        {"id":"Observation.component:lab"},
        {"id":"Observation.component:lab"}
      ]}
    }"#;
    let bytes = archive_with_entries(&[("package/StructureDefinition-example.json", structure)]);

    let error = inspect(&bytes).unwrap_err();
    assert!(matches!(error, ArtifactError::DuplicateElementId { .. }));
}

#[test]
fn preserves_slice_aware_ids_and_distinguishes_views() {
    let structure = br#"{
      "resourceType":"StructureDefinition",
      "id":"example",
      "url":"https://example.org/StructureDefinition/example",
      "version":"1.0.0",
      "snapshot":{"element":[
        {"id":"Observation.component:lab","path":"Observation.component","sliceName":"lab"}
      ]},
      "differential":{"element":[
        {"id":"Observation.component:lab.value[x]","path":"Observation.component.value[x]"}
      ]}
    }"#;
    let bytes = archive_with_entries(&[("package/StructureDefinition-example.json", structure)]);

    let report = inspect(&bytes).unwrap();
    let elements = &report.resources[0].elements;
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].view, ElementView::Snapshot);
    assert_eq!(elements[0].element_id, "Observation.component:lab");
    assert_eq!(elements[0].slice_name.as_deref(), Some("lab"));
    assert_eq!(elements[1].view, ElementView::Differential);
    assert_eq!(elements[1].element_id, "Observation.component:lab.value[x]");
}
