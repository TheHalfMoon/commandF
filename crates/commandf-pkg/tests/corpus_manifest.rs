use std::fs;
use std::path::PathBuf;

use commandf_pkg::{
    canonical_corpus_manifest_bytes, parse_corpus_manifest, CorpusError, MAX_CORPUS_ARCHIVE_BYTES,
    MAX_CORPUS_CASES, MAX_CORPUS_MANIFEST_BYTES,
};
use serde_json::{json, Value};

fn canonical_manifest_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/real-ig/v1/corpus.json");
    fs::read(path).expect("canonical CF-10 manifest should be readable")
}

fn canonical_value() -> Value {
    serde_json::from_slice(&canonical_manifest_bytes()).expect("canonical manifest should be JSON")
}

fn parse_value(value: &Value) -> Result<commandf_pkg::RealIgCorpus, CorpusError> {
    parse_corpus_manifest(&serde_json::to_vec(value).expect("test JSON should serialize"))
}

#[test]
fn canonical_manifest_matches_frozen_discovery_evidence() {
    let corpus = parse_corpus_manifest(&canonical_manifest_bytes()).expect("manifest should validate");
    assert_eq!(corpus.cases.len(), 3);

    assert_eq!(corpus.cases[0].id, "C001");
    assert_eq!(corpus.cases[0].package, "hl7.fhir.us.core");
    assert_eq!(corpus.cases[0].before.version, "8.0.1");
    assert_eq!(
        corpus.cases[0].before.archive_sha256,
        "3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464"
    );
    assert_eq!(corpus.cases[0].before.archive_bytes, 2_713_046);
    assert_eq!(corpus.cases[0].after.version, "9.0.0");
    assert_eq!(
        corpus.cases[0].after.archive_sha256,
        "d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059"
    );
    assert_eq!(corpus.cases[0].after.archive_bytes, 2_749_959);

    assert_eq!(corpus.cases[1].id, "C002");
    assert_eq!(corpus.cases[1].package, "hl7.fhir.uv.ips");
    assert_eq!(corpus.cases[1].before.version, "1.1.0");
    assert_eq!(
        corpus.cases[1].before.archive_sha256,
        "403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef"
    );
    assert_eq!(corpus.cases[1].before.archive_bytes, 1_065_103);
    assert_eq!(corpus.cases[1].after.version, "2.0.1");
    assert_eq!(
        corpus.cases[1].after.archive_sha256,
        "7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799"
    );
    assert_eq!(corpus.cases[1].after.archive_bytes, 725_312);

    assert_eq!(corpus.cases[2].id, "C003");
    assert_eq!(corpus.cases[2].package, "hl7.fhir.us.mcode");
    assert_eq!(corpus.cases[2].before.version, "3.0.0");
    assert_eq!(
        corpus.cases[2].before.archive_sha256,
        "c94c91971747efeae760aa037d168e4df992cefb6dacece08217c464b9d39214"
    );
    assert_eq!(corpus.cases[2].before.archive_bytes, 1_014_084);
    assert_eq!(corpus.cases[2].after.version, "4.0.0");
    assert_eq!(
        corpus.cases[2].after.archive_sha256,
        "e603283bafa508a3705ad022bce95bba1fbd0b8b3b87b978e7412813b7bc1778"
    );
    assert_eq!(corpus.cases[2].after.archive_bytes, 1_003_918);
}

#[test]
fn canonical_round_trip_is_deterministic() {
    let corpus = parse_corpus_manifest(&canonical_manifest_bytes()).expect("manifest should validate");
    let first = canonical_corpus_manifest_bytes(&corpus).expect("serialization should succeed");
    let reparsed = parse_corpus_manifest(&first).expect("canonical bytes should validate");
    let second = canonical_corpus_manifest_bytes(&reparsed).expect("serialization should succeed");
    assert_eq!(first, second);
    assert_eq!(corpus, reparsed);
}

#[test]
fn oversized_manifest_fails_before_json_decode() {
    let bytes = vec![b' '; MAX_CORPUS_MANIFEST_BYTES + 1];
    assert!(matches!(
        parse_corpus_manifest(&bytes),
        Err(CorpusError::ManifestTooLarge { .. })
    ));
}

#[test]
fn wrong_schema_fails_closed() {
    let mut value = canonical_value();
    value["schema"] = json!(2);
    assert_eq!(parse_value(&value), Err(CorpusError::UnsupportedSchema(2)));
}

#[test]
fn empty_and_oversized_case_sets_fail_closed() {
    let mut empty = canonical_value();
    empty["cases"] = json!([]);
    assert_eq!(parse_value(&empty), Err(CorpusError::EmptyCorpus));

    let mut oversized = canonical_value();
    let case = oversized["cases"][0].clone();
    oversized["cases"] = Value::Array(vec![case; MAX_CORPUS_CASES + 1]);
    assert!(matches!(
        parse_value(&oversized),
        Err(CorpusError::TooManyCases { .. })
    ));
}

