use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    check::validate_check_report, CheckReport, SourceIndexEvidence, SourceLocation, SourceMapError,
    SourceMappedCheckReport, SourceMappingEntry, SourceMappingStatus,
};

pub const MAX_SUSHI_INDEX_ENTRIES: usize = 100_000;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SushiFshIndexEntry {
    output_file: String,
    fsh_file: String,
    fsh_name: String,
    fsh_type: String,
    start_line: u32,
    end_line: u32,
}

#[derive(Clone, Debug)]
struct ValidatedSushiEntry {
    fsh_file: PathBuf,
    start_line: u32,
    end_line: u32,
}

pub fn build_source_mapped_check_report(
    report: &CheckReport,
    index_bytes: &[u8],
    repo_root: &Path,
    fsh_root: &Path,
) -> Result<SourceMappedCheckReport, SourceMapError> {
    validate_check_report(report)?;

    let repo_root = fs::canonicalize(repo_root)?;
    if !repo_root.is_dir() {
        return Err(SourceMapError::InvalidPath(format!(
            "repository root is not a directory: {}",
            repo_root.display()
        )));
    }

    let fsh_root_text = fsh_root.to_str().ok_or_else(|| {
        SourceMapError::InvalidPath("FSH root must be valid UTF-8 in CF-09 V1".to_owned())
    })?;
    let fsh_root_relative = portable_relative_path(fsh_root_text, "FSH root", true)?;
    let fsh_root_canonical = fs::canonicalize(repo_root.join(&fsh_root_relative))?;
    if !fsh_root_canonical.is_dir() || !fsh_root_canonical.starts_with(&repo_root) {
        return Err(SourceMapError::SourceEscape(format!(
            "FSH root escapes repository root: {}",
            fsh_root.display()
        )));
    }

    let index = parse_sushi_index(index_bytes)?;
    let mut mappings = Vec::with_capacity(report.compatibility.findings.len());

    for (finding_index, finding) in report.compatibility.findings.iter().enumerate() {
        let Some(after_filename) = finding.after_filename.as_deref() else {
            mappings.push(SourceMappingEntry {
                finding_index,
                status: SourceMappingStatus::UnmappedNoAfterFilename,
                location: None,
            });
            continue;
        };

        let Some(entry) = index.get(after_filename) else {
            mappings.push(SourceMappingEntry {
                finding_index,
                status: SourceMappingStatus::UnmappedNoIndexEntry,
                location: None,
            });
            continue;
        };

        let source_canonical = fs::canonicalize(fsh_root_canonical.join(&entry.fsh_file))
            .map_err(|_| SourceMapError::MissingSource(entry.fsh_file.display().to_string()))?;
        if !source_canonical.starts_with(&fsh_root_canonical)
            || !source_canonical.starts_with(&repo_root)
        {
            return Err(SourceMapError::SourceEscape(
                entry.fsh_file.display().to_string(),
            ));
        }
        if !fs::metadata(&source_canonical)?.is_file() {
            return Err(SourceMapError::MissingSource(
                entry.fsh_file.display().to_string(),
            ));
        }

        let repo_relative = source_canonical
            .strip_prefix(&repo_root)
            .map_err(|_| SourceMapError::SourceEscape(entry.fsh_file.display().to_string()))?;
        let file = relative_path_to_slash(repo_relative)?;

        mappings.push(SourceMappingEntry {
            finding_index,
            status: SourceMappingStatus::Mapped,
            location: Some(SourceLocation {
                file,
                line: entry.start_line,
                end_line: entry.end_line,
            }),
        });
    }

    let report = SourceMappedCheckReport {
        schema: SourceMappedCheckReport::SCHEMA_V1,
        source_index: SourceIndexEvidence {
            format: SourceMappedCheckReport::SOURCE_INDEX_FORMAT_V1.to_owned(),
            sha256: sha256_hex(index_bytes),
            entries: index.len(),
            fsh_root: relative_path_to_slash(
                fsh_root_canonical
                    .strip_prefix(&repo_root)
                    .map_err(|_| SourceMapError::SourceEscape(fsh_root.display().to_string()))?,
            )?,
        },
        check: report.clone(),
        mappings,
    };
    validate_source_mapped_check_report(&report)?;
    Ok(report)
}

