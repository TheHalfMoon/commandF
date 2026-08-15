use serde::{Deserialize, Serialize};

use crate::{
    source_map::MAX_SOURCE_MAPPED_REPORT_BYTES, source_map_error::SourceMapError, CheckReport,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceIndexEvidence {
    pub format: String,
    pub sha256: String,
    pub entries: usize,
    pub fsh_root: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub end_line: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMappingStatus {
    Mapped,
    UnmappedNoAfterFilename,
    UnmappedNoIndexEntry,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceMappingEntry {
    pub finding_index: usize,
    pub status: SourceMappingStatus,
    pub location: Option<SourceLocation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceMappedCheckReport {
    pub schema: u32,
    pub source_index: SourceIndexEvidence,
    pub check: CheckReport,
    pub mappings: Vec<SourceMappingEntry>,
}

impl SourceMappedCheckReport {
    pub const SCHEMA_V1: u32 = 1;
    pub const SOURCE_INDEX_FORMAT_V1: &'static str = "sushi-fsh-index/v1";

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, SourceMapError> {
        ensure_source_map_input_size(bytes.len(), MAX_SOURCE_MAPPED_REPORT_BYTES)?;
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

fn ensure_source_map_input_size(found: usize, maximum: usize) -> Result<(), SourceMapError> {
    if found > maximum {
        return Err(SourceMapError::ReportTooLarge { found, maximum });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_source_map_input_size_limit_is_inclusive() {
        assert!(ensure_source_map_input_size(
            MAX_SOURCE_MAPPED_REPORT_BYTES,
            MAX_SOURCE_MAPPED_REPORT_BYTES,
        )
        .is_ok());
        assert!(matches!(
            ensure_source_map_input_size(
                MAX_SOURCE_MAPPED_REPORT_BYTES + 1,
                MAX_SOURCE_MAPPED_REPORT_BYTES,
            ),
            Err(SourceMapError::ReportTooLarge {
                found,
                maximum: MAX_SOURCE_MAPPED_REPORT_BYTES,
            }) if found == MAX_SOURCE_MAPPED_REPORT_BYTES + 1
        ));
    }
}
