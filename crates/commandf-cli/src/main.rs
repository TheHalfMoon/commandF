use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
use commandf_pkg::{
    build_terminology_diff_report, check_report_to_github_annotations_bytes,
    check_report_to_sarif_bytes, classify_structural_diff, diff_package_archives,
    evaluate_compatibility_policy, inspect_package, CheckDirection, CheckFailOn, CheckPolicy,
    CheckReport, FhirRegistrySource, LocalMirrorSource, LockedPackage, Lockfile, PackageCache,
    PackageName, PackageRequest, Resolver, StructuralDiffReport, TerminologyDiffReport,
    TerminologyPackageState, VersionConstraint,
};

const MAX_CHECK_REPORT_INPUT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Parser)]
#[command(
    name = "commandf",
    version,
    about = "Healthcare interoperability change intelligence"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Pkg {
        #[command(subcommand)]
        command: PkgCommand,
    },
    Inspect {
        package: String,
        #[arg(long, default_value = ".commandf/cache")]
        cache: PathBuf,
        #[arg(long, default_value = "commandf.lock")]
        lock: PathBuf,
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    Diff {
        package: String,
        #[arg(long)]
        before_lock: PathBuf,
        #[arg(long)]
        before_cache: PathBuf,
        #[arg(long)]
        after_lock: PathBuf,
        #[arg(long)]
        after_cache: PathBuf,
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    Classify {
        package: String,
        #[arg(long)]
        before_lock: PathBuf,
        #[arg(long)]
        before_cache: PathBuf,
        #[arg(long)]
        after_lock: PathBuf,
        #[arg(long)]
        after_cache: PathBuf,
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    Check {
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
        direction: CheckDirectionArg,
        #[arg(long, value_enum, default_value = "breaking")]
        fail_on: CheckFailOnArg,
        #[arg(long, value_enum, default_value = "json")]
        format: CheckOutputFormat,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Terminology {
        package: String,
        #[arg(long)]
        before_lock: PathBuf,
        #[arg(long)]
        before_cache: PathBuf,
        #[arg(long)]
        after_lock: PathBuf,
        #[arg(long)]
        after_cache: PathBuf,
        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
    GithubAnnotations {
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckOutputFormat {
    Json,
    Sarif,
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckDirectionArg {
    Both,
    Producer,
    Consumer,
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFailOnArg {
    Breaking,
    Risky,
    None,
}

impl From<CheckDirectionArg> for CheckDirection {
    fn from(value: CheckDirectionArg) -> Self {
        match value {
            CheckDirectionArg::Both => Self::Both,
            CheckDirectionArg::Producer => Self::Producer,
            CheckDirectionArg::Consumer => Self::Consumer,
        }
    }
}

impl From<CheckFailOnArg> for CheckFailOn {
    fn from(value: CheckFailOnArg) -> Self {
        match value {
            CheckFailOnArg::Breaking => Self::Breaking,
            CheckFailOnArg::Risky => Self::Risky,
            CheckFailOnArg::None => Self::None,
        }
    }
}

#[derive(Subcommand)]
enum PkgCommand {
    Resolve {
        #[arg(required = true)]
        packages: Vec<String>,
        #[arg(long)]
        source_dir: Option<PathBuf>,
        #[arg(long, default_value = ".commandf/cache")]
        cache: PathBuf,
        #[arg(long, default_value = "commandf.lock")]
        lock: PathBuf,
    },
    Verify {
        #[arg(long, default_value = ".commandf/cache")]
        cache: PathBuf,
        #[arg(long, default_value = "commandf.lock")]
        lock: PathBuf,
    },
}

fn main() -> ExitCode {
    let is_check = std::env::args_os().nth(1).as_deref() == Some(OsStr::new("check"));
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let clap_exit = error.exit_code();
            let _ = error.print();
            if clap_exit == 0 {
                return ExitCode::SUCCESS;
            }
            if is_check {
                return ExitCode::from(1);
            }
            return ExitCode::from(clap_exit as u8);
        }
    };

    match run(cli) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("commandf: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match cli.command {
        Command::Pkg { command } => match command {
            PkgCommand::Resolve {
                packages,
                source_dir,
                cache,
                lock,
            } => {
                let requests = packages
                    .iter()
                    .map(|package| PackageRequest::parse(package))
                    .collect::<Result<Vec<_>, _>>()?;
                let cache = PackageCache::new(cache);
                let lockfile = if let Some(source_dir) = source_dir {
                    Resolver::new(&LocalMirrorSource::new(source_dir), &cache).resolve(requests)?
                } else {
                    Resolver::new(&FhirRegistrySource::new(), &cache).resolve(requests)?
                };
                fs::write(&lock, lockfile.to_bytes()?)?;
                println!(
                    "wrote {} packages to {}",
                    lockfile.packages.len(),
                    lock.display()
                );
            }
            PkgCommand::Verify { cache, lock } => {
                let lockfile = Lockfile::from_slice(&fs::read(&lock)?)?;
                let cache = PackageCache::new(cache);
                lockfile.verify_cache(&cache)?;
                println!("verified {} packages", lockfile.packages.len());
            }
        },
        Command::Inspect {
            package,
            cache,
            lock,
            format,
        } => {
            let request = PackageRequest::parse(&package)?;
            let version = match request.constraint {
                VersionConstraint::Exact(version) => version.to_string(),
                VersionConstraint::PatchWildcard { .. } => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "inspect requires an exact locked package version",
                    )
                    .into());
                }
            };
            let package_name = request.name.to_string();
            let lockfile = Lockfile::from_slice(&fs::read(&lock)?)?;
            let locked = lockfile
                .packages
                .iter()
                .find(|candidate| candidate.name == package_name && candidate.version == version)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("package {package_name}@{version} is not present in the lockfile"),
                    )
                })?;
            let cache = PackageCache::new(cache);
            cache.verify(&locked.sha256)?;
            let archive_path = cache
                .root()
                .join("sha256")
                .join(format!("{}.tgz", locked.sha256));
            let archive_bytes = fs::read(archive_path)?;
            let inspection = inspect_package(
                &locked.name,
                &locked.version,
                &locked.sha256,
                &archive_bytes,
            )?;
            match format {
                OutputFormat::Json => io::stdout().write_all(&inspection.to_json_bytes()?)?,
            }
        }
        Command::Diff {
            package,
            before_lock,
            before_cache,
            after_lock,
            after_cache,
            format,
        } => {
            let report =
                build_diff_report(package, before_lock, before_cache, after_lock, after_cache)?;
            match format {
                OutputFormat::Json => io::stdout().write_all(&report.to_json_bytes()?)?,
            }
        }
        Command::Classify {
            package,
            before_lock,
            before_cache,
            after_lock,
            after_cache,
            format,
        } => {
            let diff =
                build_diff_report(package, before_lock, before_cache, after_lock, after_cache)?;
            let report = classify_structural_diff(&diff)?;
            match format {
                OutputFormat::Json => io::stdout().write_all(&report.to_json_bytes()?)?,
            }
        }
        Command::Check {
            package,
            before_lock,
            before_cache,
            after_lock,
            after_cache,
            direction,
            fail_on,
            format,
            output,
        } => {
            let diff =
                build_diff_report(package, before_lock, before_cache, after_lock, after_cache)?;
            let compatibility = classify_structural_diff(&diff)?;
            let report = evaluate_compatibility_policy(
                &compatibility,
                CheckPolicy {
                    direction: direction.into(),
                    fail_on: fail_on.into(),
                },
            )?;
            let bytes = match format {
                CheckOutputFormat::Json => report.to_json_bytes()?,
                CheckOutputFormat::Sarif => check_report_to_sarif_bytes(&report)?,
            };
            write_check_output(&bytes, output.as_deref())?;
            if report.decision.passed {
                return Ok(ExitCode::SUCCESS);
            }
            return Ok(ExitCode::from(2));
        }
        Command::Terminology {
            package,
            before_lock,
            before_cache,
            after_lock,
            after_cache,
            format,
        } => {
            let report = build_terminology_report(
                package,
                before_lock,
                before_cache,
                after_lock,
                after_cache,
            )?;
            match format {
                OutputFormat::Json => io::stdout().write_all(&report.to_json_bytes()?)?,
            }
        }
        Command::GithubAnnotations { input } => {
            let bytes = read_bounded_file(&input, MAX_CHECK_REPORT_INPUT_BYTES)?;
            let report = CheckReport::from_json_slice(&bytes)?;
            let annotations = check_report_to_github_annotations_bytes(&report)?;
            io::stdout().write_all(&annotations)?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn read_bounded_file(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("input exceeds {max_bytes} byte limit: {}", path.display()),
        ));
    }
    Ok(bytes)
}