#[test]
fn duplicate_and_out_of_order_case_ids_fail_closed() {
    let mut duplicate = canonical_value();
    duplicate["cases"][1]["id"] = json!("C001");
    assert_eq!(
        parse_value(&duplicate),
        Err(CorpusError::DuplicateCaseId("C001".to_owned()))
    );

    let mut out_of_order = canonical_value();
    out_of_order["cases"].as_array_mut().unwrap().swap(0, 1);
    assert!(matches!(
        parse_value(&out_of_order),
        Err(CorpusError::NonCanonicalCaseOrder { .. })
    ));
}

#[test]
fn malformed_identity_and_versions_fail_closed() {
    let mut bad_id = canonical_value();
    bad_id["cases"][0]["id"] = json!("../1");
    assert!(matches!(
        parse_value(&bad_id),
        Err(CorpusError::InvalidCaseId(_))
    ));

    let mut bad_package = canonical_value();
    bad_package["cases"][0]["package"] = json!("../../etc/passwd");
    assert!(matches!(
        parse_value(&bad_package),
        Err(CorpusError::InvalidPackageName { .. })
    ));

    let mut bad_version = canonical_value();
    bad_version["cases"][0]["before"]["version"] = json!("8.x");
    assert!(matches!(
        parse_value(&bad_version),
        Err(CorpusError::InvalidVersion { .. })
    ));

    let mut same_version = canonical_value();
    same_version["cases"][0]["after"]["version"] = json!("8.0.1");
    assert_eq!(
        parse_value(&same_version),
        Err(CorpusError::SameVersion("C001".to_owned()))
    );
}

#[test]
fn non_r4_digest_and_size_fail_closed() {
    let mut non_r4 = canonical_value();
    non_r4["cases"][0]["fhir_version"] = json!("5.0.0");
    assert!(matches!(
        parse_value(&non_r4),
        Err(CorpusError::UnsupportedFhirVersion { .. })
    ));

    let mut bad_digest = canonical_value();
    bad_digest["cases"][0]["before"]["archive_sha256"] = json!("ABCDEF");
    assert!(matches!(
        parse_value(&bad_digest),
        Err(CorpusError::InvalidArchiveSha256 { .. })
    ));

    let mut zero_size = canonical_value();
    zero_size["cases"][0]["before"]["archive_bytes"] = json!(0);
    assert!(matches!(
        parse_value(&zero_size),
        Err(CorpusError::InvalidArchiveSize { .. })
    ));

    let mut huge_size = canonical_value();
    huge_size["cases"][0]["before"]["archive_bytes"] = json!(MAX_CORPUS_ARCHIVE_BYTES + 1);
    assert!(matches!(
        parse_value(&huge_size),
        Err(CorpusError::InvalidArchiveSize { .. })
    ));
}

#[test]
fn evidence_urls_and_publisher_fail_closed() {
    let mut http_publication = canonical_value();
    http_publication["cases"][0]["before"]["publication_url"] =
        json!("http://example.invalid/ig");
    assert!(matches!(
        parse_value(&http_publication),
        Err(CorpusError::InvalidEvidence { .. })
    ));

    let mut empty_publisher = canonical_value();
    empty_publisher["cases"][0]["publisher"] = json!("   ");
    assert!(matches!(
        parse_value(&empty_publisher),
        Err(CorpusError::InvalidEvidence { .. })
    ));
}

#[test]
fn unknown_fields_missing_fields_and_unknown_enums_fail_closed() {
    let mut unknown_field = canonical_value();
    unknown_field["cases"][0]["unexpected"] = json!(true);
    assert!(matches!(
        parse_value(&unknown_field),
        Err(CorpusError::InvalidJson(_))
    ));

    let mut missing_field = canonical_value();
    missing_field["cases"][0]
        .as_object_mut()
        .unwrap()
        .remove("change_evidence_url");
    assert!(matches!(
        parse_value(&missing_field),
        Err(CorpusError::InvalidJson(_))
    ));

    let mut unknown_rights = canonical_value();
    unknown_rights["cases"][0]["rights_mode"] = json!("redistribute_everything");
    assert!(matches!(
        parse_value(&unknown_rights),
        Err(CorpusError::InvalidJson(_))
    ));

    let mut unknown_oracle = canonical_value();
    unknown_oracle["cases"][0]["oracle_mode"] = json!("always");
    assert!(matches!(
        parse_value(&unknown_oracle),
        Err(CorpusError::InvalidJson(_))
    ));
}
