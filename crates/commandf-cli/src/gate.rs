use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, ValueEnum};
use commandf_pkg::{
    classify_structural_diff, diff_package_archives, evaluate_compatibility_policy,
    evaluate_quality_gate, CheckDirection, CheckFailOn, CheckPolicy, CheckReport, GateSuppressions,
    Lockfile, PackageCache, PackageName, StructuralDiffReport,
};

use super::{read_bounded_file, select_locked_package, write_check_output};

const MAX_GATE_LOCKFILE_INPUT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_GATE_ARCHIVE_INPUT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_GATE_BASELINE_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_GATE_SUPPRESSIONS_INPUT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Args)]
pub(crate) struct GateArgs {
    package: String,
    #[arg(long)]
    before_lock: PathBuf,
    #[arg(long)]
    before_cache: PathBuf,
    #[arg(long)]
    after_lock: PathBuf,
    #[arg(long)]
    after_cache: PathBuf,
    #[arg(long, value_enum, default_value = "both")]
    direction: GateDirectionArg,
    #[arg(long, value_enum, default_value = "breaking")]
    fail_on: GateFailOnArg,
    #[arg(long)]
    baseline: Option<PathBuf>,
    #[arg(long)]
    suppressions: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "json")]
    format: GateOutputFormat,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum GateOutputFormat {
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum GateDirectionArg {
    Both,
    Producer,
    Consumer,
}

#[derive(Clone, Copy, ValueEnum)]
enum GateFailOnArg {
    Breaking,
    Risky,
    None,
}

impl From<GateDirectionArg> for CheckDirection {
    fn from(value: GateDirectionArg) -> Self {
        match value {
            GateDirectionArg::Both => Self::Both,
            GateDirectionArg::Producer => Self::Producer,
            GateDirectionArg::Consumer => Self::Consumer,
        }
    }
}

impl From<GateFailOnArg> for CheckFailOn {
    fn from(value: GateFailOnArg) -> Self {
        match value {
            GateFailOnArg::Breaking => Self::Breaking,
            GateFailOnArg::Risky => Self::Risky,
            GateFailOnArg::None => Self::None,
        }
    }
}

pub(crate) fn run(args: GateArgs) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let diff = build_gate_diff_report(
        args.package,
        args.before_lock,
        args.before_cache,
        args.after_lock,
        args.after_cache,
    )?;
    let compatibility = classify_structural_diff(&diff)?;
    let current = evaluate_compatibility_policy(
        &compatibility,
        CheckPolicy {
            direction: args.direction.into(),
            fail_on: args.fail_on.into(),
        },
    )?;

    let baseline = args
        .baseline
        .as_deref()
        .map(|path| {
            let bytes = read_bounded_file(path, MAX_GATE_BASELINE_INPUT_BYTES)?;
            Ok::<_, Box<dyn std::error::Error>>(CheckReport::from_json_slice(&bytes)?)
        })
        .transpose()?;
    let suppressions = args
        .suppressions
        .as_deref()
        .map(|path| {
            let bytes = read_bounded_file(path, MAX_GATE_SUPPRESSIONS_INPUT_BYTES)?;
            Ok::<_, Box<dyn std::error::Error>>(GateSuppressions::from_json_slice(&bytes)?)
        })
        .transpose()?;

    let report = evaluate_quality_gate(&current, baseline.as_ref(), suppressions.as_ref())?;
    let bytes = match args.format {
        GateOutputFormat::Json => report.to_json_bytes()?,
    };
    write_check_output(&bytes, args.output.as_deref())?;

    if report.decision.passed {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(2))
    }
}

fn build_gate_diff_report(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
) -> Result<StructuralDiffReport, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    let before_lockfile = Lockfile::from_slice(&read_bounded_file(
        &before_lock,
        MAX_GATE_LOCKFILE_INPUT_BYTES,
    )?)?;
    let after_lockfile = Lockfile::from_slice(&read_bounded_file(
        &after_lock,
        MAX_GATE_LOCKFILE_INPUT_BYTES,
    )?)?;
    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;

    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);
    let before_bytes =
        before_cache.read_verified_bounded(&before_locked.sha256, MAX_GATE_ARCHIVE_INPUT_BYTES)?;
    let after_bytes =
        after_cache.read_verified_bounded(&after_locked.sha256, MAX_GATE_ARCHIVE_INPUT_BYTES)?;

    Ok(diff_package_archives(
        package_name.to_string(),
        &before_locked.version,
        &before_locked.sha256,
        &before_bytes,
        &after_locked.version,
        &after_locked.sha256,
        &after_bytes,
    )?)
}