fn write_check_output(bytes: &[u8], output: Option<&Path>) -> io::Result<()> {
    let Some(path) = output else {
        return io::stdout().write_all(bytes);
    };
    write_atomic_replace(path, bytes)
}

fn write_atomic_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "output parent directory does not exist: {}",
                parent.display()
            ),
        ));
    }
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "output path has no file name")
    })?;

    for attempt in 0..1000_u32 {
        let temporary = parent.join(format!(
            ".{}.commandf-tmp-{}-{attempt}",
            file_name.to_string_lossy(),
            process::id()
        ));
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };

        let result = (|| -> io::Result<()> {
            file.write_all(bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, path)?;
            Ok(())
        })();
        let _ = fs::remove_file(&temporary);
        return result;
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "unable to allocate a temporary output path",
    ))
}

fn build_diff_report(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
) -> Result<StructuralDiffReport, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    let before_lockfile = Lockfile::from_slice(&fs::read(before_lock)?)?;
    let after_lockfile = Lockfile::from_slice(&fs::read(after_lock)?)?;
    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;

    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);
    before_cache.verify(&before_locked.sha256)?;
    after_cache.verify(&after_locked.sha256)?;

    let before_bytes = fs::read(
        before_cache
            .root()
            .join("sha256")
            .join(format!("{}.tgz", before_locked.sha256)),
    )?;
    let after_bytes = fs::read(
        after_cache
            .root()
            .join("sha256")
            .join(format!("{}.tgz", after_locked.sha256)),
    )?;
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

