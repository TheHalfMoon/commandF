use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::canonical::parse_json_no_duplicates;

const WAIVER_POLICY_SCHEMA: &str = "commandf.af02-waiver-policy/v1";
const ALLOWED_RESULT_CLASS: &str = "WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE";
const REVIEW_URL_PREFIX: &str = "https://github.com/TheHalfMoon/commandF/pull/";

#[derive(Debug, Error)]
pub enum WaiverError {
    #[error("waiver policy JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("waiver policy violation: {0}")]
    Policy(String),
    #[error("waiver ancestry resolution failed: {0}")]
    Ancestry(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaiverPolicy {
    pub schema: String,
    pub policy_base_sha: String,
    pub waivers: Vec<Waiver>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Waiver {
    pub waiver_id: String,
    pub mutant_id: String,
    pub allowed_result_class: String,
    pub target_source_path: String,
    pub rationale: String,
    pub evidence_sha256: String,
    pub introduced_by_pr: u64,
    pub introduced_policy_sha: String,
    pub review_url: String,
}

pub trait CanonicalWaiverResolver {
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, WaiverError>;

    fn policy_bytes_at(&self, commit_sha: &str) -> Result<Option<Vec<u8>>, WaiverError>;
}

pub fn parse_waiver_policy(bytes: &[u8]) -> Result<WaiverPolicy, WaiverError> {
    let value = parse_json_no_duplicates(bytes)?;
    let policy: WaiverPolicy = serde_json::from_value(value)?;
    validate_waiver_policy(&policy)?;
    Ok(policy)
}

pub fn verify_waiver_canonical_ancestry<R: CanonicalWaiverResolver>(
    candidate_policy: &WaiverPolicy,
    canonical_base_sha: &str,
    resolver: &R,
) -> Result<(), WaiverError> {
    validate_waiver_policy(candidate_policy)?;
    validate_git_sha(canonical_base_sha, "canonical_base_sha")?;

    if !is_ancestor_or_equal(resolver, &candidate_policy.policy_base_sha, canonical_base_sha)? {
        return ancestry_error(format!(
            "policy_base_sha {} is not an ancestor of canonical base {canonical_base_sha}",
            candidate_policy.policy_base_sha
        ));
    }

    let canonical_base_bytes = resolver.policy_bytes_at(canonical_base_sha)?.ok_or_else(|| {
        WaiverError::Ancestry(format!(
            "waiver policy is absent at canonical base {canonical_base_sha}"
        ))
    })?;
    let canonical_base_policy = parse_waiver_policy(&canonical_base_bytes)?;
    if candidate_policy.policy_base_sha != canonical_base_policy.policy_base_sha {
        return ancestry_error(format!(
            "candidate policy_base_sha {} differs from canonical-base policy_base_sha {}",
            candidate_policy.policy_base_sha, canonical_base_policy.policy_base_sha
        ));
    }

    for waiver in &candidate_policy.waivers {
        let canonical_match = canonical_base_policy
            .waivers
            .iter()
            .filter(|candidate| candidate.waiver_id == waiver.waiver_id)
            .collect::<Vec<_>>();
        if canonical_match.len() != 1 {
            return ancestry_error(format!(
                "waiver {} does not resolve uniquely in canonical-base waiver policy",
                waiver.waiver_id
            ));
        }
        if canonical_match[0] != waiver {
            return ancestry_error(format!(
                "waiver {} differs from the canonical-base waiver entry",
                waiver.waiver_id
            ));
        }

        if !is_ancestor_or_equal(resolver, &waiver.introduced_policy_sha, canonical_base_sha)? {
            return ancestry_error(format!(
                "waiver {} introduced_policy_sha {} is not canonical before the candidate",
                waiver.waiver_id, waiver.introduced_policy_sha
            ));
        }

        let introduced_bytes = resolver
            .policy_bytes_at(&waiver.introduced_policy_sha)?
            .ok_or_else(|| {
                WaiverError::Ancestry(format!(
                    "waiver policy is absent at introduced_policy_sha {} for {}",
                    waiver.introduced_policy_sha, waiver.waiver_id
                ))
            })?;
        let introduced_policy = parse_waiver_policy(&introduced_bytes)?;
        let introduced_match = introduced_policy
            .waivers
            .iter()
            .filter(|candidate| candidate.waiver_id == waiver.waiver_id)
            .collect::<Vec<_>>();
        if introduced_match.len() != 1 || introduced_match[0] != waiver {
            return ancestry_error(format!(
                "waiver {} was not already canonical with identical authority at introduced_policy_sha {}",
                waiver.waiver_id, waiver.introduced_policy_sha
            ));
        }
    }

    Ok(())
}

fn validate_waiver_policy(policy: &WaiverPolicy) -> Result<(), WaiverError> {
    if policy.schema != WAIVER_POLICY_SCHEMA {
        return policy_error(format!("unexpected waiver policy schema {}", policy.schema));
    }
    validate_git_sha(&policy.policy_base_sha, "policy_base_sha")?;

    let mut waiver_ids = BTreeSet::new();
    let mut mutant_ids = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for waiver in &policy.waivers {
        if !valid_waiver_id(&waiver.waiver_id) {
            return policy_error(format!("invalid waiver_id {}", waiver.waiver_id));
        }
        validate_sha256(&waiver.mutant_id, "mutant_id")?;
        if waiver.allowed_result_class != ALLOWED_RESULT_CLASS {
            return policy_error(format!(
                "waiver {} has unsupported allowed_result_class {}",
                waiver.waiver_id, waiver.allowed_result_class
            ));
        }
        validate_repo_path(&waiver.target_source_path, "target_source_path")?;
        let rationale_chars = waiver.rationale.chars().count();
        if !(20..=4096).contains(&rationale_chars) {
            return policy_error(format!(
                "waiver {} rationale must contain 20..=4096 Unicode scalar values",
                waiver.waiver_id
            ));
        }
        validate_sha256(&waiver.evidence_sha256, "evidence_sha256")?;
        if waiver.introduced_by_pr == 0 {
            return policy_error(format!(
                "waiver {} introduced_by_pr must be positive",
                waiver.waiver_id
            ));
        }
        validate_git_sha(&waiver.introduced_policy_sha, "introduced_policy_sha")?;
        let expected_review_url = format!("{REVIEW_URL_PREFIX}{}", waiver.introduced_by_pr);
        if waiver.review_url != expected_review_url {
            return policy_error(format!(
                "waiver {} review_url does not bind introduced_by_pr {}",
                waiver.waiver_id, waiver.introduced_by_pr
            ));
        }
        if !waiver_ids.insert(waiver.waiver_id.as_str()) {
            return policy_error(format!("duplicate waiver_id {}", waiver.waiver_id));
        }
        if !mutant_ids.insert(waiver.mutant_id.as_str()) {
            return policy_error(format!(
                "multiple waivers bind mutant_id {}",
                waiver.mutant_id
            ));
        }
        if previous_id.is_some_and(|previous| previous >= waiver.waiver_id.as_str()) {
            return policy_error("waivers must be strictly ordered by waiver_id");
        }
        previous_id = Some(waiver.waiver_id.as_str());
    }

    Ok(())
}

fn is_ancestor_or_equal<R: CanonicalWaiverResolver>(
    resolver: &R,
    ancestor: &str,
    descendant: &str,
) -> Result<bool, WaiverError> {
    if ancestor == descendant {
        return Ok(true);
    }
    resolver.is_ancestor(ancestor, descendant)
}

fn validate_git_sha(value: &str, label: &str) -> Result<(), WaiverError> {
    if value.len() != 40 || !value.bytes().all(is_lower_hex) {
        return policy_error(format!(
            "{label} must be 40 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), WaiverError> {
    if value.len() != 64 || !value.bytes().all(is_lower_hex) {
        return policy_error(format!(
            "{label} must be 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn valid_waiver_id(value: &str) -> bool {
    value.len() == 8
        && value.starts_with("AF02-W")
        && value.as_bytes()[6..].iter().all(u8::is_ascii_digit)
}

fn validate_repo_path(value: &str, label: &str) -> Result<(), WaiverError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains("//")
        || value.contains('\0')
        || value.split('/').any(|part| matches!(part, "." | ".."))
    {
        return policy_error(format!("invalid {label}: {value}"));
    }
    Ok(())
}

fn policy_error<T>(message: impl Into<String>) -> Result<T, WaiverError> {
    Err(WaiverError::Policy(message.into()))
}

fn ancestry_error<T>(message: impl Into<String>) -> Result<T, WaiverError> {
    Err(WaiverError::Ancestry(message.into()))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;

    const POLICY_BASE: &str = "2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1";
    const CANONICAL_BASE: &str = "beaddf19403a992385196aac18395c6c6e14ba0d";
    const INTRODUCED: &str = "1111111111111111111111111111111111111111";

    #[derive(Default)]
    struct TestResolver {
        ancestors: BTreeSet<(String, String)>,
        policies: BTreeMap<String, Vec<u8>>,
    }

    impl CanonicalWaiverResolver for TestResolver {
        fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, WaiverError> {
            Ok(self
                .ancestors
                .contains(&(ancestor.to_owned(), descendant.to_owned())))
        }

        fn policy_bytes_at(&self, commit_sha: &str) -> Result<Option<Vec<u8>>, WaiverError> {
            Ok(self.policies.get(commit_sha).cloned())
        }
    }

    fn empty_policy() -> WaiverPolicy {
        WaiverPolicy {
            schema: WAIVER_POLICY_SCHEMA.to_owned(),
            policy_base_sha: POLICY_BASE.to_owned(),
            waivers: Vec::new(),
        }
    }

    fn waiver() -> Waiver {
        Waiver {
            waiver_id: "AF02-W0001".to_owned(),
            mutant_id: "1".repeat(64),
            allowed_result_class: ALLOWED_RESULT_CLASS.to_owned(),
            target_source_path: "crates/commandf-pkg/src/lock.rs".to_owned(),
            rationale: "Equivalent mutant proven by canonical evidence.".to_owned(),
            evidence_sha256: "2".repeat(64),
            introduced_by_pr: 70,
            introduced_policy_sha: INTRODUCED.to_owned(),
            review_url: "https://github.com/TheHalfMoon/commandF/pull/70".to_owned(),
        }
    }

    fn bytes(policy: &WaiverPolicy) -> Vec<u8> {
        serde_json::to_vec(policy).unwrap()
    }

    fn resolver_with_base(base_policy: &WaiverPolicy) -> TestResolver {
        let mut resolver = TestResolver::default();
        resolver
            .ancestors
            .insert((POLICY_BASE.to_owned(), CANONICAL_BASE.to_owned()));
        resolver
            .policies
            .insert(CANONICAL_BASE.to_owned(), bytes(base_policy));
        resolver
    }

    #[test]
    fn accepts_zero_waiver_canonical_seed() {
        let candidate = empty_policy();
        let resolver = resolver_with_base(&candidate);
        verify_waiver_canonical_ancestry(&candidate, CANONICAL_BASE, &resolver).unwrap();
    }

    #[test]
    fn rejects_review_url_pr_mismatch() {
        let mut policy = empty_policy();
        let mut entry = waiver();
        entry.review_url = "https://github.com/TheHalfMoon/commandF/pull/71".to_owned();
        policy.waivers.push(entry);
        let error = validate_waiver_policy(&policy).unwrap_err();
        assert!(error.to_string().contains("review_url does not bind"));
    }

    #[test]
    fn rejects_duplicate_mutant_waiver() {
        let mut policy = empty_policy();
        let first = waiver();
        let mut second = first.clone();
        second.waiver_id = "AF02-W0002".to_owned();
        policy.waivers.extend([first, second]);
        let error = validate_waiver_policy(&policy).unwrap_err();
        assert!(error.to_string().contains("multiple waivers bind mutant_id"));
    }

    #[test]
    fn rejects_same_candidate_waiver_with_forged_old_sha() {
        let base_policy = empty_policy();
        let mut candidate = empty_policy();
        candidate.waivers.push(waiver());
        let mut resolver = resolver_with_base(&base_policy);
        resolver
            .ancestors
            .insert((INTRODUCED.to_owned(), CANONICAL_BASE.to_owned()));
        resolver
            .policies
            .insert(INTRODUCED.to_owned(), bytes(&base_policy));

        let error =
            verify_waiver_canonical_ancestry(&candidate, CANONICAL_BASE, &resolver).unwrap_err();
        assert!(error
            .to_string()
            .contains("does not resolve uniquely in canonical-base waiver policy"));
    }

    #[test]
    fn accepts_identical_precanonical_waiver() {
        let mut canonical_policy = empty_policy();
        canonical_policy.waivers.push(waiver());
        let mut resolver = resolver_with_base(&canonical_policy);
        resolver
            .ancestors
            .insert((INTRODUCED.to_owned(), CANONICAL_BASE.to_owned()));
        resolver
            .policies
            .insert(INTRODUCED.to_owned(), bytes(&canonical_policy));

        verify_waiver_canonical_ancestry(&canonical_policy, CANONICAL_BASE, &resolver).unwrap();
    }

    #[test]
    fn rejects_candidate_mutation_of_canonical_waiver() {
        let mut canonical_policy = empty_policy();
        canonical_policy.waivers.push(waiver());
        let mut candidate = canonical_policy.clone();
        candidate.waivers[0].rationale =
            "Changed candidate rationale cannot inherit canonical authority.".to_owned();
        let mut resolver = resolver_with_base(&canonical_policy);
        resolver
            .ancestors
            .insert((INTRODUCED.to_owned(), CANONICAL_BASE.to_owned()));
        resolver
            .policies
            .insert(INTRODUCED.to_owned(), bytes(&canonical_policy));

        let error =
            verify_waiver_canonical_ancestry(&candidate, CANONICAL_BASE, &resolver).unwrap_err();
        assert!(error
            .to_string()
            .contains("differs from the canonical-base waiver entry"));
    }
}
