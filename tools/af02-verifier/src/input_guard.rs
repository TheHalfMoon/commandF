use std::collections::BTreeMap;
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use commandf_af02_verifier::canonical::{parse_json_no_duplicates, sha256_hex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const POLICY_BYTES: &[u8] = include_bytes!(
    "../../../specs/016-af-02-adversarial-test-strength/verifier-input-policy.json"
);

#[derive(Debug, Error)]
pub enum InputGuardError {
    #[error("AF-02 input guard violation: {0}")]
    Violation(String),
    #[error("AF-02 input guard I/O failure: {0}")]
    Io(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateFormat {
    Json,
    Yaml,
}

impl CandidateFormat {
    pub fn parse(value: &str) -> Result<Self, InputGuardError> {
        match value {
            "json" => Ok(Self::Json),
            "yaml" | "yml" => Ok(Self::Yaml),
            other => violation(format!("unsupported candidate input format {other}")),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateInput {
    pub format: CandidateFormat,
    pub relative_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GuardReport {
    pub schema: &'static str,
    pub policy_sha256: String,
    pub files: u64,
    pub aggregate_bytes: u64,
    pub records: u64,
    pub max_depth: u64,
    pub json_files: u64,
    pub yaml_files: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifierInputPolicy {
    schema: String,
    preparse: PreparsePolicy,
    json: JsonPolicy,
    yaml: YamlPolicy,
    aggregate: AggregatePolicy,
    parser_runtime: Value,
    enforcement_evidence: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparsePolicy {
    max_single_file_bytes: u64,
    regular_file_required: bool,
    symlinks_allowed: bool,
    containment_required: bool,
    byte_limit_before_parse: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonPolicy {
    utf8_required: bool,
    bom_allowed: bool,
    max_depth: u64,
    max_object_properties: u64,
    max_array_items: u64,
    max_string_bytes: u64,
    max_number_digits: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlPolicy {
    safe_loader_required: bool,
    custom_tags_allowed: bool,
    aliases_allowed: bool,
    merge_keys_allowed: bool,
    max_depth: u64,
    max_sequence_items: u64,
    max_scalar_bytes: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AggregatePolicy {
    max_candidate_authority_bytes: u64,
    max_candidate_authority_files: u64,
    max_total_records: u64,
    parser_wall_seconds: u64,
    parser_memory_mib: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FileStats {
    bytes: u64,
    records: u64,
    depth: u64,
}

pub fn canonical_policy_sha256() -> String {
    sha256_hex(POLICY_BYTES)
}

pub fn guard_inputs(
    candidate_root: &Path,
    inputs: &[CandidateInput],
) -> Result<GuardReport, InputGuardError> {
    let policy = load_policy()?;
    validate_policy_contract(&policy)?;
    if inputs.is_empty() {
        return violation("candidate authority input set must not be empty");
    }
    let files = u64::try_from(inputs.len())
        .map_err(|_| InputGuardError::Violation("candidate input file count overflows u64".to_owned()))?;
    if files > policy.aggregate.max_candidate_authority_files {
        return violation("candidate input file count exceeds policy");
    }

    let root_metadata = fs::symlink_metadata(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return violation("candidate root must be a non-symlink directory");
    }
    let canonical_root = fs::canonicalize(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;

    let mut aggregate_bytes = 0_u64;
    let mut records = 0_u64;
    let mut max_depth = 0_u64;
    let mut json_files = 0_u64;
    let mut yaml_files = 0_u64;

    for input in inputs {
        let remaining_records = policy
            .aggregate
            .max_total_records
            .checked_sub(records)
            .ok_or_else(|| InputGuardError::Violation("candidate input record count exceeds policy".to_owned()))?;
        let stats = guard_one(
            candidate_root,
            &canonical_root,
            input,
            &policy,
            remaining_records,
        )?;
        aggregate_bytes = aggregate_bytes
            .checked_add(stats.bytes)
            .ok_or_else(|| InputGuardError::Violation("candidate input aggregate bytes overflow".to_owned()))?;
        if aggregate_bytes > policy.aggregate.max_candidate_authority_bytes {
            return violation("candidate input aggregate bytes exceed policy");
        }
        records = records
            .checked_add(stats.records)
            .ok_or_else(|| InputGuardError::Violation("candidate input record count overflow".to_owned()))?;
        if records > policy.aggregate.max_total_records {
            return violation("candidate input record count exceeds policy");
        }
        max_depth = max_depth.max(stats.depth);
        match input.format {
            CandidateFormat::Json => json_files += 1,
            CandidateFormat::Yaml => yaml_files += 1,
        }
    }

    Ok(GuardReport {
        schema: "commandf.af02-input-guard-report/v1",
        policy_sha256: canonical_policy_sha256(),
        files,
        aggregate_bytes,
        records,
        max_depth,
        json_files,
        yaml_files,
    })
}

fn load_policy() -> Result<VerifierInputPolicy, InputGuardError> {
    let value = parse_json_no_duplicates(POLICY_BYTES)
        .map_err(|error| InputGuardError::Violation(format!("canonical input policy parse failed: {error}")))?;
    serde_json::from_value(value)
        .map_err(|error| InputGuardError::Violation(format!("canonical input policy shape failed: {error}")))
}

fn validate_policy_contract(policy: &VerifierInputPolicy) -> Result<(), InputGuardError> {
    if policy.schema != "commandf.af02-verifier-input-policy/v1" {
        return violation("unexpected canonical verifier input policy schema");
    }
    if !policy.preparse.regular_file_required
        || policy.preparse.symlinks_allowed
        || !policy.preparse.containment_required
        || !policy.preparse.byte_limit_before_parse
        || policy.preparse.max_single_file_bytes == 0
    {
        return violation("canonical preparse policy is not fail-closed");
    }
    if !policy.json.utf8_required
        || policy.json.bom_allowed
        || policy.json.max_depth == 0
        || policy.json.max_object_properties == 0
        || policy.json.max_array_items == 0
        || policy.json.max_string_bytes == 0
        || policy.json.max_number_digits == 0
    {
        return violation("canonical JSON input policy is not fail-closed");
    }
    if !policy.yaml.safe_loader_required
        || policy.yaml.custom_tags_allowed
        || policy.yaml.aliases_allowed
        || policy.yaml.merge_keys_allowed
        || policy.yaml.max_depth == 0
        || policy.yaml.max_sequence_items == 0
        || policy.yaml.max_scalar_bytes == 0
    {
        return violation("canonical YAML input policy is not fail-closed");
    }
    if policy.aggregate.max_candidate_authority_bytes == 0
        || policy.aggregate.max_candidate_authority_files == 0
        || policy.aggregate.max_total_records == 0
        || policy.aggregate.parser_wall_seconds == 0
        || policy.aggregate.parser_memory_mib == 0
    {
        return violation("canonical aggregate input policy is not fail-closed");
    }
    if !policy.parser_runtime.is_object() || !policy.enforcement_evidence.is_object() {
        return violation("canonical parser runtime or enforcement evidence policy is malformed");
    }
    Ok(())
}

fn guard_one(
    candidate_root: &Path,
    canonical_root: &Path,
    input: &CandidateInput,
    policy: &VerifierInputPolicy,
    remaining_records: u64,
) -> Result<FileStats, InputGuardError> {
    validate_relative_path(&input.relative_path)?;
    let joined = candidate_root.join(&input.relative_path);
    verify_no_symlink_components(candidate_root, &input.relative_path)?;
    let metadata = fs::symlink_metadata(&joined).map_err(|error| io_error(&joined, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return violation(format!(
            "candidate path {} is not a regular non-symlink file",
            input.relative_path.display()
        ));
    }
    if metadata.len() > policy.preparse.max_single_file_bytes {
        return violation(format!(
            "candidate path {} exceeds single-file byte policy before parse",
            input.relative_path.display()
        ));
    }
    let canonical_before = fs::canonicalize(&joined).map_err(|error| io_error(&joined, error))?;
    if !canonical_before.starts_with(canonical_root) {
        return violation(format!(
            "candidate path {} escapes candidate root",
            input.relative_path.display()
        ));
    }

    let file = File::open(&joined).map_err(|error| io_error(&joined, error))?;
    let opened_metadata = file.metadata().map_err(|error| io_error(&joined, error))?;
    if !opened_metadata.is_file() {
        return violation(format!(
            "candidate path {} changed away from a regular file before read",
            input.relative_path.display()
        ));
    }
    let mut bytes = Vec::new();
    file.take(policy.preparse.max_single_file_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(&joined, error))?;
    let byte_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if byte_len > policy.preparse.max_single_file_bytes {
        return violation(format!(
            "candidate path {} exceeds single-file byte policy before parse",
            input.relative_path.display()
        ));
    }

    verify_no_symlink_components(candidate_root, &input.relative_path)?;
    let canonical_after = fs::canonicalize(&joined).map_err(|error| io_error(&joined, error))?;
    let path_metadata = fs::metadata(&joined).map_err(|error| io_error(&joined, error))?;
    if canonical_after != canonical_before
        || !canonical_after.starts_with(canonical_root)
        || !same_file_identity(&opened_metadata, &path_metadata)
    {
        return violation(format!(
            "candidate path {} changed identity during guarded read",
            input.relative_path.display()
        ));
    }

    let mut stats = match input.format {
        CandidateFormat::Json => guard_json(&bytes, &policy.json, remaining_records, &input.relative_path)?,
        CandidateFormat::Yaml => guard_yaml(&bytes, &policy.yaml, remaining_records, &input.relative_path)?,
    };
    stats.bytes = byte_len;
    Ok(stats)
}

#[cfg(unix)]
fn same_file_identity(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file_identity(left: &Metadata, right: &Metadata) -> bool {
    left.len() == right.len() && left.modified().ok() == right.modified().ok()
}

fn validate_relative_path(path: &Path) -> Result<(), InputGuardError> {
    let text = path
        .to_str()
        .ok_or_else(|| InputGuardError::Violation("candidate path must be UTF-8".to_owned()))?;
    if text.is_empty()
        || text.starts_with('/')
        || text.contains('\\')
        || text.contains('\0')
        || text
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return violation(format!("candidate path {text:?} is not portable and relative"));
    }
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return violation(format!("candidate path {text:?} contains a non-normal component"));
    }
    Ok(())
}

fn verify_no_symlink_components(root: &Path, relative: &Path) -> Result<(), InputGuardError> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return violation("candidate path contains a non-normal component");
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current).map_err(|error| io_error(&current, error))?;
        if metadata.file_type().is_symlink() {
            return violation(format!(
                "candidate path {} contains a symlink component",
                relative.display()
            ));
        }
    }
    Ok(())
}

fn guard_json(
    bytes: &[u8],
    policy: &JsonPolicy,
    remaining_records: u64,
    path: &Path,
) -> Result<FileStats, InputGuardError> {
    std::str::from_utf8(bytes).map_err(|_| {
        InputGuardError::Violation(format!("candidate JSON {} is not UTF-8", path.display()))
    })?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return violation(format!(
            "candidate JSON {} contains a prohibited BOM",
            path.display()
        ));
    }
    preflight_json(bytes, policy, remaining_records, path)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|error| {
        InputGuardError::Violation(format!(
            "candidate JSON {} is invalid: {error}",
            path.display()
        ))
    })?;
    analyze_json_value(&value, policy, remaining_records, path)
}

#[derive(Clone, Copy, Debug)]
enum LexKind {
    Object,
    Array,
}

#[derive(Clone, Copy, Debug)]
struct LexFrame {
    kind: LexKind,
    count: u64,
    array_has_value: bool,
}

fn preflight_json(
    bytes: &[u8],
    policy: &JsonPolicy,
    remaining_records: u64,
    path: &Path,
) -> Result<(), InputGuardError> {
    let mut stack = Vec::<LexFrame>::new();
    let mut index = 0_usize;
    let mut records = 1_u64;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => {
                mark_array_value(&mut stack);
                let start = index;
                index += 1;
                let mut escaped = false;
                while index < bytes.len() {
                    let byte = bytes[index];
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        break;
                    }
                    index += 1;
                }
                if index >= bytes.len() {
                    return violation(format!(
                        "candidate JSON {} has an unterminated string",
                        path.display()
                    ));
                }
                let decoded: String = serde_json::from_slice(&bytes[start..=index]).map_err(|error| {
                    InputGuardError::Violation(format!(
                        "candidate JSON {} has an invalid string: {error}",
                        path.display()
                    ))
                })?;
                if u64::try_from(decoded.len()).unwrap_or(u64::MAX) > policy.max_string_bytes {
                    return violation(format!(
                        "candidate JSON {} string exceeds policy",
                        path.display()
                    ));
                }
                index += 1;
            }
            b'{' | b'[' => {
                mark_array_value(&mut stack);
                let depth = u64::try_from(stack.len() + 1).unwrap_or(u64::MAX);
                if depth > policy.max_depth {
                    return violation(format!(
                        "candidate JSON {} exceeds nesting-depth policy before parse",
                        path.display()
                    ));
                }
                stack.push(LexFrame {
                    kind: if bytes[index] == b'{' {
                        LexKind::Object
                    } else {
                        LexKind::Array
                    },
                    count: 0,
                    array_has_value: false,
                });
                index += 1;
            }
            b'}' => {
                let Some(frame) = stack.pop() else {
                    return violation(format!(
                        "candidate JSON {} has unmatched object closure",
                        path.display()
                    ));
                };
                if !matches!(frame.kind, LexKind::Object) {
                    return violation(format!(
                        "candidate JSON {} has mismatched object closure",
                        path.display()
                    ));
                }
                index += 1;
            }
            b']' => {
                let Some(mut frame) = stack.pop() else {
                    return violation(format!(
                        "candidate JSON {} has unmatched array closure",
                        path.display()
                    ));
                };
                if !matches!(frame.kind, LexKind::Array) {
                    return violation(format!(
                        "candidate JSON {} has mismatched array closure",
                        path.display()
                    ));
                }
                if frame.array_has_value {
                    frame.count = frame.count.saturating_add(1);
                }
                if frame.count > policy.max_array_items {
                    return violation(format!(
                        "candidate JSON {} array exceeds item policy before parse",
                        path.display()
                    ));
                }
                records = records.saturating_add(frame.count);
                if records > remaining_records {
                    return violation(format!(
                        "candidate JSON {} exceeds aggregate record policy before parse",
                        path.display()
                    ));
                }
                index += 1;
            }
            b':' => {
                if let Some(frame) = stack.last_mut()
                    && matches!(frame.kind, LexKind::Object)
                {
                    frame.count = frame.count.saturating_add(1);
                    if frame.count > policy.max_object_properties {
                        return violation(format!(
                            "candidate JSON {} object exceeds property policy before parse",
                            path.display()
                        ));
                    }
                    records = records.saturating_add(1);
                    if records > remaining_records {
                        return violation(format!(
                            "candidate JSON {} exceeds aggregate record policy before parse",
                            path.display()
                        ));
                    }
                }
                index += 1;
            }
            b',' => {
                if let Some(frame) = stack.last_mut()
                    && matches!(frame.kind, LexKind::Array)
                    && frame.array_has_value
                {
                    frame.count = frame.count.saturating_add(1);
                    frame.array_has_value = false;
                    if frame.count > policy.max_array_items {
                        return violation(format!(
                            "candidate JSON {} array exceeds item policy before parse",
                            path.display()
                        ));
                    }
                }
                index += 1;
            }
            b'-' | b'0'..=b'9' => {
                mark_array_value(&mut stack);
                let mut cursor = index;
                let mut digits = 0_u64;
                while cursor < bytes.len()
                    && matches!(
                        bytes[cursor],
                        b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E'
                    )
                {
                    if bytes[cursor].is_ascii_digit() {
                        digits = digits.saturating_add(1);
                        if digits > policy.max_number_digits {
                            return violation(format!(
                                "candidate JSON {} number exceeds digit policy before parse",
                                path.display()
                            ));
                        }
                    }
                    cursor += 1;
                }
                index = cursor.max(index + 1);
            }
            b't' | b'f' | b'n' => {
                mark_array_value(&mut stack);
                index += 1;
            }
            _ => index += 1,
        }
    }
    if !stack.is_empty() {
        return violation(format!(
            "candidate JSON {} has unclosed containers",
            path.display()
        ));
    }
    Ok(())
}

fn mark_array_value(stack: &mut [LexFrame]) {
    if let Some(frame) = stack.last_mut()
        && matches!(frame.kind, LexKind::Array)
    {
        frame.array_has_value = true;
    }
}

fn analyze_json_value(
    value: &Value,
    policy: &JsonPolicy,
    remaining_records: u64,
    path: &Path,
) -> Result<FileStats, InputGuardError> {
    fn walk(
        value: &Value,
        policy: &JsonPolicy,
        depth: u64,
        records: &mut u64,
        max_depth: &mut u64,
        path: &Path,
    ) -> Result<(), InputGuardError> {
        *max_depth = (*max_depth).max(depth);
        if depth > policy.max_depth {
            return violation(format!(
                "candidate JSON {} exceeds nesting-depth policy",
                path.display()
            ));
        }
        match value {
            Value::Object(object) => {
                let properties = u64::try_from(object.len()).unwrap_or(u64::MAX);
                if properties > policy.max_object_properties {
                    return violation(format!(
                        "candidate JSON {} object exceeds property policy",
                        path.display()
                    ));
                }
                *records = records.saturating_add(properties);
                for (key, nested) in object {
                    if u64::try_from(key.len()).unwrap_or(u64::MAX) > policy.max_string_bytes {
                        return violation(format!(
                            "candidate JSON {} key exceeds string policy",
                            path.display()
                        ));
                    }
                    walk(nested, policy, depth + 1, records, max_depth, path)?;
                }
            }
            Value::Array(array) => {
                let items = u64::try_from(array.len()).unwrap_or(u64::MAX);
                if items > policy.max_array_items {
                    return violation(format!(
                        "candidate JSON {} array exceeds item policy",
                        path.display()
                    ));
                }
                *records = records.saturating_add(items);
                for nested in array {
                    walk(nested, policy, depth + 1, records, max_depth, path)?;
                }
            }
            Value::String(text) => {
                if u64::try_from(text.len()).unwrap_or(u64::MAX) > policy.max_string_bytes {
                    return violation(format!(
                        "candidate JSON {} string exceeds policy",
                        path.display()
                    ));
                }
            }
            Value::Number(number) => {
                let digits = number
                    .to_string()
                    .bytes()
                    .filter(|byte| byte.is_ascii_digit())
                    .count();
                if u64::try_from(digits).unwrap_or(u64::MAX) > policy.max_number_digits {
                    return violation(format!(
                        "candidate JSON {} number exceeds digit policy",
                        path.display()
                    ));
                }
            }
            Value::Bool(_) | Value::Null => {}
        }
        Ok(())
    }

    let mut records = 1_u64;
    let mut max_depth = 1_u64;
    walk(value, policy, 1, &mut records, &mut max_depth, path)?;
    if records > remaining_records {
        return violation(format!(
            "candidate JSON {} exceeds aggregate record policy",
            path.display()
        ));
    }
    Ok(FileStats {
        bytes: 0,
        records,
        depth: max_depth,
    })
}

fn guard_yaml(
    bytes: &[u8],
    policy: &YamlPolicy,
    remaining_records: u64,
    path: &Path,
) -> Result<FileStats, InputGuardError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        InputGuardError::Violation(format!("candidate YAML {} is not UTF-8", path.display()))
    })?;
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return violation(format!(
            "candidate YAML {} contains a prohibited BOM",
            path.display()
        ));
    }

    let mut indent_stack = vec![0_usize];
    let mut sequence_counts = BTreeMap::<usize, u64>::new();
    let mut block_scalar: Option<(usize, u64)> = None;
    let mut records = 0_u64;
    let mut max_depth = 1_u64;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line_number = line_index + 1;
        let indent = leading_spaces(raw_line, path, line_number)?;
        let trimmed_raw = raw_line.trim();

        if let Some((header_indent, accumulated)) = block_scalar.as_mut() {
            if trimmed_raw.is_empty() || indent > *header_indent {
                *accumulated = accumulated
                    .checked_add(u64::try_from(raw_line.len() + 1).unwrap_or(u64::MAX))
                    .ok_or_else(|| {
                        InputGuardError::Violation(format!(
                            "candidate YAML {} block scalar byte count overflow",
                            path.display()
                        ))
                    })?;
                if *accumulated > policy.max_scalar_bytes {
                    return violation(format!(
                        "candidate YAML {} line {line_number} block scalar exceeds policy",
                        path.display()
                    ));
                }
                continue;
            }
            block_scalar = None;
        }

        let without_comment = strip_yaml_comment(raw_line);
        let trimmed = without_comment.trim();
        if trimmed.is_empty() || trimmed == "---" || trimmed == "..." {
            continue;
        }
        if trimmed.starts_with("%TAG") || trimmed.starts_with("%YAML") {
            return violation(format!(
                "candidate YAML {} line {line_number} uses a prohibited directive",
                path.display()
            ));
        }

        while indent < *indent_stack.last().unwrap_or(&0) {
            if let Some(closed) = indent_stack.pop() {
                sequence_counts.remove(&closed);
            }
        }
        if indent > *indent_stack.last().unwrap_or(&0) {
            indent_stack.push(indent);
        }
        let depth = u64::try_from(indent_stack.len()).unwrap_or(u64::MAX);
        max_depth = max_depth.max(depth);
        if depth > policy.max_depth {
            return violation(format!(
                "candidate YAML {} exceeds nesting-depth policy",
                path.display()
            ));
        }

        let mut content = trimmed;
        if let Some(rest) = content.strip_prefix('-')
            && (rest.is_empty()
                || rest
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| byte.is_ascii_whitespace()))
        {
            let count = sequence_counts.entry(indent).or_insert(0);
            *count = count.saturating_add(1);
            if *count > policy.max_sequence_items {
                return violation(format!(
                    "candidate YAML {} sequence exceeds item policy",
                    path.display()
                ));
            }
            records = records.saturating_add(1);
            content = rest.trim_start();
        }

        validate_yaml_flow(content, policy, path, line_number)?;
        if let Some((key, value)) = split_yaml_mapping(content) {
            if key.trim() == "<<" {
                return violation(format!(
                    "candidate YAML {} line {line_number} contains a prohibited merge key",
                    path.display()
                ));
            }
            validate_yaml_scalar(key.trim(), policy, path, line_number)?;
            let value = value.trim();
            if is_block_scalar_header(value) {
                block_scalar = Some((indent, 0));
            } else if !value.is_empty() {
                validate_yaml_scalar(value, policy, path, line_number)?;
            }
            records = records.saturating_add(1);
        } else if !content.is_empty() {
            if is_block_scalar_header(content) {
                block_scalar = Some((indent, 0));
            } else {
                validate_yaml_scalar(content, policy, path, line_number)?;
            }
            if !trimmed.starts_with('-') {
                records = records.saturating_add(1);
            }
        }
        if records > remaining_records {
            return violation(format!(
                "candidate YAML {} exceeds aggregate record policy before parse",
                path.display()
            ));
        }
    }

    Ok(FileStats {
        bytes: 0,
        records,
        depth: max_depth,
    })
}