pub fn validate_source_mapped_check_report(
    report: &SourceMappedCheckReport,
) -> Result<(), SourceMapError> {
    if report.schema != SourceMappedCheckReport::SCHEMA_V1 {
        return Err(SourceMapError::UnsupportedSchema {
            found: report.schema,
            expected: SourceMappedCheckReport::SCHEMA_V1,
        });
    }
    if report.source_index.format != SourceMappedCheckReport::SOURCE_INDEX_FORMAT_V1 {
        return Err(SourceMapError::UnsupportedSourceIndexFormat {
            found: report.source_index.format.clone(),
            expected: SourceMappedCheckReport::SOURCE_INDEX_FORMAT_V1.to_owned(),
        });
    }
    if report.source_index.sha256.len() != 64
        || !report
            .source_index
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(SourceMapError::InvalidIndex(
            "source-index sha256 must be 64 hexadecimal characters".to_owned(),
        ));
    }
    let serialized_fsh_root =
        portable_relative_path(&report.source_index.fsh_root, "serialized FSH root", true)?;
    validate_check_report(&report.check)?;

    let expected = report.check.compatibility.findings.len();
    if report.mappings.len() != expected {
        return Err(SourceMapError::FindingCountMismatch {
            found: report.mappings.len(),
            expected,
        });
    }

    for (expected_index, mapping) in report.mappings.iter().enumerate() {
        if mapping.finding_index != expected_index {
            return Err(SourceMapError::FindingIndexMismatch {
                found: mapping.finding_index,
                expected: expected_index,
            });
        }
        match (mapping.status, mapping.location.as_ref()) {
            (SourceMappingStatus::Mapped, Some(location)) => {
                let mapped_path =
                    portable_relative_path(&location.file, "mapped repository path", false)?;
                if !serialized_fsh_root.as_os_str().is_empty()
                    && !mapped_path.starts_with(&serialized_fsh_root)
                {
                    return Err(SourceMapError::InvalidMappingEntry {
                        index: expected_index,
                    });
                }
                if location.line == 0 || location.end_line == 0 || location.line > location.end_line
                {
                    return Err(SourceMapError::InvalidMappingEntry {
                        index: expected_index,
                    });
                }
            }
            (
                SourceMappingStatus::UnmappedNoAfterFilename
                | SourceMappingStatus::UnmappedNoIndexEntry,
                None,
            ) => {}
            _ => {
                return Err(SourceMapError::InvalidMappingEntry {
                    index: expected_index,
                });
            }
        }
    }
    Ok(())
}

fn parse_sushi_index(
    bytes: &[u8],
) -> Result<BTreeMap<String, ValidatedSushiEntry>, SourceMapError> {
    let entries: Vec<SushiFshIndexEntry> = serde_json::from_slice(bytes)
        .map_err(|error| SourceMapError::InvalidIndex(error.to_string()))?;
    if entries.len() > MAX_SUSHI_INDEX_ENTRIES {
        return Err(SourceMapError::TooManyEntries {
            found: entries.len(),
            maximum: MAX_SUSHI_INDEX_ENTRIES,
        });
    }

    let mut index = BTreeMap::new();
    for entry in entries {
        if entry.output_file.is_empty() || entry.fsh_name.is_empty() || entry.fsh_type.is_empty() {
            return Err(SourceMapError::InvalidIndex(
                "outputFile, fshName, and fshType must be non-empty strings".to_owned(),
            ));
        }
        if entry.output_file.contains('/')
            || entry.output_file.contains('\\')
            || matches!(entry.output_file.as_str(), "." | "..")
        {
            return Err(SourceMapError::InvalidIndex(format!(
                "outputFile must be one generated package-root filename: {}",
                entry.output_file
            )));
        }
        if entry.start_line == 0 || entry.end_line == 0 || entry.start_line > entry.end_line {
            return Err(SourceMapError::InvalidIndex(format!(
                "invalid line range for {}: {}..{}",
                entry.output_file, entry.start_line, entry.end_line
            )));
        }
        let fsh_file = portable_relative_path(&entry.fsh_file, "SUSHI fshFile", false)?;
        let validated = ValidatedSushiEntry {
            fsh_file,
            start_line: entry.start_line,
            end_line: entry.end_line,
        };
        if index.insert(entry.output_file.clone(), validated).is_some() {
            return Err(SourceMapError::DuplicateOutputFile(entry.output_file));
        }
    }
    Ok(index)
}

fn portable_relative_path(
    value: &str,
    label: &str,
    allow_dot: bool,
) -> Result<PathBuf, SourceMapError> {
    if value.is_empty() {
        return Err(SourceMapError::InvalidPath(format!(
            "{label} must not be empty"
        )));
    }
    if allow_dot && value == "." {
        return Ok(PathBuf::new());
    }

    let normalized = value.replace('\\', "/");
    if normalized.starts_with('/')
        || normalized.starts_with("//")
        || normalized
            .as_bytes()
            .get(1)
            .is_some_and(|byte| *byte == b':')
    {
        return Err(SourceMapError::InvalidPath(format!(
            "{label} must be repository-relative: {value}"
        )));
    }

    let mut path = PathBuf::new();
    for component in normalized.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(SourceMapError::InvalidPath(format!(
                "{label} contains an invalid path component: {value}"
            )));
        }
        path.push(component);
    }
    Ok(path)
}

fn relative_path_to_slash(path: &Path) -> Result<String, SourceMapError> {
    if path.as_os_str().is_empty() {
        return Ok(".".to_owned());
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or_else(|| {
                        SourceMapError::InvalidPath(
                            "repository-relative source path must be UTF-8 in CF-09 V1".to_owned(),
                        )
                    })?
                    .to_owned(),
            ),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SourceMapError::InvalidPath(format!(
                    "source path is not repository-relative: {}",
                    path.display()
                )));
            }
        }
    }
    if parts.is_empty() {
        Ok(".".to_owned())
    } else {
        Ok(parts.join("/"))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
