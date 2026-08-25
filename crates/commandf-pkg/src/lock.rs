use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{PackageCache, PackageError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    pub schema: u32,
    pub roots: Vec<String>,
    pub packages: Vec<LockedPackage>,
    #[serde(default)]
    pub resolved_dependencies: Vec<ResolvedDependency>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub sha256: String,
    pub source: String,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub from_name: String,
    pub from_version: String,
    pub to_name: String,
    pub to_version: String,
    pub declared_constraint: String,
}

#[derive(Deserialize)]
struct RawLockfile {
    schema: u32,
    roots: Vec<String>,
    packages: Vec<LockedPackage>,
    resolved_dependencies: Option<Vec<ResolvedDependency>>,
}

#[derive(Serialize)]
struct LockfileV1<'a> {
    schema: u32,
    roots: &'a [String],
    packages: &'a [LockedPackage],
}

#[derive(Serialize)]
struct LockfileV2<'a> {
    schema: u32,
    roots: &'a [String],
    packages: &'a [LockedPackage],
    resolved_dependencies: &'a [ResolvedDependency],
}

impl Lockfile {
    pub const SCHEMA_V1: u32 = 1;
    pub const SCHEMA_V2: u32 = 2;

    pub fn new(mut roots: Vec<String>, mut packages: Vec<LockedPackage>) -> Self {
        canonicalize_roots_and_packages(&mut roots, &mut packages);
        Self {
            schema: Self::SCHEMA_V1,
            roots,
            packages,
            resolved_dependencies: Vec::new(),
        }
    }

