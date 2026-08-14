use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use commandf_pkg::{
    classify_structural_diff, diff_package_archives, inspect_package, FhirRegistrySource,
    LocalMirrorSource, LockedPackage, Lockfile, PackageCache, PackageName, PackageRequest, Resolver,
    StructuralDiffReport, VersionConstraint,
};

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
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
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
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("commandf: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
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
            let report = build_diff_report(
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
        Command::Classify {
            package,
            before_lock,
            before_cache,
            after_lock,
            after_cache,
            format,
        } => {
            let diff = build_diff_report(
                package,
                before_lock,
                before_cache,
                after_lock,
                after_cache,
            )?;
            let report = classify_structural_diff(&diff)?;
            match format {
                OutputFormat::Json => io::stdout().write_all(&report.to_json_bytes()?)?,
            }
        }
    }
    Ok(())
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
