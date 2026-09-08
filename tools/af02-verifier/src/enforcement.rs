use std::collections::BTreeSet;

use commandf_af02_verifier::canonical::{git_blob_sha1_hex, parse_json_no_duplicates, sha256_hex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::semantic::{validate_enforcement_inventory_closure, EnforcementRole, SemanticError};

const ENFORCEMENT_SCHEMA_ID: &str = "commandf.af02-enforcement-inventory/v1";
const ENFORCEMENT_SCHEMA_URL: &str =
    "https://commandf.dev/schemas/af02-enforcement-inventory-v1.schema.json";
const ENFORCEMENT_SCHEMA_GIT_BLOB_SHA: &str = "19e6e591d021992db8fccaaf9249d99e82635f2d";
const POLICY_STATUS: &str = "PLANNING_FREEZE";
const CLOSURE_RULE: &str = "EXACT_ROLE_SET_NO_DUPLICATES_ACTIVE_PATHS_MUST_RESOLVE_AT_OR_AFTER_REQUIRED_STACK";
const ROLE_COUNT: usize = 27;

#[derive(Debug, Error)]
pub enum EnforcementError {
    #[error("enforcement-inventory JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("enforcement-inventory semantic error: {0}")]
    Semantic(#[from] SemanticError),
    #[error("enforcement-inventory contract violation: {0}")]
    Contract(String),
    #[error("trusted canonical-base resolver failure: {0}")]
    Resolver(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementInventory {
    pub schema: String,
    pub policy_status: String,
    pub entries: Vec<EnforcementInventoryEntry>,
    pub closure_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementInventoryEntry {
    pub role: String,
    pub required_from_stack: String,
    pub implementation_kind: String,
    pub planned_path: String,
    pub entrypoint: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VerifiedEnforcementInventory {
    pub schema: String,
    pub current_stack: String,
    pub role_count: usize,
    pub active_role_count: usize,
    pub inventory_sha256: String,
    pub schema_git_blob_sha: String,
}

pub trait CanonicalEnforcementResolver {
    fn resolves_on_canonical_base(
        &self,
        entry: &EnforcementInventoryEntry,
    ) -> Result<bool, String>;
}

pub fn parse_frozen_enforcement_inventory(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<EnforcementInventory, EnforcementError> {
    verify_schema_identity(schema_bytes)?;
    let schema_value = parse_json_no_duplicates(schema_bytes)?;
    if schema_value.get("$id").and_then(Value::as_str) != Some(ENFORCEMENT_SCHEMA_URL) {
        return contract_error("unexpected enforcement-inventory schema URL");
    }

    let frozen_entries_value = schema_value
        .pointer("/properties/entries/const")
        .cloned()
        .ok_or_else(|| EnforcementError::Contract("schema is missing entries.const".to_owned()))?;
    let frozen_entries: Vec<EnforcementInventoryEntry> =
        serde_json::from_value(frozen_entries_value)?;
    if frozen_entries.len() != ROLE_COUNT {
        return contract_error(format!(
            "schema freezes {} enforcement roles instead of {ROLE_COUNT}",
            frozen_entries.len()
        ));
    }

    let instance_value = parse_json_no_duplicates(instance_bytes)?;
    let inventory: EnforcementInventory = serde_json::from_value(instance_value)?;
    if inventory.schema != ENFORCEMENT_SCHEMA_ID {
        return contract_error(format!(
            "unexpected enforcement-inventory schema id {}",
            inventory.schema
        ));
    }
    if inventory.policy_status != POLICY_STATUS {
        return contract_error(format!(
            "unexpected enforcement-inventory policy status {}",
            inventory.policy_status
        ));
    }
    if inventory.closure_rule != CLOSURE_RULE {
        return contract_error(format!(
            "unexpected enforcement-inventory closure rule {}",
            inventory.closure_rule
        ));
    }
    if inventory.entries != frozen_entries {
        return contract_error("enforcement inventory differs from schema-frozen role topology");
    }

    let mut roles = BTreeSet::new();
    for entry in &inventory.entries {
        if !roles.insert(entry.role.as_str()) {
            return contract_error(format!("duplicate enforcement role {}", entry.role));
        }
        stack_rank(&entry.required_from_stack)?;
        if entry.planned_path.is_empty() || entry.entrypoint.is_empty() {
            return contract_error(format!(
                "enforcement role {} has an empty planned path or entrypoint",
                entry.role
            ));
        }
    }
    if roles.len() != ROLE_COUNT {
        return contract_error(format!(
            "enforcement inventory contains {} unique roles instead of {ROLE_COUNT}",
            roles.len()
        ));
    }

    Ok(inventory)
}

pub fn verify_enforcement_inventory<R: CanonicalEnforcementResolver>(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
    current_stack: &str,
    resolver: &R,
) -> Result<VerifiedEnforcementInventory, EnforcementError> {
    let inventory = parse_frozen_enforcement_inventory(instance_bytes, schema_bytes)?;
    let current_rank = stack_rank(current_stack)?;
    let expected_roles = inventory
        .entries
        .iter()
        .map(|entry| entry.role.clone())
        .collect::<Vec<_>>();

    let mut active_role_count = 0usize;
    let mut semantic_inventory = Vec::with_capacity(inventory.entries.len());
    for entry in &inventory.entries {
        let active = stack_rank(&entry.required_from_stack)? <= current_rank;
        let resolved_on_base = if active {
            active_role_count += 1;
            resolver
                .resolves_on_canonical_base(entry)
                .map_err(EnforcementError::Resolver)?
        } else {
            false
        };

        // The planning-frozen topology intentionally allows one implementation module to own
        // multiple enforcement roles. The T021 semantic helper predates that activation binding
        // and treats its `planned_path` field as a uniqueness key. Feed it the already-resolved
        // path+entrypoint authority identity while the trusted resolver above validates the real
        // frozen path and entrypoint independently.
        let semantic_authority_key = format!("{}#{}", entry.planned_path, entry.entrypoint);
        semantic_inventory.push(EnforcementRole {
            role: entry.role.clone(),
            planned_path: semantic_authority_key,
            entrypoint: entry.entrypoint.clone(),
            required_from_stack: entry.required_from_stack.clone(),
            resolved_on_base,
        });
    }

    validate_enforcement_inventory_closure(
        &expected_roles,
        &semantic_inventory,
        current_stack,
    )?;

    Ok(VerifiedEnforcementInventory {
        schema: "commandf.af02-enforcement-inventory-verification/v1".to_owned(),
        current_stack: current_stack.to_owned(),
        role_count: inventory.entries.len(),
        active_role_count,
        inventory_sha256: sha256_hex(instance_bytes),
        schema_git_blob_sha: ENFORCEMENT_SCHEMA_GIT_BLOB_SHA.to_owned(),
    })
}

fn verify_schema_identity(schema_bytes: &[u8]) -> Result<(), EnforcementError> {
    let observed = git_blob_sha1_hex(schema_bytes);
    if observed != ENFORCEMENT_SCHEMA_GIT_BLOB_SHA {
        return contract_error(format!(
            "enforcement-inventory schema Git blob {observed} differs from frozen {ENFORCEMENT_SCHEMA_GIT_BLOB_SHA}"
        ));
    }
    Ok(())
}

fn stack_rank(value: &str) -> Result<u8, EnforcementError> {
    match value {
        "A0" => Ok(0),
        "A1" => Ok(1),
        "B0" => Ok(2),
        "B1" => Ok(3),
        "C0" => Ok(4),
        "C1" => Ok(5),
        other => contract_error(format!("unknown AF-02 stack {other}")),
    }
}

fn contract_error<T>(message: impl Into<String>) -> Result<T, EnforcementError> {
    Err(EnforcementError::Contract(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVENTORY_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/enforcement-inventory.json"
    );
    const SCHEMA_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-enforcement-inventory-v1.schema.json"
    );

    #[derive(Default)]
    struct Resolver {
        missing: BTreeSet<String>,
    }

    impl Resolver {
        fn missing(role: &str) -> Self {
            Self {
                missing: BTreeSet::from([role.to_owned()]),
            }
        }
    }

    impl CanonicalEnforcementResolver for Resolver {
        fn resolves_on_canonical_base(
            &self,
            entry: &EnforcementInventoryEntry,
        ) -> Result<bool, String> {
            Ok(!self.missing.contains(&entry.role))
        }
    }

    #[test]
    fn parses_exact_schema_frozen_inventory_with_shared_module_paths() {
        let inventory = parse_frozen_enforcement_inventory(INVENTORY_BYTES, SCHEMA_BYTES).unwrap();
        assert_eq!(inventory.entries.len(), ROLE_COUNT);
        let surface_roles = inventory
            .entries
            .iter()
            .filter(|entry| entry.planned_path == "tools/af02-verifier/src/surface.rs")
            .map(|entry| entry.role.as_str())
            .collect::<Vec<_>>();
        assert_eq!(surface_roles, ["SURFACE_SCANNER", "SURFACE_POLICY_PARSER"]);
    }

    #[test]
    fn verifies_a0_activation_with_schema_frozen_shared_paths() {
        let verified = verify_enforcement_inventory(
            INVENTORY_BYTES,
            SCHEMA_BYTES,
            "A0",
            &Resolver::default(),
        )
        .unwrap();
        assert_eq!(verified.role_count, 27);
        assert_eq!(verified.active_role_count, 16);
        assert_eq!(verified.current_stack, "A0");
    }

    #[test]
    fn rejects_missing_active_a0_authority() {
        let error = verify_enforcement_inventory(
            INVENTORY_BYTES,
            SCHEMA_BYTES,
            "A0",
            &Resolver::missing("VERIFIER_INPUT_GUARD"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("VERIFIER_INPUT_GUARD"));
        assert!(error.to_string().contains("unresolved"));
    }

    #[test]
    fn future_roles_do_not_activate_before_their_stack() {
        verify_enforcement_inventory(
            INVENTORY_BYTES,
            SCHEMA_BYTES,
            "A0",
            &Resolver::missing("REPLAY_RUNNER"),
        )
        .unwrap();
        let error = verify_enforcement_inventory(
            INVENTORY_BYTES,
            SCHEMA_BYTES,
            "A1",
            &Resolver::missing("REPLAY_RUNNER"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("REPLAY_RUNNER"));
    }

    #[test]
    fn rejects_inventory_that_differs_from_schema_frozen_topology() {
        let mut value = parse_json_no_duplicates(INVENTORY_BYTES).unwrap();
        value["entries"][0]["entrypoint"] = Value::String("candidate-defined-entrypoint".to_owned());
        let tampered = serde_json::to_vec(&value).unwrap();
        let error = parse_frozen_enforcement_inventory(&tampered, SCHEMA_BYTES).unwrap_err();
        assert!(error.to_string().contains("schema-frozen role topology"));
    }

    #[test]
    fn rejects_modified_schema_bytes_even_when_json_remains_valid() {
        let mut schema = SCHEMA_BYTES.to_vec();
        schema.push(b'\n');
        let error = parse_frozen_enforcement_inventory(INVENTORY_BYTES, &schema).unwrap_err();
        assert!(error.to_string().contains("schema Git blob"));
    }

    #[test]
    fn rejects_unknown_stack() {
        let error = verify_enforcement_inventory(
            INVENTORY_BYTES,
            SCHEMA_BYTES,
            "D0",
            &Resolver::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("unknown AF-02 stack D0"));
    }
}
