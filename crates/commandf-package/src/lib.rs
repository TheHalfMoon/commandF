//! Content-addressed package and lockfile contracts for commandF.

use commandf_csir::ContentHash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageKind {
    Profile,
    Mapping,
    Terminology,
    RuleSet,
    Recipe,
    Connector,
    Benchmark,
    CertificateBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PackageIdentity {
    pub namespace: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageRequirement {
    pub namespace: String,
    pub name: String,
    pub version_requirement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub manifest_schema: String,
    pub identity: PackageIdentity,
    pub kind: PackageKind,
    pub artifact_digest: ContentHash,
    pub media_type: String,
    #[serde(default)]
    pub dependencies: Vec<PackageRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub identity: PackageIdentity,
    pub digest: ContentHash,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lockfile {
    pub lock_schema: String,
    pub root: PackageIdentity,
    #[serde(default)]
    pub packages: Vec<LockedPackage>,
}

impl Lockfile {
    /// Production resolution is valid only when every dependency is pinned to
    /// a concrete content digest. Tags and version strings alone are not proof
    /// of the bytes used by a certification run.
    pub fn is_fully_pinned(&self) -> bool {
        !self.packages.is_empty()
            && self.packages.iter().all(|package| {
                !package.digest.algorithm.trim().is_empty()
                    && !package.digest.value.trim().is_empty()
                    && !package.source.trim().is_empty()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(version: &str) -> PackageIdentity {
        PackageIdentity {
            namespace: "hospital-x".into(),
            name: "lab-map".into(),
            version: version.into(),
        }
    }

    #[test]
    fn lockfile_rejects_unpinned_dependency() {
        let lock = Lockfile {
            lock_schema: "commandf.lock/0".into(),
            root: identity("1.0.0"),
            packages: vec![LockedPackage {
                identity: identity("1.0.0"),
                digest: ContentHash {
                    algorithm: "sha256".into(),
                    value: String::new(),
                },
                source: "oci://registry.example/hospital-x/lab-map".into(),
            }],
        };

        assert!(!lock.is_fully_pinned());
    }

    #[test]
    fn lockfile_accepts_content_addressed_dependency() {
        let lock = Lockfile {
            lock_schema: "commandf.lock/0".into(),
            root: identity("1.0.0"),
            packages: vec![LockedPackage {
                identity: identity("1.0.0"),
                digest: ContentHash {
                    algorithm: "sha256".into(),
                    value: "abc123".into(),
                },
                source: "oci://registry.example/hospital-x/lab-map".into(),
            }],
        };

        assert!(lock.is_fully_pinned());
    }
}