fn leading_spaces(line: &str, path: &Path, line_number: usize) -> Result<usize, InputGuardError> {
    let mut count = 0_usize;
    for byte in line.bytes() {
        match byte {
            b' ' => count += 1,
            b'\t' => {
                return violation(format!(
                    "candidate YAML {} line {line_number} uses tab indentation",
                    path.display()
                ));
            }
            _ => break,
        }
    }
    Ok(count)
}

fn strip_yaml_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if double && escaped {
            escaped = false;
            continue;
        }
        if double && byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'#' if !single
                && !double
                && (index == 0 || bytes[index - 1].is_ascii_whitespace()) =>
            {
                return &line[..index];
            }
            _ => {}
        }
    }
    line
}

fn split_yaml_mapping(text: &str) -> Option<(&str, &str)> {
    let bytes = text.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut flow_depth = 0_u64;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if double && escaped {
            escaped = false;
            continue;
        }
        if double && byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'[' | b'{' if !single && !double => flow_depth += 1,
            b']' | b'}' if !single && !double => flow_depth = flow_depth.saturating_sub(1),
            b':' if !single && !double && flow_depth == 0 => {
                let next = bytes.get(index + 1).copied();
                if next.is_none() || next.is_some_and(|byte| byte.is_ascii_whitespace()) {
                    return Some((&text[..index], &text[index + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn is_block_scalar_header(value: &str) -> bool {
    matches!(value, "|" | "|-" | "|+" | ">" | ">-" | ">+")
}

fn validate_yaml_scalar(
    scalar: &str,
    policy: &YamlPolicy,
    path: &Path,
    line: usize,
) -> Result<(), InputGuardError> {
    if u64::try_from(scalar.len()).unwrap_or(u64::MAX) > policy.max_scalar_bytes {
        return violation(format!(
            "candidate YAML {} line {line} scalar exceeds policy",
            path.display()
        ));
    }
    let trimmed = scalar.trim_start();
    if trimmed.starts_with('*') || trimmed.starts_with('&') {
        return violation(format!(
            "candidate YAML {} line {line} contains a prohibited alias or anchor",
            path.display()
        ));
    }
    if trimmed.starts_with('!') {
        return violation(format!(
            "candidate YAML {} line {line} contains a prohibited custom tag",
            path.display()
        ));
    }
    for segment in yaml_flow_segments(trimmed) {
        let segment = segment.trim_start();
        if segment.starts_with('*') || segment.starts_with('&') {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited alias or anchor",
                path.display()
            ));
        }
        if segment.starts_with('!') {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited custom tag",
                path.display()
            ));
        }
        if segment.starts_with("<<:") || segment == "<<" {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited merge key",
                path.display()
            ));
        }
    }
    Ok(())
}

