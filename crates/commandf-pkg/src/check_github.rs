use crate::{
    check::{direction_selected, validate_check_report},
    validate_source_mapped_check_report, CheckError, CheckReport, CompatibilityDirection,
    CompatibilityFinding, CompatibilitySeverity, ElementView, ResourceKeyKind, SourceLocation,
    SourceMapError, SourceMappedCheckReport, SourceMappingEntry, StructuralChangeKind,
};

const MAX_ERROR_ANNOTATIONS: usize = 10;
const MAX_WARNING_ANNOTATIONS: usize = 10;
const MAX_NOTICE_ANNOTATIONS: usize = 10;
const MAX_ANNOTATION_TITLE_CHARS: usize = 256;
const MAX_ANNOTATION_MESSAGE_CHARS: usize = 4_000;

#[derive(Clone, Copy)]
enum AnnotationLevel {
    Error,
    Warning,
    Notice,
}

impl AnnotationLevel {
    fn index(self) -> usize {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
            Self::Notice => 2,
        }
    }

    fn command(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Notice => "notice",
        }
    }
}

pub fn check_report_to_github_annotations_bytes(
    report: &CheckReport,
) -> Result<Vec<u8>, CheckError> {
    validate_check_report(report)?;
    Ok(render_annotations(report, None))
}

pub fn source_mapped_check_report_to_github_annotations_bytes(
    report: &CheckReport,
    source_map: &SourceMappedCheckReport,
) -> Result<Vec<u8>, SourceMapError> {
    validate_check_report(report)?;
    validate_source_mapped_check_report(source_map)?;
    if source_map.check != *report {
        return Err(SourceMapError::CheckReportMismatch);
    }
    Ok(render_annotations(report, Some(source_map)))
}

fn render_annotations(
    report: &CheckReport,
    source_map: Option<&SourceMappedCheckReport>,
) -> Vec<u8> {
    let selected = report
        .compatibility
        .findings
        .iter()
        .enumerate()
        .filter(|(_, finding)| direction_selected(report.policy.direction, finding.direction))
        .collect::<Vec<_>>();

    let mut totals = [0_usize; 3];
    for (_, finding) in &selected {
        totals[annotation_level(finding.severity).index()] += 1;
    }

    let overflow = totals[0] > MAX_ERROR_ANNOTATIONS
        || totals[1] > MAX_WARNING_ANNOTATIONS
        || totals[2] > MAX_NOTICE_ANNOTATIONS;
    let limits = [
        MAX_ERROR_ANNOTATIONS,
        MAX_WARNING_ANNOTATIONS,
        if overflow {
            MAX_NOTICE_ANNOTATIONS - 1
        } else {
            MAX_NOTICE_ANNOTATIONS
        },
    ];

    let mut emitted = [0_usize; 3];
    let mut omitted = [0_usize; 3];
    let mut output = String::new();

    for (finding_index, finding) in selected {
        let level = annotation_level(finding.severity);
        let index = level.index();
        if emitted[index] >= limits[index] {
            omitted[index] += 1;
            continue;
        }
        emitted[index] += 1;
        let mapping = source_map.map(|mapped| &mapped.mappings[finding_index]);
        write_annotation(&mut output, level, finding, mapping);
    }

    if overflow {
        let message = format!(
            "GitHub annotation projection is incomplete: omitted errors={}, warnings={}, notices={}; the complete CF-05 JSON report remains authoritative",
            omitted[0], omitted[1], omitted[2]
        );
        output.push_str("::notice title=");
        output.push_str(&escape_property(
            "commandF annotation projection incomplete",
        ));
        output.push_str("::");
        output.push_str(&escape_data(&message));
        output.push('\n');
    }

    output.into_bytes()
}

fn write_annotation(
    output: &mut String,
    level: AnnotationLevel,
    finding: &CompatibilityFinding,
    mapping: Option<&SourceMappingEntry>,
) {
    let title = bounded_title(&format!("commandF {}", finding.rule_id));
    let message = bounded_message(&annotation_message(finding, mapping));

    output.push_str("::");
    output.push_str(level.command());
    output.push_str(" title=");
    output.push_str(&escape_property(&title));
    if let Some(location) = mapping.and_then(|entry| entry.location.as_ref()) {
        write_location_properties(output, location);
    }
    output.push_str("::");
    output.push_str(&escape_data(&message));
    output.push('\n');
}

fn write_location_properties(output: &mut String, location: &SourceLocation) {
    output.push_str(",file=");
    output.push_str(&escape_property(&location.file));
    output.push_str(",line=");
    output.push_str(&location.line.to_string());
    output.push_str(",endLine=");
    output.push_str(&location.end_line.to_string());
}

