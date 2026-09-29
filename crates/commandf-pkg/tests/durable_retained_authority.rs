use std::fs;
use std::path::PathBuf;

use commandf_pkg::{
    classify_regenerated_archive, evaluate_durable_packet, LiveArtifactObservation,
    RegenerationClass, TrustedRetainedBinding,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn trusted() -> TrustedRetainedBinding {
    TrustedRetainedBinding {
        artifact_id: 9_255_732_702,
        artifact_name: "cf10-real-corpus-evidence".to_owned(),
        workflow_run_id: 31_916_124_080,
        recorded_sha256: "9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612"
            .to_owned(),
        retained_head: "5fe10d9859407272acf6649fc3e868d3eb2fbd12".to_owned(),
        retained_base: "5cb1a4c3445c0ebd86654cfb467a5e008e801c3e".to_owned(),
        manifest_git_blob_sha1: "655949a8a30d67502dffd624a175d2e8e02b1d1f".to_owned(),
        donor_git_blob_sha1: "566b46f4e6f467a1ccae3ac810b31956309173b6".to_owned(),
    }
}

fn historical_packet() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/019-durable-offline-retained-authority/historical-packet.json");
    fs::read(path).expect("historical durable packet")
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn packet_with(
    trusted: &TrustedRetainedBinding,
    byte_state: &str,
    durable_digest: Value,
) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "artifact_id": trusted.artifact_id,
        "artifact_name": trusted.artifact_name,
        "donor_git_blob_sha1": trusted.donor_git_blob_sha1,
        "durable_bytes_sha256": durable_digest,
        "historical_artifact_byte_state": byte_state,
        "historical_artifact_identity_state": "HISTORICAL_ARTIFACT_IDENTITY_KNOWN",
        "manifest_git_blob_sha1": trusted.manifest_git_blob_sha1,
        "recorded_sha256": trusted.recorded_sha256,
        "retained_base": trusted.retained_base,
        "retained_head": trusted.retained_head,
        "schema": "commandf.durable-retained-authority/v1",
        "workflow_run_id": trusted.workflow_run_id
    }))
    .unwrap()
}

fn evaluate_historical(live: LiveArtifactObservation) -> commandf_pkg::DurableProjection {
    let binding = trusted();
    evaluate_durable_packet(
        &historical_packet(),
        &binding,
        None,
        &binding.manifest_git_blob_sha1.clone(),
        &binding.donor_git_blob_sha1.clone(),
        live,
    )
    .unwrap()
}

#[test]
fn absent_live_artifact_with_unavailable_bytes_does_not_claim_verification() {
    let projection = evaluate_historical(LiveArtifactObservation::AbsentEmpty);
    assert_eq!(
        projection.historical_artifact_identity_state,
        "HISTORICAL_ARTIFACT_IDENTITY_KNOWN"
    );
    assert_eq!(
        projection.historical_artifact_byte_state,
        "HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE"
    );
    assert_eq!(projection.durable_packet_state, "DURABLE_PACKET_VERIFIED");
    assert_eq!(projection.offline_replay_state, "OFFLINE_REPLAY_VERIFIED");
    assert_eq!(projection.live_github_artifact_state, "ABSENT");
    assert!(!projection.byte_digest_verified);
}

