use std::io::Cursor;

use commandf_pkg::{diff_package_archives, PackageCache, StructuralDiffError};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};

fn archive(body: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let mut header = Header::new_gnu();
        header
            .set_path("package/StructureDefinition-example.json")
            .unwrap();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, Cursor::new(body)).unwrap();
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
        "2.0.0",
        PackageCache::digest(after),
        after,
    )
}

fn valid_structure() -> &'static [u8] {
    br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","kind":"resource","abstract":false,"type":"Observation","snapshot":{"element":[{"id":"Observation","path":"Observation","min":0,"max":"1","mustSupport":false,"isModifier":false,"isSummary":false}]}}"#
}

#[test]
fn malformed_element_scalar_and_container_shapes_fail_closed() {
    let elements = [
        br#"{"id":"Observation","path":42}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","min":"one"}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","max":1}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","mustSupport":"true"}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","sliceIsConstraining":"false"}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","slicing":[]}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","binding":[]}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","extension":{}}"#.as_slice(),
        br#"{"id":"Observation","path":"Observation","maxLength":"10"}"#.as_slice(),
    ];
    let after = archive(valid_structure());

    for element in elements {
        let body = format!(
            "{{\"resourceType\":\"StructureDefinition\",\"id\":\"example\",\"url\":\"https://example.org/StructureDefinition/example\",\"kind\":\"resource\",\"abstract\":false,\"type\":\"Observation\",\"snapshot\":{{\"element\":[{}]}}}}",
            std::str::from_utf8(element).unwrap()
        );
        let before = archive(body.as_bytes());
        assert!(matches!(
            diff(&before, &after),
            Err(StructuralDiffError::InvalidStructuralField { .. })
        ));
    }
}

#[test]
fn malformed_structure_definition_metadata_shapes_fail_closed() {
    let bodies = [
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","kind":false,"abstract":false,"type":"Observation","snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#.as_slice(),
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","kind":"resource","abstract":"false","type":"Observation","snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#.as_slice(),
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","kind":"resource","abstract":false,"type":42,"snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#.as_slice(),
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","kind":"resource","abstract":false,"type":"Observation","baseDefinition":42,"snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#.as_slice(),
    ];
    let after = archive(valid_structure());

    for body in bodies {
        let before = archive(body);
        assert!(matches!(
            diff(&before, &after),
            Err(StructuralDiffError::InvalidStructuralField { .. })
        ));
    }
}