fn annotation_level(severity: CompatibilitySeverity) -> AnnotationLevel {
    match severity {
        CompatibilitySeverity::Breaking => AnnotationLevel::Error,
        CompatibilitySeverity::Risky => AnnotationLevel::Warning,
        CompatibilitySeverity::Additive => AnnotationLevel::Notice,
    }
}

fn annotation_message(
    finding: &CompatibilityFinding,
    mapping: Option<&SourceMappingEntry>,
) -> String {
    let source_message = match mapping.and_then(|entry| entry.location.as_ref()) {
        Some(_) => {
            "FSH definition-range mapping via SUSHI fsh-index.json; exact rule-line attribution not proven"
        }
        None if mapping.is_some() => "artifact-level finding; no proven current FSH source mapping",
        None => "artifact-level finding; source mapping deferred to CF-09",
    };
    let mut parts = vec![
        source_message.to_owned(),
        format!("severity={}", severity_name(finding.severity)),
        format!("direction={}", direction_name(finding.direction)),
        format!("change={}", change_kind_name(finding.source_kind)),
        format!(
            "resource={}:{}",
            resource_kind_name(finding.resource.kind),
            finding.resource.value
        ),
    ];

    if let Some(before_filename) = &finding.before_filename {
        parts.push(format!("before_file={before_filename}"));
    }
    if let Some(after_filename) = &finding.after_filename {
        parts.push(format!("after_file={after_filename}"));
    }
    if let Some(view) = finding.view {
        parts.push(format!("view={}", view_name(view)));
    }
    if let Some(element_id) = &finding.element_id {
        parts.push(format!("element={element_id}"));
    }
    if let Some(field) = &finding.field {
        parts.push(format!("field={field}"));
    }
    parts.push(finding.message.clone());
    parts.join(" | ")
}

fn bounded_title(value: &str) -> String {
    const SUFFIX: &str = "… [title truncated]";
    if value.chars().count() <= MAX_ANNOTATION_TITLE_CHARS {
        return value.to_owned();
    }
    let mut truncated = value
        .chars()
        .take(MAX_ANNOTATION_TITLE_CHARS.saturating_sub(SUFFIX.chars().count()))
        .collect::<String>();
    truncated.push_str(SUFFIX);
    truncated
}

fn bounded_message(value: &str) -> String {
    const SUFFIX: &str = "… [projection truncated]";
    if value.chars().count() <= MAX_ANNOTATION_MESSAGE_CHARS {
        return value.to_owned();
    }
    let mut truncated = value
        .chars()
        .take(MAX_ANNOTATION_MESSAGE_CHARS.saturating_sub(SUFFIX.chars().count()))
        .collect::<String>();
    truncated.push_str(SUFFIX);
    truncated
}

fn escape_data(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn escape_property(value: &str) -> String {
    escape_data(value).replace(':', "%3A").replace(',', "%2C")
}

fn severity_name(value: CompatibilitySeverity) -> &'static str {
    match value {
        CompatibilitySeverity::Breaking => "BREAKING",
        CompatibilitySeverity::Risky => "RISKY",
        CompatibilitySeverity::Additive => "ADDITIVE",
    }
}

fn direction_name(value: CompatibilityDirection) -> &'static str {
    match value {
        CompatibilityDirection::Producer => "producer",
        CompatibilityDirection::Consumer => "consumer",
    }
}

fn resource_kind_name(value: ResourceKeyKind) -> &'static str {
    match value {
        ResourceKeyKind::Canonical => "canonical",
        ResourceKeyKind::ResourceId => "resource_id",
        ResourceKeyKind::Filename => "filename",
    }
}

fn view_name(value: ElementView) -> &'static str {
    match value {
        ElementView::Snapshot => "snapshot",
        ElementView::Differential => "differential",
    }
}

fn change_kind_name(value: StructuralChangeKind) -> &'static str {
    match value {
        StructuralChangeKind::ResourceAdded => "resource_added",
        StructuralChangeKind::ResourceRemoved => "resource_removed",
        StructuralChangeKind::ResourceFilenameChanged => "resource_filename_changed",
        StructuralChangeKind::ResourceVersionChanged => "resource_version_changed",
        StructuralChangeKind::ResourceTypeChanged => "resource_type_changed",
        StructuralChangeKind::ResourceIdChanged => "resource_id_changed",
        StructuralChangeKind::ResourceBytesChanged => "resource_bytes_changed",
        StructuralChangeKind::StructureFieldChanged => "structure_field_changed",
        StructuralChangeKind::ViewAdded => "view_added",
        StructuralChangeKind::ViewRemoved => "view_removed",
        StructuralChangeKind::ElementAdded => "element_added",
        StructuralChangeKind::ElementRemoved => "element_removed",
        StructuralChangeKind::ElementFieldChanged => "element_field_changed",
    }
}