fn validate_yaml_flow(
    text: &str,
    policy: &YamlPolicy,
    path: &Path,
    line: usize,
) -> Result<(), InputGuardError> {
    let bytes = text.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut sequence_items = Vec::<u64>::new();
    let mut flow_depth = 0_u64;
    for byte in bytes.iter().copied() {
        if double && escaped {
            escaped = false;
            continue;
        }
        if double && byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'[' if !single && !double => {
                flow_depth += 1;
                sequence_items.push(1);
                if flow_depth > policy.max_depth {
                    return violation(format!(
                        "candidate YAML {} line {line} exceeds flow nesting-depth policy",
                        path.display()
                    ));
                }
            }
            b']' if !single && !double => {
                sequence_items.pop();
                flow_depth = flow_depth.saturating_sub(1);
            }
            b',' if !single && !double => {
                if let Some(items) = sequence_items.last_mut() {
                    *items = items.saturating_add(1);
                    if *items > policy.max_sequence_items {
                        return violation(format!(
                            "candidate YAML {} line {line} flow sequence exceeds item policy",
                            path.display()
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn yaml_flow_segments(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut starts = vec![0_usize];
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if double && escaped {
            escaped = false;
            continue;
        }
        if double && byte == b'\\' {
            escaped = true;
            continue;
        }
        match byte {
            b'\'' if !double => single = !single,
            b'"' if !single => double = !double,
            b'[' | b'{' | b',' | b':' if !single && !double => starts.push(index + 1),
            _ => {}
        }
    }
    starts.into_iter().map(|start| &text[start..]).collect()
}

fn io_error(path: &Path, error: std::io::Error) -> InputGuardError {
    InputGuardError::Io(format!("{}: {error}", path.display()))
}

fn violation<T>(message: impl Into<String>) -> Result<T, InputGuardError> {
    Err(InputGuardError::Violation(message.into()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new() -> Self {
            let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "commandf-af02-input-guard-{}-{nonce}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn write(&self, relative: &str, bytes: &[u8]) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, bytes).unwrap();
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn input(format: CandidateFormat, path: &str) -> CandidateInput {
        CandidateInput {
            format,
            relative_path: PathBuf::from(path),
        }
    }

    #[test]
    fn accepts_bounded_json_yaml_and_block_scalars_deterministically() {
        let root = TempRoot::new();
        root.write("policy.json", br#"{"a":[1,2],"b":"ok"}"#);
        root.write(
            "workflow.yml",
            b"name: test\njobs:\n  verify:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: |\n          echo ok\n          echo done\n",
        );
        let inputs = [
            input(CandidateFormat::Json, "policy.json"),
            input(CandidateFormat::Yaml, "workflow.yml"),
        ];
        let first = guard_inputs(&root.path, &inputs).unwrap();
        let second = guard_inputs(&root.path, &inputs).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.files, 2);
        assert_eq!(first.json_files, 1);
        assert_eq!(first.yaml_files, 1);
        assert_eq!(first.policy_sha256.len(), 64);
    }

    #[test]
    fn sibling_yaml_sequences_have_independent_limits() {
        let root = TempRoot::new();
        let first = "  - a\n".repeat(6_000);
        let second = "  - b\n".repeat(6_000);
        let document = format!("first:\n{first}second:\n{second}");
        root.write("siblings.yml", document.as_bytes());
        guard_inputs(
            &root.path,
            &[input(CandidateFormat::Yaml, "siblings.yml")],
        )
        .unwrap();
    }

    #[test]
    fn rejects_oversize_before_json_parse() {
        let root = TempRoot::new();
        let policy = load_policy().unwrap();
        let bytes = vec![
            b' ';
            usize::try_from(policy.preparse.max_single_file_bytes + 1).unwrap()
        ];
        root.write("oversize.json", &bytes);
        let error = guard_inputs(
            &root.path,
            &[input(CandidateFormat::Json, "oversize.json")],
        )
        .unwrap_err();
        assert!(error.to_string().contains("before parse"));
    }

    #[test]
    fn rejects_json_depth_before_full_parse() {
        let root = TempRoot::new();
        let policy = load_policy().unwrap();
        let depth = usize::try_from(policy.json.max_depth + 1).unwrap();
        let text = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        root.write("deep.json", text.as_bytes());
        let error = guard_inputs(&root.path, &[input(CandidateFormat::Json, "deep.json")])
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("nesting-depth policy before parse"));
    }

    #[test]
    fn rejects_aggregate_record_exhaustion() {
        let root = TempRoot::new();
        let array = format!(
            "[{}]",
            (0..17_000).map(|_| "0").collect::<Vec<_>>().join(",")
        );
        let document = format!("{{\"a\":{array},\"b\":{array},\"c\":{array}}}");
        root.write("records.json", document.as_bytes());
        let error = guard_inputs(
            &root.path,
            &[input(CandidateFormat::Json, "records.json")],
        )
        .unwrap_err();
        assert!(error.to_string().contains("record policy"));
    }

    #[test]
    fn rejects_yaml_alias_anchor_custom_tag_and_merge_key() {
        for (name, text, expected) in [
            ("alias.yml", "value: *shared\n", "alias"),
            ("anchor.yml", "value: &shared x\n", "anchor"),
            ("tag.yml", "value: !custom tagged\n", "custom tag"),
            ("merge.yml", "<<: *base\n", "merge key"),
        ] {
            let root = TempRoot::new();
            root.write(name, text.as_bytes());
            let error = guard_inputs(&root.path, &[input(CandidateFormat::Yaml, name)])
                .unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn rejects_path_traversal() {
        let root = TempRoot::new();
        let error = guard_inputs(
            &root.path,
            &[input(CandidateFormat::Json, "../outside.json")],
        )
        .unwrap_err();
        assert!(error.to_string().contains("not portable"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_component_before_read() {
        use std::os::unix::fs::symlink;

        let root = TempRoot::new();
        let outside = TempRoot::new();
        outside.write("value.json", b"{}");
        symlink(&outside.path, root.path.join("link")).unwrap();
        let error = guard_inputs(
            &root.path,
            &[input(CandidateFormat::Json, "link/value.json")],
        )
        .unwrap_err();
        assert!(error.to_string().contains("symlink component"));
    }

    #[test]
    fn rejects_unsupported_format() {
        let error = CandidateFormat::parse("toml").unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported candidate input format"));
    }
}