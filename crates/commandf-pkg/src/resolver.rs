use std::collections::{BTreeMap, BTreeSet, VecDeque};

use semver::Version;

use crate::archive::read_manifest;
use crate::lock::{LockedPackage, Lockfile, ResolvedDependency};
use crate::{PackageCache, PackageError, PackageRequest, PackageSource, VersionConstraint};

type PackageIdentity = (String, String);

struct PendingRequest {
    request: PackageRequest,
    parent: Option<PackageIdentity>,
    declared_constraint: Option<String>,
}

pub struct Resolver<'a, S: PackageSource> {
    source: &'a S,
    cache: &'a PackageCache,
}

impl<'a, S: PackageSource> Resolver<'a, S> {
    pub fn new(source: &'a S, cache: &'a PackageCache) -> Self {
        Self { source, cache }
    }

    pub fn resolve(&self, roots: Vec<PackageRequest>) -> Result<Lockfile, PackageError> {
        let root_labels = roots.iter().map(PackageRequest::display).collect();
        let mut queue = roots
            .into_iter()
            .map(|request| PendingRequest {
                request,
                parent: None,
                declared_constraint: None,
            })
            .collect::<VecDeque<_>>();
        let mut selected: BTreeMap<PackageIdentity, LockedPackage> = BTreeMap::new();
        let mut resolved_dependencies = BTreeSet::new();

        while let Some(pending) = queue.pop_front() {
            let version = self.select_version(&pending.request)?;
            let identity = (pending.request.name.to_string(), version.to_string());

            if let Some((from_name, from_version)) = pending.parent {
                let declared_constraint = pending.declared_constraint.ok_or_else(|| {
                    PackageError::InvalidLockfile(
                        "resolver dependency request is missing its declared constraint".to_owned(),
                    )
                })?;
                resolved_dependencies.insert(ResolvedDependency {
                    from_name,
                    from_version,
                    to_name: identity.0.clone(),
                    to_version: identity.1.clone(),
                    declared_constraint,
                });
            }

            if selected.contains_key(&identity) {
                continue;
            }

            let archive = self
                .source
                .archive_with_source(&pending.request.name, &version)?;
            let manifest = read_manifest(&archive.bytes)?;
            let manifest_version = Version::parse(&manifest.version)?;
            let expected = format!("{}@{}", pending.request.name, version);
            let found = format!("{}@{}", manifest.name, manifest_version);
            if manifest.name != pending.request.name.as_str() || manifest_version != version {
                return Err(PackageError::IdentityMismatch { expected, found });
            }

            let digest = self.cache.put(&archive.bytes)?;
            let dependencies = manifest.dependencies;
            selected.insert(
                identity.clone(),
                LockedPackage {
                    name: pending.request.name.to_string(),
                    version: version.to_string(),
                    sha256: digest,
                    source: archive.source,
                    dependencies: dependencies.clone(),
                },
            );

            for (name, constraint) in dependencies {
                let declared_constraint = constraint.clone();
                let name = crate::PackageName::parse(name)?;
                let constraint = VersionConstraint::parse(&name, &constraint)?;
                queue.push_back(PendingRequest {
                    request: PackageRequest { name, constraint },
                    parent: Some(identity.clone()),
                    declared_constraint: Some(declared_constraint),
                });
            }
        }

        Ok(Lockfile::new_v2(
            root_labels,
            selected.into_values().collect(),
            resolved_dependencies.into_iter().collect(),
        ))
    }

    fn select_version(&self, request: &PackageRequest) -> Result<Version, PackageError> {
        match &request.constraint {
            VersionConstraint::Exact(version) => Ok(version.clone()),
            VersionConstraint::PatchWildcard { .. } => self
                .source
                .available_versions(&request.name)?
                .into_iter()
                .filter(|version| request.constraint.matches(version))
                .max()
                .ok_or_else(|| PackageError::NoMatchingVersion {
                    name: request.name.to_string(),
                    constraint: request.constraint.to_string(),
                }),
        }
    }
}
