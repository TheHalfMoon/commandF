#[cfg(unix)]
mod legacy {
    include!("input_guard_unix.rs");
}

#[cfg(unix)]
pub use legacy::{
    canonical_policy_sha256, CandidateFormat, CandidateInput, GuardReport, InputGuardError,
};

#[cfg(not(unix))]
mod unsupported {
    use std::path::PathBuf;

    use commandf_af02_verifier::canonical::sha256_hex;
    use serde::Serialize;
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
                other => Err(InputGuardError::Violation(format!(
                    "unsupported candidate input format {other}"
                ))),
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

    pub fn canonical_policy_sha256() -> String {
        sha256_hex(POLICY_BYTES)
    }
}

#[cfg(not(unix))]
pub use unsupported::{
    canonical_policy_sha256, CandidateFormat, CandidateInput, GuardReport, InputGuardError,
};

#[cfg(unix)]
use std::fs::{self, File};
#[cfg(unix)]
use std::io::Read;
use std::path::Path;
#[cfg(unix)]
use std::path::{Component, PathBuf};

#[cfg(unix)]
use commandf_af02_verifier::canonical::parse_json_no_duplicates;
#[cfg(unix)]
use serde::Deserialize;

#[cfg(unix)]
const POLICY_BYTES: &[u8] = include_bytes!(
    "../../../specs/016-af-02-adversarial-test-strength/verifier-input-policy.json"
);

