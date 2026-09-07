use std::path::PathBuf;
use std::process::{Command, Output};

fn verifier() -> &'static str {
    env!("CARGO_BIN_EXE_commandf-af02-verifier")
}

fn contract_paths() -> (PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    (
        root.join("specs/016-af-02-adversarial-test-strength/semantic-contract.json"),
        root.join(
            "specs/016-af-02-adversarial-test-strength/schemas/af02-semantic-contract-v1.schema.json",
        ),
    )
}

fn run_semantic_contract() -> Output {
    let (contract, schema) = contract_paths();
    Command::new(verifier())
        .arg("verify-semantic-contract")
        .arg(contract)
        .arg(schema)
        .output()
        .expect("run semantic-contract verifier")
}

#[test]
fn verify_semantic_contract_cli_succeeds_deterministically() {
    let first = run_semantic_contract();
    assert!(first.status.success(), "stderr: {}", String::from_utf8_lossy(&first.stderr));
    let second = run_semantic_contract();
    assert!(second.status.success(), "stderr: {}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(first.stdout, second.stdout);

    let value: serde_json::Value = serde_json::from_slice(&first.stdout).expect("parse verifier output");
    assert_eq!(value["schema"], "commandf.af02-semantic-contract-validation/v1");
    assert_eq!(value["algorithm_count"], 25);
    assert_eq!(value["negative_fixture_count"], 72);
}

#[test]
fn unknown_cli_entrypoint_fails_closed() {
    let output = Command::new(verifier())
        .arg("not-a-command")
        .output()
        .expect("run verifier with unknown entrypoint");
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown entrypoint not-a-command"));
}
