use std::fs;
use std::io::Write;
use std::path::PathBuf;

use commandf_af02_verifier::surface_proof::{canonical_surface_proof_bytes, prove_surface};

fn main() {
    if let Err(error) = run() {
        eprintln!("commandf-af02-prove-surface: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
    let exclusion_policy_path = PathBuf::from(args.next().ok_or("missing exclusion policy path")?);
    let source_repo_root = PathBuf::from(args.next().ok_or("missing source repository root")?);
    if args.next().is_some() {
        return Err(
            "prove_surface accepts exactly a surface policy path, exclusion policy path, and source repository root"
                .into(),
        );
    }

    let evidence = prove_surface(
        &fs::read(policy_path)?,
        &fs::read(exclusion_policy_path)?,
        &source_repo_root,
    )?;
    std::io::stdout()
        .lock()
        .write_all(&canonical_surface_proof_bytes(&evidence)?)?;
    Ok(())
}