    pub fn new_v2(
        mut roots: Vec<String>,
        mut packages: Vec<LockedPackage>,
        mut resolved_dependencies: Vec<ResolvedDependency>,
    ) -> Self {
        canonicalize_roots_and_packages(&mut roots, &mut packages);
        resolved_dependencies.sort();
        resolved_dependencies.dedup();
        Self {
            schema: Self::SCHEMA_V2,
            roots,
            packages,
            resolved_dependencies,
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, PackageError> {
        let mut bytes = match self.schema {
            Self::SCHEMA_V1 => serde_json::to_vec_pretty(&LockfileV1 {
                schema: self.schema,
                roots: &self.roots,
                packages: &self.packages,
            })?,
            Self::SCHEMA_V2 => {
                self.validate_v2()?;
                serde_json::to_vec_pretty(&LockfileV2 {
                    schema: self.schema,
                    roots: &self.roots,
                    packages: &self.packages,
                    resolved_dependencies: &self.resolved_dependencies,
                })?
            }
            found => {
                return Err(PackageError::UnsupportedLockSchema {
                    found,
                    expected: Self::SCHEMA_V2,
                })
            }
        };
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, PackageError> {
        let raw: RawLockfile = serde_json::from_slice(bytes)?;
        match raw.schema {
            Self::SCHEMA_V1 => {
                if raw.resolved_dependencies.is_some() {
                    return Err(PackageError::InvalidLockfile(
                        "schema v1 must not contain resolved_dependencies".to_owned(),
                    ));
                }
                Ok(Self {
                    schema: raw.schema,
                    roots: raw.roots,
                    packages: raw.packages,
                    resolved_dependencies: Vec::new(),
                })
            }
            Self::SCHEMA_V2 => {
                let resolved_dependencies = raw.resolved_dependencies.ok_or_else(|| {
                    PackageError::InvalidLockfile(
                        "schema v2 requires resolved_dependencies".to_owned(),
                    )
                })?;
                let lockfile = Self {
                    schema: raw.schema,
                    roots: raw.roots,
                    packages: raw.packages,
                    resolved_dependencies,
                };
                lockfile.validate_v2()?;
                Ok(lockfile)
            }
            found => Err(PackageError::UnsupportedLockSchema {
                found,
                expected: Self::SCHEMA_V2,
            }),
        }
    }

    pub fn verify_cache(&self, cache: &PackageCache) -> Result<(), PackageError> {
        for package in &self.packages {
            cache.verify(&package.sha256)?;
        }
        Ok(())
    }

    fn validate_v2(&self) -> Result<(), PackageError> {
        let mut canonical_roots = self.roots.clone();
        canonical_roots.sort();
        canonical_roots.dedup();
        if canonical_roots != self.roots {
            return Err(PackageError::InvalidLockfile(
                "schema v2 roots must be sorted and deduplicated".to_owned(),
            ));
        }

        let mut package_order = self
            .packages
            .iter()
            .map(|package| (package.name.as_str(), package.version.as_str()))
            .collect::<Vec<_>>();
        let package_identities = package_order.iter().copied().collect::<BTreeSet<_>>();
        if package_identities.len() != self.packages.len() {
            return Err(PackageError::InvalidLockfile(
                "schema v2 packages contain a duplicate exact identity".to_owned(),
            ));
        }
        package_order.sort();
        if package_order
            != self
                .packages
                .iter()
                .map(|package| (package.name.as_str(), package.version.as_str()))
                .collect::<Vec<_>>()
        {
            return Err(PackageError::InvalidLockfile(
                "schema v2 packages must be sorted by name and version".to_owned(),
            ));
        }

        let mut canonical_edges = self.resolved_dependencies.clone();
        canonical_edges.sort();
        canonical_edges.dedup();
        if canonical_edges != self.resolved_dependencies {
            return Err(PackageError::InvalidLockfile(
                "schema v2 resolved_dependencies must be sorted and deduplicated".to_owned(),
            ));
        }

        let packages_by_identity = self
            .packages
            .iter()
            .map(|package| ((package.name.as_str(), package.version.as_str()), package))
            .collect::<BTreeMap<_, _>>();
        let mut covered_dependencies = BTreeSet::new();

        for edge in &self.resolved_dependencies {
            let parent = packages_by_identity
                .get(&(edge.from_name.as_str(), edge.from_version.as_str()))
                .ok_or_else(|| {
                    PackageError::InvalidLockfile(format!(
                        "resolved dependency source {}@{} is not present in packages",
                        edge.from_name, edge.from_version
                    ))
                })?;
            if !package_identities.contains(&(edge.to_name.as_str(), edge.to_version.as_str())) {
                return Err(PackageError::InvalidLockfile(format!(
                    "resolved dependency target {}@{} is not present in packages",
                    edge.to_name, edge.to_version
                )));
            }
            if edge.declared_constraint.is_empty() {
                return Err(PackageError::InvalidLockfile(format!(
                    "resolved dependency {}@{} -> {}@{} has an empty declared constraint",
                    edge.from_name, edge.from_version, edge.to_name, edge.to_version
                )));
            }

            let manifest_constraint = parent.dependencies.get(&edge.to_name).ok_or_else(|| {
                PackageError::InvalidLockfile(format!(
                    "resolved dependency {}@{} -> {}@{} is not declared by the source package manifest",
                    edge.from_name, edge.from_version, edge.to_name, edge.to_version
                ))
            })?;
            if manifest_constraint != &edge.declared_constraint {
                return Err(PackageError::InvalidLockfile(format!(
                    "resolved dependency {}@{} -> {}@{} records constraint {:?}, but the source package declares {:?}",
                    edge.from_name,
                    edge.from_version,
                    edge.to_name,
                    edge.to_version,
                    edge.declared_constraint,
                    manifest_constraint
                )));
            }

            let dependency_key = (
                edge.from_name.as_str(),
                edge.from_version.as_str(),
                edge.to_name.as_str(),
            );
            if !covered_dependencies.insert(dependency_key) {
                return Err(PackageError::InvalidLockfile(format!(
                    "schema v2 records more than one resolved target for dependency {}@{} -> {}",
                    edge.from_name, edge.from_version, edge.to_name
                )));
            }
        }

        for package in &self.packages {
            for dependency_name in package.dependencies.keys() {
                if !covered_dependencies.contains(&(
                    package.name.as_str(),
                    package.version.as_str(),
                    dependency_name.as_str(),
                )) {
                    return Err(PackageError::InvalidLockfile(format!(
                        "schema v2 is missing resolved dependency evidence for {}@{} -> {}",
                        package.name, package.version, dependency_name
                    )));
                }
            }
        }

        Ok(())
    }
}

fn canonicalize_roots_and_packages(roots: &mut Vec<String>, packages: &mut [LockedPackage]) {
    roots.sort();
    roots.dedup();
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
}