pub fn guard_inputs(
    candidate_root: &Path,
    inputs: &[CandidateInput],
) -> Result<GuardReport, InputGuardError> {
    #[cfg(not(unix))]
    {
        let _ = (candidate_root, inputs);
        Err(InputGuardError::Violation(
            "secure opened-file identity verification is unavailable on this platform".to_owned(),
        ))
    }

    #[cfg(unix)]
    {
        let policy = load_hardening_policy()?;
        let hardening = hardened_yaml_preflight(candidate_root, inputs, &policy)?;
        let mut report = legacy::guard_inputs(candidate_root, inputs)?;
        report.records = report.records.checked_add(hardening.flow_records).ok_or_else(|| {
            InputGuardError::Violation("candidate input record count overflow".to_owned())
        })?;
        if report.records > policy.aggregate.max_total_records {
            return Err(InputGuardError::Violation(
                "candidate input record count exceeds policy".to_owned(),
            ));
        }
        report.max_depth = report.max_depth.max(hardening.max_depth);
        Ok(report)
    }
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct HardeningPolicy {
    preparse: HardeningPreparsePolicy,
    yaml: HardeningYamlPolicy,
    aggregate: HardeningAggregatePolicy,
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct HardeningPreparsePolicy {
    max_single_file_bytes: u64,
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct HardeningYamlPolicy {
    max_depth: u64,
    max_sequence_items: u64,
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
struct HardeningAggregatePolicy {
    max_total_records: u64,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct HardenedYamlStats {
    flow_records: u64,
    max_depth: u64,
}

#[cfg(unix)]
fn load_hardening_policy() -> Result<HardeningPolicy, InputGuardError> {
    let value = parse_json_no_duplicates(POLICY_BYTES).map_err(|error| {
        InputGuardError::Violation(format!("canonical input policy parse failed: {error}"))
    })?;
    serde_json::from_value(value).map_err(|error| {
        InputGuardError::Violation(format!("canonical input policy shape failed: {error}"))
    })
}

#[cfg(unix)]
fn hardened_yaml_preflight(
    candidate_root: &Path,
    inputs: &[CandidateInput],
    policy: &HardeningPolicy,
) -> Result<HardenedYamlStats, InputGuardError> {
    if !inputs
        .iter()
        .any(|input| input.format == CandidateFormat::Yaml)
    {
        return Ok(HardenedYamlStats::default());
    }

    let root_metadata = fs::symlink_metadata(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return violation("candidate root must be a non-symlink directory");
    }
    let canonical_root = fs::canonicalize(candidate_root)
        .map_err(|error| io_error(candidate_root, error))?;

    let mut aggregate = HardenedYamlStats::default();
    for input in inputs
        .iter()
        .filter(|input| input.format == CandidateFormat::Yaml)
    {
        let bytes = read_guarded_yaml(
            candidate_root,
            &canonical_root,
            input,
            policy.preparse.max_single_file_bytes,
        )?;
        let stats = preflight_yaml_flow(
            &bytes,
            &policy.yaml,
            &input.relative_path,
        )?;
        aggregate.flow_records = aggregate
            .flow_records
            .checked_add(stats.flow_records)
            .ok_or_else(|| {
                InputGuardError::Violation("candidate input record count overflow".to_owned())
            })?;
        if aggregate.flow_records > policy.aggregate.max_total_records {
            return violation("candidate YAML flow records exceed aggregate record policy before parse");
        }
        aggregate.max_depth = aggregate.max_depth.max(stats.max_depth);
    }
    Ok(aggregate)
}

#[cfg(unix)]
fn read_guarded_yaml(
    candidate_root: &Path,
    canonical_root: &Path,
    input: &CandidateInput,
    max_single_file_bytes: u64,
) -> Result<Vec<u8>, InputGuardError> {
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
    if metadata.len() > max_single_file_bytes {
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
    file.take(max_single_file_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error(&joined, error))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_single_file_bytes {
        return violation(format!(
            "candidate path {} exceeds single-file byte policy before parse",
            input.relative_path.display()
        ));
    }

    verify_no_symlink_components(candidate_root, &input.relative_path)?;
    let canonical_after = fs::canonicalize(&joined).map_err(|error| io_error(&joined, error))?;
    let path_metadata = fs::metadata(&joined).map_err(|error| io_error(&joined, error))?;
    use std::os::unix::fs::MetadataExt;
    if canonical_after != canonical_before
        || !canonical_after.starts_with(canonical_root)
        || opened_metadata.dev() != path_metadata.dev()
        || opened_metadata.ino() != path_metadata.ino()
    {
        return violation(format!(
            "candidate path {} changed identity during guarded read",
            input.relative_path.display()
        ));
    }
    Ok(bytes)
}

#[cfg(unix)]
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
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return violation(format!("candidate path {text:?} is not portable and relative"));
    }
    Ok(())
}

#[cfg(unix)]
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

#[cfg(unix)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FlowKind {
    Sequence,
    Mapping,
}

#[cfg(unix)]
#[derive(Clone, Copy, Debug)]
struct FlowFrame {
    kind: FlowKind,
    sequence_items: u64,
    sequence_has_item: bool,
    mapping_expects_key_separator: bool,
}

#[cfg(unix)]
impl FlowFrame {
    fn sequence() -> Self {
        Self {
            kind: FlowKind::Sequence,
            sequence_items: 0,
            sequence_has_item: false,
            mapping_expects_key_separator: false,
        }
    }

    fn mapping() -> Self {
        Self {
            kind: FlowKind::Mapping,
            sequence_items: 0,
            sequence_has_item: false,
            mapping_expects_key_separator: true,
        }
    }
}

#[cfg(unix)]
fn preflight_yaml_flow(
    bytes: &[u8],
    policy: &HardeningYamlPolicy,
    path: &Path,
) -> Result<HardenedYamlStats, InputGuardError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        InputGuardError::Violation(format!("candidate YAML {} is not UTF-8", path.display()))
    })?;

    let mut stats = HardenedYamlStats::default();
    let mut indent_stack = vec![0_usize];
    let mut flow_stack = Vec::<FlowFrame>::new();
    let mut flow_base_depth: Option<u64> = None;
    let mut block_scalar_indent: Option<usize> = None;

    for (line_index, raw_line) in text.lines().enumerate() {
        let line = line_index + 1;
        let indent = leading_spaces(raw_line, path, line)?;
        let trimmed_raw = raw_line.trim();

        if let Some(header_indent) = block_scalar_indent {
            if trimmed_raw.is_empty() || indent > header_indent {
                continue;
            }
            block_scalar_indent = None;
        }

        let without_comment = strip_yaml_comment(raw_line);
        let trimmed = without_comment.trim();
        if trimmed.is_empty() || trimmed == "---" || trimmed == "..." {
            continue;
        }

        reject_explicit_node_properties(trimmed, path, line)?;

        let base_depth = if flow_stack.is_empty() {
            while indent < *indent_stack.last().unwrap_or(&0) {
                indent_stack.pop();
            }
            if indent > *indent_stack.last().unwrap_or(&0) {
                indent_stack.push(indent);
            }
            u64::try_from(indent_stack.len()).unwrap_or(u64::MAX)
        } else {
            flow_base_depth.unwrap_or(1)
        };

        scan_flow_line(
            trimmed,
            policy,
            path,
            line,
            base_depth,
            &mut flow_stack,
            &mut stats,
        )?;

        if flow_stack.is_empty() {
            flow_base_depth = None;
            if is_block_scalar_header_line(trimmed) {
                block_scalar_indent = Some(indent);
            }
        } else if flow_base_depth.is_none() {
            flow_base_depth = Some(base_depth);
        }
    }

    if !flow_stack.is_empty() {
        return violation(format!(
            "candidate YAML {} has an unclosed flow collection",
            path.display()
        ));
    }
    Ok(stats)
}

