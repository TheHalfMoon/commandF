use std::collections::{BTreeMap, VecDeque};

use semver::Version;

use crate::archive::read_manifest;
use crate::lock::{LockedPackage, Lockfile};
use crate::{PackageCache, PackageError, PackageRequest, PackageSource, VersionConstraint};

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
        let mut queue: VecDeque<PackageRequest> = roots.into();
        let mut selected: BTreeMap<String, LockedPackage> = BTreeMap::new();

        while let Some(request) = queue.pop_front() {
            if let Some(existing) = selected.get(request.name.as_str()) {
                let selected_version = Version::parse(&existing.version)?;
                if request.constraint.matches(&selected_version) {
                    continue;
                }
                return Err(PackageError::VersionConflict {
                    name: request.name.to_string(),
                    selected: existing.version.clone(),
                    requested: request.constraint.to_string(),
                });
            }

            let version = self.select_version(&request)?;
            let archive = self.source.archive(&request.name, &version)?;
            let manifest = read_manifest(&archive)?;
            let manifest_version = Version::parse(&manifest.version)?;
            let expected = format!("{}@{}", request.name, version);
            let found = format!("{}@{}", manifest.name, manifest_version);
            if manifest.name != request.name.as_str() || manifest_version != version {
                return Err(PackageError::IdentityMismatch { expected, found });
            }

            let digest = self.cache.put(&archive)?;
            let dependencies = manifest.dependencies;
            selected.insert(
                request.name.to_string(),
                LockedPackage {
                    name: request.name.to_string(),
                    version: version.to_string(),
                    sha256: digest,
                    source: self.source.source_id(),
                    dependencies: dependencies.clone(),
                },
            );

            for (name, constraint) in dependencies {
                let name = crate::PackageName::parse(name)?;
                let constraint = VersionConstraint::parse(&name, &constraint)?;
                queue.push_back(PackageRequest { name, constraint });
            }
        }

        Ok(Lockfile::new(root_labels, selected.into_values().collect()))
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
