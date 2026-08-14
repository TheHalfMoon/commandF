use std::io::Cursor;

use commandf_pkg::{diff_package_archives, PackageCache, StructuralDiffError};
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

fn diff(before: &[u8], after: &[u8]) -> Result<commandf_pkg::StructuralDiffReport, StructuralDiffError> {
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

#[test]
fn canonical_multiplicity_without_usable_version_fails_closed() {
    let before = archive_with_entries(&[
        (
            "package/ValueSet-missing.json",
            br#"{"resourceType":"ValueSet","id":"missing","url":"https://example.org/ValueSet/shared"}"#,
        ),
        (
            "package/ValueSet-v1.json",
            br#"{"resourceType":"ValueSet","id":"v1","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#,
        ),
    ]);
    let after = archive_with_entries(&[(
        "package/ValueSet-v1.json",
        br#"{"resourceType":"ValueSet","id":"v1","url":"https://example.org/ValueSet/shared","version":"1.0.0"}"#,
    )]);

    let error = diff(&before, &after).unwrap_err();
    assert!(matches!(
        error,
        StructuralDiffError::CanonicalMultiplicityMissingVersion { .. }
    ));
}

#[test]
fn malformed_element_structural_fields_fail_closed() {
    let cases = [
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","representation":"xmlAttr"}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","condition":{"bad":true}}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","type":{"code":"string"}}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","type":[{"profile":["x"]}]}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","type":[{"code":"string","profile":"x"}]}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","constraint":{"key":"a"}}"#.as_slice(),
        br#"{"id":"Observation.value[x]","path":"Observation.value[x]","constraint":[{"severity":"error"}]}"#.as_slice(),
    ];

    for element in cases {
        let before_body = format!(
            "{{\"resourceType\":\"StructureDefinition\",\"id\":\"example\",\"url\":\"https://example.org/StructureDefinition/example\",\"snapshot\":{{\"element\":[{}]}}}}",
            std::str::from_utf8(element).unwrap()
        );
        let after_body = br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","snapshot":{"element":[{"id":"Observation.value[x]","path":"Observation.value[x]"}]}}"#;
        let before = archive_with_entries(&[(
            "package/StructureDefinition-example.json",
            before_body.as_bytes(),
        )]);
        let after = archive_with_entries(&[(
            "package/StructureDefinition-example.json",
            after_body,
        )]);

        let error = diff(&before, &after).unwrap_err();
        assert!(matches!(
            error,
            StructuralDiffError::InvalidStructuralField { .. }
        ));
    }
}

#[test]
fn malformed_resource_context_fields_fail_closed() {
    let before = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","contextInvariant":"not-an-array","snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#,
    )]);
    let after = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#,
    )]);
    assert!(matches!(
        diff(&before, &after),
        Err(StructuralDiffError::InvalidStructuralField { .. })
    ));

    let before = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","context":[{"type":"element"}],"snapshot":{"element":[{"id":"Observation","path":"Observation"}]}}"#,
    )]);
    assert!(matches!(
        diff(&before, &after),
        Err(StructuralDiffError::InvalidStructuralField { .. })
    ));
}

#[test]
fn valid_context_and_interpreted_arrays_continue_to_diff() {
    let before = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","context":[{"type":"element","expression":"Observation"}],"contextInvariant":["b","a"],"snapshot":{"element":[{"id":"Observation.value[x]","path":"Observation.value[x]","representation":["xmlAttr","typeAttr"],"condition":["b","a"],"type":[{"code":"string","profile":["z","a"]}],"constraint":[{"key":"a","severity":"error"}]}]}}"#,
    )]);
    let after = archive_with_entries(&[(
        "package/StructureDefinition-example.json",
        br#"{"resourceType":"StructureDefinition","id":"example","url":"https://example.org/StructureDefinition/example","context":[{"type":"element","expression":"Observation"}],"contextInvariant":["a","b"],"snapshot":{"element":[{"id":"Observation.value[x]","path":"Observation.value[x]","representation":["typeAttr","xmlAttr"],"condition":["a","b"],"type":[{"code":"string","profile":["a","z"]}],"constraint":[{"key":"a","severity":"error"}]}]}}"#,
    )]);

    let report = diff(&before, &after).unwrap();
    assert!(report
        .changes
        .iter()
        .all(|change| change.kind != commandf_pkg::StructuralChangeKind::ElementFieldChanged));
}
