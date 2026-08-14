use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{PackageCache, PackageError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    pub schema: u32,
    pub roots: Vec<String>,
    pub packages: Vec<LockedPackage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub source: String,
    pub dependencies: BTreeMap<String, String>,
}

impl Lockfile {
    pub const SCHEMA_V1: u32 = 1;

    pub fn new(mut roots: Vec<String>, mut packages: Vec<LockedPackage>) -> Self {
        roots.sort();
        roots.dedup();
        packages.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.version.cmp(&right.version))
        });
        Self {
            schema: Self::SCHEMA_V1,
            roots,
            packages,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, PackageError> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, PackageError> {
        let lockfile: Self = serde_json::from_slice(bytes)?;
        if lockfile.schema != Self::SCHEMA_V1 {
            return Err(PackageError::UnsupportedLockSchema {
                found: lockfile.schema,
                expected: Self::SCHEMA_V1,
            });
        }
        Ok(lockfile)
    }

    pub fn verify_cache(&self, cache: &PackageCache) -> Result<(), PackageError> {
        for package in &self.packages {
            cache.verify(&package.sha256)?;
        }
        Ok(())
    }
}