#[test]
fn verified_exact_bytes_pass_when_live_artifact_is_absent() {
    let bytes = b"synthetic-durable-bytes-not-the-historical-artifact";
    let digest = sha256_hex(bytes);
    let mut binding = trusted();
    binding.recorded_sha256 = digest.clone();
    let packet = packet_with(
        &binding,
        "HISTORICAL_ARTIFACT_BYTES_VERIFIED",
        json!(digest),
    );
    let projection = evaluate_durable_packet(
        &packet,
        &binding,
        Some(bytes),
        &binding.manifest_git_blob_sha1.clone(),
        &binding.donor_git_blob_sha1.clone(),
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap();
    assert!(projection.byte_digest_verified);
    assert_eq!(
        projection.historical_artifact_byte_state,
        "HISTORICAL_ARTIFACT_BYTES_VERIFIED"
    );
    assert_ne!(
        projection.recorded_sha256,
        "9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612"
    );
}

#[test]
fn present_exact_live_artifact_does_not_invent_missing_bytes() {
    let projection = evaluate_historical(LiveArtifactObservation::PresentExact);
    assert_eq!(projection.live_github_artifact_state, "PRESENT_EXACT");
    assert!(!projection.byte_digest_verified);
}

#[test]
fn live_identity_mismatch_fails() {
    let error = evaluate_durable_packet(
        &historical_packet(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::IdentityMismatch,
    )
    .unwrap_err();
    assert!(error.to_string().contains("identity mismatch"));
}

#[test]
fn live_digest_mismatch_fails() {
    let error = evaluate_durable_packet(
        &historical_packet(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::DigestMismatch,
    )
    .unwrap_err();
    assert!(error.to_string().contains("digest mismatch"));
}

#[test]
fn durable_byte_mismatch_fails() {
    let bytes = b"not-the-recorded-bytes";
    let mut binding = trusted();
    binding.recorded_sha256 = sha256_hex(b"other");
    let packet = packet_with(
        &binding,
        "HISTORICAL_ARTIFACT_BYTES_VERIFIED",
        json!(binding.recorded_sha256),
    );
    let error = evaluate_durable_packet(
        &packet,
        &binding,
        Some(bytes),
        &binding.manifest_git_blob_sha1.clone(),
        &binding.donor_git_blob_sha1.clone(),
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("byte mismatch"));
}

#[test]
fn durable_digest_field_mismatch_fails() {
    let bytes = b"synthetic-durable-bytes-not-the-historical-artifact";
    let mut binding = trusted();
    binding.recorded_sha256 = sha256_hex(bytes);
    let packet = packet_with(
        &binding,
        "HISTORICAL_ARTIFACT_BYTES_VERIFIED",
        json!("0000000000000000000000000000000000000000000000000000000000000000"),
    );
    let error = evaluate_durable_packet(
        &packet,
        &binding,
        Some(bytes),
        &binding.manifest_git_blob_sha1.clone(),
        &binding.donor_git_blob_sha1.clone(),
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("digest mismatch"));
}

#[test]
fn manifest_git_blob_mismatch_fails() {
    let binding = trusted();
    let error = evaluate_durable_packet(
        &historical_packet(),
        &binding,
        None,
        "0000000000000000000000000000000000000000",
        &binding.donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("manifest Git blob mismatch"));
}

#[test]
fn donor_git_blob_mismatch_fails() {
    let binding = trusted();
    let error = evaluate_durable_packet(
        &historical_packet(),
        &binding,
        None,
        &binding.manifest_git_blob_sha1,
        "0000000000000000000000000000000000000000",
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("donor Git blob mismatch"));
}

#[test]
fn candidate_edited_constants_cannot_self_authorize() {
    let mut value: Value = serde_json::from_slice(&historical_packet()).unwrap();
    value["artifact_id"] = json!(1);
    let error = evaluate_durable_packet(
        &serde_json::to_vec(&value).unwrap(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("wrong artifact id"));
}

#[test]
fn malformed_packet_fails() {
    let error = evaluate_durable_packet(
        b"{\"schema\":\"commandf.durable-retained-authority/v1\",\"schema\":1}",
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("malformed"));
}

#[test]
fn partial_packet_fails() {
    let mut value: Value = serde_json::from_slice(&historical_packet()).unwrap();
    value.as_object_mut().unwrap().remove("workflow_run_id");
    let error = evaluate_durable_packet(
        &serde_json::to_vec(&value).unwrap(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("partial"));
}

#[test]
fn wrong_workflow_identity_fails() {
    let mut value: Value = serde_json::from_slice(&historical_packet()).unwrap();
    value["workflow_run_id"] = json!(1);
    let error = evaluate_durable_packet(
        &serde_json::to_vec(&value).unwrap(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("wrong workflow run"));
}

#[test]
fn wrong_retained_head_or_base_fails() {
    let mut head = serde_json::from_slice::<Value>(&historical_packet()).unwrap();
    head["retained_head"] = json!("0000000000000000000000000000000000000000");
    let error = evaluate_durable_packet(
        &serde_json::to_vec(&head).unwrap(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("wrong retained head"));

    let mut base = serde_json::from_slice::<Value>(&historical_packet()).unwrap();
    base["retained_base"] = json!("0000000000000000000000000000000000000000");
    let error = evaluate_durable_packet(
        &serde_json::to_vec(&base).unwrap(),
        &trusted(),
        None,
        &trusted().manifest_git_blob_sha1,
        &trusted().donor_git_blob_sha1,
        LiveArtifactObservation::AbsentEmpty,
    )
    .unwrap_err();
    assert!(error.to_string().contains("wrong retained base"));
}

#[test]
fn offline_replay_is_byte_identical_and_repeated() {
    let first = evaluate_historical(LiveArtifactObservation::AbsentEmpty);
    let second = evaluate_historical(LiveArtifactObservation::NotConsulted);
    let first_bytes = serde_json::to_vec(&first).unwrap();
    let repeated =
        serde_json::to_vec(&evaluate_historical(LiveArtifactObservation::AbsentEmpty)).unwrap();
    assert_eq!(first_bytes, repeated);
    assert_eq!(first.offline_replay_state, "OFFLINE_REPLAY_VERIFIED");
    assert_ne!(
        first.live_github_artifact_state,
        second.live_github_artifact_state
    );
}

#[test]
fn cf10_six_state_projection_snapshot_remains_unchanged() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/016-af-02-adversarial-test-strength/authority-baseline.json");
    let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let cf10 = &value["cf10"];
    assert_eq!(cf10["retained_run_conclusion"], "failure");
    assert_eq!(cf10["retained_artifact_id"].as_u64(), Some(9_255_732_702));
    assert_eq!(
        cf10["retained_artifact_sha256"],
        "9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612"
    );
    assert_eq!(cf10["deltas"].as_array().unwrap().len(), 3);
    let states = cf10["states"].as_array().unwrap();
    assert_eq!(states.len(), 6);
    let ids: Vec<&str> = states
        .iter()
        .map(|state| state["state_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "C001-after",
            "C001-before",
            "C002-after",
            "C002-before",
            "C003-after",
            "C003-before",
        ]
    );
}

#[test]
fn nonmatching_archive_is_not_historical_recovery() {
    let class = classify_regenerated_archive(
        b"reconstructed-but-not-the-historical-zip",
        "9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612",
    );
    assert_eq!(
        class,
        RegenerationClass::ReconstructedSemanticEquivalentCandidate
    );
    let matching = b"exact-synthetic";
    assert_eq!(
        classify_regenerated_archive(matching, &sha256_hex(matching)),
        RegenerationClass::HistoricalBytesVerified
    );
}
