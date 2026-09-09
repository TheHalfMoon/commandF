use std::path::Path;

use serde::Serialize;

use crate::corpus::ExpectedOutcome;
use crate::resource::{run_bounded, ResourceError, ResourcePolicy, RunnerOutcome};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarnessFailure {
    Timeout,
    MemoryLimit,
    FilesystemLimit,
    ProcessLimit,
    InternalError,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "class", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReplayExecution {
    Completed { runner: RunnerOutcome },
    HarnessFailure { failure: HarnessFailure },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SurfaceObservation {
    AcceptCanonical,
    RejectInvalid,
    FailClosedLimit,
    InvariantViolation,
    OracleDivergence,
    PanicOrAbort,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NormalizedOutcome {
    AcceptCanonical,
    RejectInvalid,
    FailClosedLimit,
    UnexpectedAcceptance,
    InvariantViolation,
    OracleDivergence,
    PanicOrAbort,
    HarnessTimeout,
    HarnessMemoryLimit,
    HarnessFilesystemLimit,
    HarnessProcessLimit,
    HarnessInternalError,
}

pub fn run_replay(
    policy: &ResourcePolicy,
    source_dir: &Path,
    output_dir: &Path,
    command: &[String],
) -> ReplayExecution {
    match run_bounded(policy, source_dir, output_dir, command) {
        Ok(runner) => ReplayExecution::Completed { runner },
        Err(error) => ReplayExecution::HarnessFailure {
            failure: classify_resource_error(&error),
        },
    }
}

pub fn normalize_result(
    expected: ExpectedOutcome,
    execution: &ReplayExecution,
    observed: Option<SurfaceObservation>,
) -> NormalizedOutcome {
    if let ReplayExecution::HarnessFailure { failure } = execution {
        return normalize_harness_failure(*failure);
    }

    let Some(observed) = observed else {
        return NormalizedOutcome::HarnessInternalError;
    };

    match observed {
        SurfaceObservation::InvariantViolation => NormalizedOutcome::InvariantViolation,
        SurfaceObservation::OracleDivergence => NormalizedOutcome::OracleDivergence,
        SurfaceObservation::PanicOrAbort => NormalizedOutcome::PanicOrAbort,
        SurfaceObservation::AcceptCanonical
            if matches!(
                expected,
                ExpectedOutcome::RejectInvalid | ExpectedOutcome::FailClosedLimit
            ) =>
        {
            NormalizedOutcome::UnexpectedAcceptance
        }
        SurfaceObservation::AcceptCanonical => NormalizedOutcome::AcceptCanonical,
        SurfaceObservation::RejectInvalid => NormalizedOutcome::RejectInvalid,
        SurfaceObservation::FailClosedLimit => NormalizedOutcome::FailClosedLimit,
    }
}

fn normalize_harness_failure(failure: HarnessFailure) -> NormalizedOutcome {
    match failure {
        HarnessFailure::Timeout => NormalizedOutcome::HarnessTimeout,
        HarnessFailure::MemoryLimit => NormalizedOutcome::HarnessMemoryLimit,
        HarnessFailure::FilesystemLimit => NormalizedOutcome::HarnessFilesystemLimit,
        HarnessFailure::ProcessLimit => NormalizedOutcome::HarnessProcessLimit,
        HarnessFailure::InternalError => NormalizedOutcome::HarnessInternalError,
    }
}

fn classify_resource_error(error: &ResourceError) -> HarnessFailure {
    match error {
        ResourceError::Timeout => HarnessFailure::Timeout,
        ResourceError::Artifact(_) | ResourceError::Path(_) => HarnessFailure::FilesystemLimit,
        ResourceError::RuntimeInspection(message)
            if message.contains("AF02_RESOURCE_PROBE_FAIL=TEMP_FILE_LIMIT")
                || message.contains("AF02_RESOURCE_PROBE_FAIL=TEMP_BYTE_LIMIT") =>
        {
            HarnessFailure::FilesystemLimit
        }
        _ => HarnessFailure::InternalError,
    }
}
