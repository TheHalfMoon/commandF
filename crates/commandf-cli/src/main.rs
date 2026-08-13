use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use commandf_artifact::inspect_package;
use commandf_pkg::{
    FhirRegistrySource, LocalMirrorSource, Lockfile, PackageCache, PackageRequest, Resolver,
    VersionConstraint,
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
        format: InspectFormat,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum InspectFormat {
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
                InspectFormat::Json => io::stdout().write_all(&inspection.to_json_bytes()?)?,
            }
        }
    }
    Ok(())
}