#[cfg(unix)]
fn scan_flow_line(
    text: &str,
    policy: &HardeningYamlPolicy,
    path: &Path,
    line: usize,
    base_depth: u64,
    stack: &mut Vec<FlowFrame>,
    stats: &mut HardenedYamlStats,
) -> Result<(), InputGuardError> {
    let bytes = text.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut escaped = false;

    for byte in bytes.iter().copied() {
        if double && escaped {
            escaped = false;
            continue;
        }
        if double {
            if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                double = false;
            }
            continue;
        }
        if single {
            if byte == b'\'' {
                single = false;
            }
            continue;
        }

        match byte {
            b'\'' => {
                mark_sequence_content(stack);
                single = true;
            }
            b'"' => {
                mark_sequence_content(stack);
                double = true;
            }
            b'[' | b'{' => {
                mark_sequence_content(stack);
                stack.push(if byte == b'[' {
                    FlowFrame::sequence()
                } else {
                    FlowFrame::mapping()
                });
                let depth = base_depth
                    .checked_add(u64::try_from(stack.len()).unwrap_or(u64::MAX))
                    .unwrap_or(u64::MAX);
                stats.max_depth = stats.max_depth.max(depth);
                if depth > policy.max_depth {
                    return violation(format!(
                        "candidate YAML {} line {line} exceeds flow nesting-depth policy",
                        path.display()
                    ));
                }
            }
            b']' => close_flow(
                FlowKind::Sequence,
                policy,
                path,
                line,
                stack,
                stats,
            )?,
            b'}' => close_flow(
                FlowKind::Mapping,
                policy,
                path,
                line,
                stack,
                stats,
            )?,
            b',' => {
                if let Some(frame) = stack.last_mut() {
                    match frame.kind {
                        FlowKind::Sequence => finish_sequence_item(frame, policy, path, line)?,
                        FlowKind::Mapping => frame.mapping_expects_key_separator = true,
                    }
                }
            }
            b':' => {
                if let Some(frame) = stack.last_mut()
                    && frame.kind == FlowKind::Mapping
                    && frame.mapping_expects_key_separator
                {
                    stats.flow_records = stats.flow_records.checked_add(1).ok_or_else(|| {
                        InputGuardError::Violation(
                            "candidate YAML flow record count overflow".to_owned(),
                        )
                    })?;
                    frame.mapping_expects_key_separator = false;
                }
            }
            byte if byte.is_ascii_whitespace() => {}
            _ => mark_sequence_content(stack),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn close_flow(
    expected: FlowKind,
    policy: &HardeningYamlPolicy,
    path: &Path,
    line: usize,
    stack: &mut Vec<FlowFrame>,
    stats: &mut HardenedYamlStats,
) -> Result<(), InputGuardError> {
    let Some(mut frame) = stack.pop() else {
        return violation(format!(
            "candidate YAML {} line {line} has an unmatched flow closure",
            path.display()
        ));
    };
    if frame.kind != expected {
        return violation(format!(
            "candidate YAML {} line {line} has a mismatched flow closure",
            path.display()
        ));
    }
    if frame.kind == FlowKind::Sequence {
        finish_sequence_item(&mut frame, policy, path, line)?;
        stats.flow_records = stats
            .flow_records
            .checked_add(frame.sequence_items)
            .ok_or_else(|| {
                InputGuardError::Violation("candidate YAML flow record count overflow".to_owned())
            })?;
    }
    Ok(())
}

#[cfg(unix)]
fn finish_sequence_item(
    frame: &mut FlowFrame,
    policy: &HardeningYamlPolicy,
    path: &Path,
    line: usize,
) -> Result<(), InputGuardError> {
    if !frame.sequence_has_item {
        return Ok(());
    }
    frame.sequence_items = frame.sequence_items.saturating_add(1);
    frame.sequence_has_item = false;
    if frame.sequence_items > policy.max_sequence_items {
        return violation(format!(
            "candidate YAML {} line {line} flow sequence exceeds item policy",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn mark_sequence_content(stack: &mut [FlowFrame]) {
    if let Some(frame) = stack.last_mut()
        && frame.kind == FlowKind::Sequence
    {
        frame.sequence_has_item = true;
    }
}

#[cfg(unix)]
fn reject_explicit_node_properties(
    text: &str,
    path: &Path,
    line: usize,
) -> Result<(), InputGuardError> {
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

    for start in starts {
        let segment = text[start..].trim_start();
        let Some(rest) = segment.strip_prefix('?') else {
            continue;
        };
        if !rest.is_empty()
            && !rest
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            continue;
        }
        let node = rest.trim_start();
        if node.starts_with('*') || node.starts_with('&') {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited alias or anchor in an explicit key",
                path.display()
            ));
        }
        if node.starts_with('!') {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited custom tag in an explicit key",
                path.display()
            ));
        }
        if node == "<<"
            || node.starts_with("<<:")
            || node
                .strip_prefix("<<")
                .and_then(|tail| tail.as_bytes().first())
                .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            return violation(format!(
                "candidate YAML {} line {line} contains a prohibited merge key in an explicit key",
                path.display()
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn leading_spaces(
    line: &str,
    path: &Path,
    line_number: usize,
) -> Result<usize, InputGuardError> {
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

#[cfg(unix)]
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

#[cfg(unix)]
fn is_block_scalar_header_line(text: &str) -> bool {
    const MARKERS: [&str; 6] = ["|", "|-", "|+", ">", ">-", ">+"];
    let trimmed = text.trim_end();
    MARKERS.iter().any(|marker| {
        trimmed == *marker
            || trimmed.ends_with(&format!(": {marker}"))
            || trimmed.ends_with(&format!("- {marker}"))
    })
}

#[cfg(unix)]
fn io_error(path: &Path, error: std::io::Error) -> InputGuardError {
    InputGuardError::Io(format!("{}: {error}", path.display()))
}

#[cfg(unix)]
fn violation<T>(message: impl Into<String>) -> Result<T, InputGuardError> {
    Err(InputGuardError::Violation(message.into()))
}

#[cfg(all(test, unix))]
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
                "commandf-af02-hardened-input-guard-{}-{nonce}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn write(&self, relative: &str, text: &str) {
            fs::write(self.path.join(relative), text.as_bytes()).unwrap();
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn yaml(path: &str) -> CandidateInput {
        CandidateInput {
            format: CandidateFormat::Yaml,
            relative_path: PathBuf::from(path),
        }
    }

    #[test]
    fn rejects_flow_records_that_exhaust_aggregate_budget() {
        let root = TempRoot::new();
        let sequence = (0..9_000).map(|_| "0").collect::<Vec<_>>().join(",");
        let document = (0..6)
            .map(|index| format!("k{index}: [{sequence}]"))
            .collect::<Vec<_>>()
            .join("\n");
        root.write("records.yml", &document);
        let error = guard_inputs(&root.path, &[yaml("records.yml")]).unwrap_err();
        assert!(error.to_string().contains("record"), "{error}");
    }

    #[test]
    fn rejects_flow_mapping_depth_combined_with_block_depth() {
        let root = TempRoot::new();
        let policy = load_hardening_policy().unwrap();
        let flow_depth = usize::try_from(policy.yaml.max_depth).unwrap();
        let document = format!(
            "value: {}0{}\n",
            "{a: ".repeat(flow_depth),
            "}".repeat(flow_depth)
        );
        root.write("deep.yml", &document);
        let error = guard_inputs(&root.path, &[yaml("deep.yml")]).unwrap_err();
        assert!(error.to_string().contains("nesting-depth"), "{error}");
    }

    #[test]
    fn rejects_prohibited_node_properties_in_explicit_yaml_keys() {
        for (name, text, expected) in [
            ("alias.yml", "? *shared\n: value\n", "alias"),
            ("anchor.yml", "? &shared key\n: value\n", "anchor"),
            ("tag.yml", "? !custom key\n: value\n", "custom tag"),
            ("merge.yml", "? <<\n: value\n", "merge key"),
        ] {
            let root = TempRoot::new();
            root.write(name, text);
            let error = guard_inputs(&root.path, &[yaml(name)]).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    #[test]
    fn accepts_bounded_flow_yaml_and_reports_flow_records() {
        let root = TempRoot::new();
        root.write("flow.yml", "value: [{a: 1}, {b: [2, 3]}]\n");
        let report = guard_inputs(&root.path, &[yaml("flow.yml")]).unwrap();
        assert!(report.records >= 7);
        assert!(report.max_depth >= 4);
    }
}

#[cfg(all(test, not(unix)))]
mod non_unix_tests {
    use super::*;

    #[test]
    fn input_guard_fails_closed_without_secure_file_identity() {
        let error = guard_inputs(Path::new("."), &[]).unwrap_err();
        assert!(error
            .to_string()
            .contains("secure opened-file identity verification is unavailable"));
    }
}
