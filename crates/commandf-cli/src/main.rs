use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use commandf_pkg::{
    FhirRegistrySource, LocalMirrorSource, Lockfile, PackageCache, PackageRequest, Resolver,
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
    }
    Ok(())
}