fn build_terminology_report(
    package: String,
    before_lock: PathBuf,
    before_cache: PathBuf,
    after_lock: PathBuf,
    after_cache: PathBuf,
) -> Result<TerminologyDiffReport, Box<dyn std::error::Error>> {
    let package_name = PackageName::parse(package)?;
    let before_lockfile = Lockfile::from_slice(&fs::read(before_lock)?)?;
    let after_lockfile = Lockfile::from_slice(&fs::read(after_lock)?)?;
    let before_locked = select_locked_package(&before_lockfile, package_name.as_str())?;
    let after_locked = select_locked_package(&after_lockfile, package_name.as_str())?;
    let before_cache = PackageCache::new(before_cache);
    let after_cache = PackageCache::new(after_cache);

    before_cache.verify(&before_locked.sha256)?;
    after_cache.verify(&after_locked.sha256)?;
    let before_bytes = read_locked_archive(&before_cache, before_locked)?;
    let after_bytes = read_locked_archive(&after_cache, after_locked)?;
    let diff = diff_package_archives(
        package_name.to_string(),
        &before_locked.version,
        &before_locked.sha256,
        &before_bytes,
        &after_locked.version,
        &after_locked.sha256,
        &after_bytes,
    )?;
    let compatibility = classify_structural_diff(&diff)?;
    Ok(build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: &before_lockfile,
            cache: &before_cache,
            root_bytes: &before_bytes,
        },
        TerminologyPackageState {
            lockfile: &after_lockfile,
            cache: &after_cache,
            root_bytes: &after_bytes,
        },
        &diff,
        &compatibility,
    )?)
}

fn read_locked_archive(cache: &PackageCache, locked: &LockedPackage) -> Result<Vec<u8>, io::Error> {
    fs::read(
        cache
            .root()
            .join("sha256")
            .join(format!("{}.tgz", locked.sha256)),
    )
}

fn select_locked_package<'a>(
    lockfile: &'a Lockfile,
    package_name: &str,
) -> Result<&'a LockedPackage, io::Error> {
    let mut matches = lockfile
        .packages
        .iter()
        .filter(|candidate| candidate.name == package_name);
    let selected = matches.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("package {package_name} is not present in the lockfile"),
        )
    })?;
    if matches.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("package {package_name} appears more than once in the lockfile"),
        ));
    }
    Ok(selected)
}
