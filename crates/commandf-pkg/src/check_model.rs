use serde::{Deserialize, Serialize};

use crate::CompatibilityReport;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckDirection {
    Both,
    Producer,
    Consumer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckFailOn {
    Breaking,
    Risky,
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckPolicy {
    pub direction: CheckDirection,
    pub fail_on: CheckFailOn,
}

impl Default for CheckPolicy {
    fn default() -> Self {
        Self {
            direction: CheckDirection::Both,
            fail_on: CheckFailOn::Breaking,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckDecision {
    pub passed: bool,
    pub total_findings: usize,
    pub selected_findings: usize,
    pub breaking_findings: usize,
    pub risky_findings: usize,
    pub additive_findings: usize,
    pub blocking_findings: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CheckReport {
    pub schema: u32,
    pub policy: CheckPolicy,
    pub decision: CheckDecision,
    pub compatibility: CompatibilityReport,
}

impl CheckReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
