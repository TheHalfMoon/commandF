use commandf_af02_verifier::waiver::{parse_waiver_policy, WaiverError};

const POLICY_BASE: &str = "2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1";

#[test]
fn parse_waiver_policy_accepts_valid_zero_waiver_policy() {
    let input = format!(
        r#"{{"schema":"commandf.af02-waiver-policy/v1","policy_base_sha":"{POLICY_BASE}","waivers":[]}}"#
    );

    let policy = parse_waiver_policy(input.as_bytes()).expect("valid waiver policy must parse");

    assert_eq!(policy.schema, "commandf.af02-waiver-policy/v1");
    assert_eq!(policy.policy_base_sha, POLICY_BASE);
    assert!(policy.waivers.is_empty());
}

#[test]
fn parse_waiver_policy_rejects_duplicate_keys() {
    let input = format!(
        r#"{{"schema":"commandf.af02-waiver-policy/v1","schema":"commandf.af02-waiver-policy/v1","policy_base_sha":"{POLICY_BASE}","waivers":[]}}"#
    );

    let error = parse_waiver_policy(input.as_bytes()).expect_err("duplicate keys must fail closed");

    assert!(matches!(error, WaiverError::Json(_)));
}

#[test]
fn parse_waiver_policy_rejects_unknown_fields() {
    let input = format!(
        r#"{{"schema":"commandf.af02-waiver-policy/v1","policy_base_sha":"{POLICY_BASE}","waivers":[],"unexpected":true}}"#
    );

    let error = parse_waiver_policy(input.as_bytes()).expect_err("unknown fields must fail closed");

    assert!(matches!(error, WaiverError::Json(_)));
}
