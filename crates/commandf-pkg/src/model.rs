use std::collections::BTreeMap;
use std::fmt;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::PackageError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PackageName(String);

impl PackageName {
    pub fn parse(value: impl Into<String>) -> Result<Self, PackageError> {
        let value = value.into();
        let mut count = 0usize;
        for part in value.split('.') {
            count += 1;
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return Err(PackageError::InvalidPackageName(value));
            };
            if !first.is_ascii_lowercase() {
                return Err(PackageError::InvalidPackageName(value));
            }
            if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
                return Err(PackageError::InvalidPackageName(value));
            }
        }
        if count < 2 {
            return Err(PackageError::InvalidPackageName(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VersionConstraint {
    Exact(Version),
    PatchWildcard { major: u64, minor: u64 },
}

impl VersionConstraint {
    pub fn parse(name: &PackageName, raw: &str) -> Result<Self, PackageError> {
        if let Some(prefix) = raw.strip_suffix(".x") {
            let mut parts = prefix.split('.');
            let major = parts.next().and_then(|v| v.parse::<u64>().ok());
            let minor = parts.next().and_then(|v| v.parse::<u64>().ok());
            if parts.next().is_none() {
                if let (Some(major), Some(minor)) = (major, minor) {
                    return Ok(Self::PatchWildcard { major, minor });
                }
            }
            return Err(PackageError::UnsupportedConstraint {
                name: name.to_string(),
                constraint: raw.to_owned(),
            });
        }

        if raw.contains('*')
            || raw.contains('^')
            || raw.contains('~')
            || raw.contains('>')
            || raw.contains('<')
            || raw.starts_with("file:")
            || raw.starts_with("git")
            || raw.starts_with('.')
            || raw.starts_with('/')
        {
            return Err(PackageError::UnsupportedConstraint {
                name: name.to_string(),
                constraint: raw.to_owned(),
            });
        }

        Ok(Self::Exact(Version::parse(raw)?))
    }

    pub fn matches(&self, version: &Version) -> bool {
        match self {
            Self::Exact(expected) => expected == version,
            Self::PatchWildcard { major, minor } => {
                version.major == *major && version.minor == *minor && version.pre.is_empty()
            }
        }
    }
}

impl fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(version) => version.fmt(f),
            Self::PatchWildcard { major, minor } => write!(f, "{major}.{minor}.x"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRequest {
    pub name: PackageName,
    pub constraint: VersionConstraint,
}

impl PackageRequest {
    pub fn parse(raw: &str) -> Result<Self, PackageError> {
        let Some((name, version)) = raw.rsplit_once('@') else {
            return Err(PackageError::InvalidRequest(raw.to_owned()));
        };
        if name.is_empty() || version.is_empty() {
            return Err(PackageError::InvalidRequest(raw.to_owned()));
        }
        let name = PackageName::parse(name)?;
        let constraint = VersionConstraint::parse(&name, version)?;
        Ok(Self { name, constraint })
    }

    pub fn display(&self) -> String {
        format!("{}@{}", self.name, self.constraint)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}
