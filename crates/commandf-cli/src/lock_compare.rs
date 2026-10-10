//! Offline bounded CF11 lockfile divergence attribution CLI adapter.

use std::path::Path;

use commandf_pkg::compare_cf11_lockfiles;

use crate::lock_input;

pub(crate) fn run(
    first_path: &Path,
    second_path: &Path,
) -> Result<(Vec<u8>, bool), Box<dyn std::error::Error>> {
    let first = lock_input::read_lockfile(first_path)?;
    let second = lock_input::read_lockfile(second_path)?;
    Ok(compare_cf11_lockfiles(&first, &second)?)
}
