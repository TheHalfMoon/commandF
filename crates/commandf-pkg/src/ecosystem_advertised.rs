use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use semver::Version;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::ecosystem_registry::{PRIMARY_BASE, SECONDARY_BASE, SECONDARY_TARBALL_BASE};
use crate::{
    project_snapshot, require_authorized_registry_url, PackageCache, PackageError, PackageName,
    RegistryHttpResponse, RegistryTransport, SnapshotError, SnapshotPackage, ARCHIVE_LIMIT,
    IMMUTABLE_RELEASE, METADATA_LIMIT, PRIMARY_HOST, SECONDARY_HOST,
};

pub const ECOSYSTEM_ADVERTISED_VERSIONS_SCHEMA: &str = "commandf.ecosystem-advertised-versions/v1";
pub const ADVERTISED_BY_OFFICIAL_SOURCE: &str = "ADVERTISED_BY_OFFICIAL_SOURCE";
pub const MAX_ADVERTISED_VERSIONS: usize = 256;
pub const MAX_ADVERTISED_NAME_BYTES: usize = 256;
pub const MAX_ADVERTISED_VERSION_BYTES: usize = 128;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdvertisedLimits {
    pub metadata_bytes: u64,
    pub archive_bytes: u64,
    pub max_versions: usize,
}

impl AdvertisedLimits {
    pub fn official() -> Self {
        Self {
            metadata_bytes: METADATA_LIMIT,
            archive_bytes: ARCHIVE_LIMIT,
            max_versions: MAX_ADVERTISED_VERSIONS,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvertisedRequest<'a> {
    pub name: &'a str,
    pub expected_listing_sha256: Option<&'a str>,
    pub expected_archives: &'a [(&'a str, &'a str)],
    pub retrieved_at: &'a str,
    pub limits: AdvertisedLimits,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AdvertisedVersionNode {
    pub archive_sha256: String,
    pub snapshot_sha256: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvertisedIdentity {
    pub coverage: String,
    pub listing_sha256: String,
    pub name: String,
    pub schema: String,
    pub source_host: String,
    pub versions: Vec<AdvertisedVersionNode>,
    pub document_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvertisedArchive {
    pub name: String,
    pub version: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvertisedObservation {
    pub identity: AdvertisedIdentity,
    pub listing_bytes: Vec<u8>,
    pub archives: Vec<AdvertisedArchive>,
    pub retrieved_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredArchive<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub bytes: &'a [u8],
    pub archive_sha256: &'a str,
}

#[derive(Debug, Error)]
pub enum AdvertisedError {
    #[error("package version must be an exact semantic version")]
    InvalidVersion,

    #[error("registry host is not an authorized official source")]
    UnauthorizedHost,

    #[error("registry acquisition timed out")]
    Timeout,

    #[error("registry listing or archive is malformed")]
    Malformed,

    #[error("registry redirect is outside the pinned official tarball")]
    Redirect,

    #[error("registry response exceeded the byte limit")]
    Oversized,

    #[error("registry response was truncated")]
    Truncated,

    #[error("registry digest does not match the expected digest")]
    DigestMismatch,

    #[error("duplicate advertised version {version}")]
    DuplicateVersion { version: String },

    #[error("advertised version count exceeds {MAX_ADVERTISED_VERSIONS}")]
    TooManyVersions,

    #[error("package name does not match the listing")]
    NameMismatch,

    #[error("package version does not match the listing")]
    VersionMismatch,

    #[error("the same registry URL was requested more than once")]
    Retry,

    #[error("registry acquisition could not be verified")]
    Unverifiable,

    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),
}

#[derive(Serialize)]
struct IdentityBody<'a> {
    coverage: &'a str,
    listing_sha256: &'a str,
    name: &'a str,
    schema: &'a str,
    source_host: &'a str,
    versions: &'a [AdvertisedVersionNode],
}

#[derive(Deserialize)]
struct ListingDocument {
    versions: UniqueVersions,
}

struct UniqueVersions(BTreeMap<String, serde_json::Value>);

impl<'de> Deserialize<'de> for UniqueVersions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_map(UniqueVersionVisitor)
            .map(UniqueVersions)
    }
}

struct UniqueVersionVisitor;

impl<'de> Visitor<'de> for UniqueVersionVisitor {
    type Value = BTreeMap<String, serde_json::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object of exact package versions")
    }

    fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut versions = BTreeMap::new();
        while let Some((key, value)) = access.next_entry::<String, serde_json::Value>()? {
            if versions.insert(key.clone(), value).is_some() {
                return Err(de::Error::custom(format!("duplicate version {key}")));
            }
        }
        Ok(versions)
    }
}

enum SelectedHost {
    Primary,
    Secondary,
}

struct ListingFetch {
    host: SelectedHost,
    body: Vec<u8>,
    source_host: String,
}

pub fn acquire_advertised_versions<T: RegistryTransport>(
    transport: &mut T,
    cache: Option<&PackageCache>,
    request: &AdvertisedRequest<'_>,
) -> Result<AdvertisedObservation, AdvertisedError> {
    validate_name(request.name)?;
    let mut seen = BTreeSet::new();
    let listing = load_listing(transport, &mut seen, request)?;
    let listing_sha256 = PackageCache::digest(&listing.body);
    if request
        .expected_listing_sha256
        .is_some_and(|expected| expected != listing_sha256)
    {
        return Err(AdvertisedError::DigestMismatch);
    }
    let versions = parse_versions(&listing.body, request.limits.max_versions)?;
    let mut archives = Vec::with_capacity(versions.len());
    let mut nodes = Vec::with_capacity(versions.len());
    for version in versions {
        let archive = fetch_archive(transport, &mut seen, request, &listing.host, &version)?;
        let digest = PackageCache::digest(&archive);
        if request
            .expected_archives
            .iter()
            .any(|(expected_version, expected_digest)| {
                *expected_version == version && *expected_digest != digest
            })
        {
            return Err(AdvertisedError::DigestMismatch);
        }
        let snapshot = project_snapshot(
            vec![SnapshotPackage {
                name: request.name.to_owned(),
                version: version.clone(),
                archive_sha256: digest.clone(),
                source_id: listing.source_host.clone(),
                mutability: IMMUTABLE_RELEASE.to_owned(),
            }],
            Vec::new(),
        )?;
        if let Some(cache) = cache {
            let stored = cache.put(&archive)?;
            if stored != digest {
                return Err(AdvertisedError::DigestMismatch);
            }
        }
        nodes.push(AdvertisedVersionNode {
            archive_sha256: digest,
            snapshot_sha256: snapshot.snapshot_sha256,
            version: version.clone(),
        });
        archives.push(AdvertisedArchive {
            name: request.name.to_owned(),
            version,
            bytes: archive,
        });
    }
    if let Some(cache) = cache {
        let stored = cache.put(&listing.body)?;
        if stored != listing_sha256 {
            return Err(AdvertisedError::DigestMismatch);
        }
    }
    let identity = identity_from(request.name, &listing.source_host, &listing_sha256, nodes)?;
    Ok(AdvertisedObservation {
        identity,
        listing_bytes: listing.body,
        archives,
        retrieved_at: request.retrieved_at.to_owned(),
    })
}

pub fn replay_advertised_versions(
    name: &str,
    source_host: &str,
    listing_bytes: &[u8],
    listing_sha256: &str,
    archives: &[StoredArchive<'_>],
    max_versions: usize,
) -> Result<AdvertisedIdentity, AdvertisedError> {
    validate_name(name)?;
    if source_host != PRIMARY_HOST && source_host != SECONDARY_HOST {
        return Err(AdvertisedError::UnauthorizedHost);
    }
    if PackageCache::digest(listing_bytes) != listing_sha256 {
        return Err(AdvertisedError::DigestMismatch);
    }
    let versions = parse_versions(listing_bytes, max_versions)?;
    if archives.len() != versions.len() {
        return Err(AdvertisedError::VersionMismatch);
    }
    let mut nodes = Vec::with_capacity(versions.len());
    for version in &versions {
        let archive = archives
            .iter()
            .find(|archive| archive.version == version)
            .ok_or(AdvertisedError::VersionMismatch)?;
        if archive.name != name {
            return Err(AdvertisedError::NameMismatch);
        }
        if PackageCache::digest(archive.bytes) != archive.archive_sha256 {
            return Err(AdvertisedError::DigestMismatch);
        }
        let snapshot = project_snapshot(
            vec![SnapshotPackage {
                name: name.to_owned(),
                version: version.clone(),
                archive_sha256: archive.archive_sha256.to_owned(),
                source_id: source_host.to_owned(),
                mutability: IMMUTABLE_RELEASE.to_owned(),
            }],
            Vec::new(),
        )?;
        nodes.push(AdvertisedVersionNode {
            archive_sha256: archive.archive_sha256.to_owned(),
            snapshot_sha256: snapshot.snapshot_sha256,
            version: version.clone(),
        });
    }
    identity_from(name, source_host, listing_sha256, nodes)
}

fn validate_name(name: &str) -> Result<(), AdvertisedError> {
    if name.len() > MAX_ADVERTISED_NAME_BYTES {
        return Err(AdvertisedError::NameMismatch);
    }
    PackageName::parse(name)?;
    Ok(())
}

fn load_listing<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    request: &AdvertisedRequest<'_>,
) -> Result<ListingFetch, AdvertisedError> {
    let primary = format!("{PRIMARY_BASE}/{}", request.name);
    match read_listing(transport, seen, &primary, request.limits.metadata_bytes) {
        Ok(body) => {
            return Ok(ListingFetch {
                host: SelectedHost::Primary,
                body,
                source_host: PRIMARY_HOST.to_owned(),
            });
        }
        Err(
            error @ (AdvertisedError::Redirect
            | AdvertisedError::Oversized
            | AdvertisedError::Truncated
            | AdvertisedError::DuplicateVersion { .. }
            | AdvertisedError::TooManyVersions
            | AdvertisedError::InvalidVersion),
        ) => return Err(error),
        Err(_) => {}
    }
    let secondary = format!("{SECONDARY_BASE}/{}", request.name);
    let body = read_listing(transport, seen, &secondary, request.limits.metadata_bytes)?;
    Ok(ListingFetch {
        host: SelectedHost::Secondary,
        body,
        source_host: SECONDARY_HOST.to_owned(),
    })
}

fn read_listing<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    url: &str,
    limit: u64,
) -> Result<Vec<u8>, AdvertisedError> {
    let response = fetch_once(transport, seen, url)?;
    if response.status == 302 {
        return Err(AdvertisedError::Redirect);
    }
    if !(200..300).contains(&response.status) {
        return Err(AdvertisedError::Unverifiable);
    }
    bounded_body(&response, limit)?;
    Ok(response.body)
}

fn fetch_archive<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    request: &AdvertisedRequest<'_>,
    host: &SelectedHost,
    version: &str,
) -> Result<Vec<u8>, AdvertisedError> {
    let archive_url = match host {
        SelectedHost::Primary => format!("{PRIMARY_BASE}/{}/{version}", request.name),
        SelectedHost::Secondary => format!("{SECONDARY_BASE}/{}/{version}", request.name),
    };
    let response = fetch_once(transport, seen, &archive_url)?;
    let final_response = if response.status == 302 {
        follow_archive_redirect(transport, seen, host, request.name, version, &response)?
    } else if (200..300).contains(&response.status) {
        response
    } else {
        return Err(AdvertisedError::Unverifiable);
    };
    bounded_body(&final_response, request.limits.archive_bytes)?;
    if !final_response.body.starts_with(&GZIP_MAGIC) {
        return Err(AdvertisedError::Malformed);
    }
    Ok(final_response.body)
}

fn follow_archive_redirect<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    host: &SelectedHost,
    name: &str,
    version: &str,
    response: &RegistryHttpResponse,
) -> Result<RegistryHttpResponse, AdvertisedError> {
    if !matches!(host, SelectedHost::Secondary) {
        return Err(AdvertisedError::Redirect);
    }
    let expected = format!("{SECONDARY_TARBALL_BASE}/{name}-{version}.tgz");
    if response.location.as_deref() != Some(expected.as_str()) {
        return Err(AdvertisedError::Redirect);
    }
    let redirected = fetch_once(transport, seen, &expected)?;
    if !(200..300).contains(&redirected.status) {
        return Err(AdvertisedError::Redirect);
    }
    Ok(redirected)
}

fn fetch_once<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    url: &str,
) -> Result<RegistryHttpResponse, AdvertisedError> {
    require_authorized_registry_url(url).map_err(|_| AdvertisedError::UnauthorizedHost)?;
    if !seen.insert(url.to_owned()) {
        return Err(AdvertisedError::Retry);
    }
    transport.get(url).map_err(|error| match error {
        crate::AcquisitionError::Timeout => AdvertisedError::Timeout,
        crate::AcquisitionError::UnauthorizedHost => AdvertisedError::UnauthorizedHost,
        _ => AdvertisedError::Unverifiable,
    })
}

fn bounded_body(response: &RegistryHttpResponse, limit: u64) -> Result<(), AdvertisedError> {
    if response.content_length.is_some_and(|length| length > limit)
        || response.body.len() as u64 > limit
    {
        return Err(AdvertisedError::Oversized);
    }
    if response.truncated {
        return Err(AdvertisedError::Truncated);
    }
    Ok(())
}

fn parse_versions(bytes: &[u8], max_versions: usize) -> Result<Vec<String>, AdvertisedError> {
    let document: ListingDocument =
        serde_json::from_slice(bytes).map_err(|error| map_listing_error(&error.to_string()))?;
    if document.versions.0.len() > max_versions {
        return Err(AdvertisedError::TooManyVersions);
    }
    let mut parsed = Vec::with_capacity(document.versions.0.len());
    for version in document.versions.0.keys() {
        if version.len() > MAX_ADVERTISED_VERSION_BYTES {
            return Err(AdvertisedError::InvalidVersion);
        }
        let parsed_version =
            Version::parse(version).map_err(|_| AdvertisedError::InvalidVersion)?;
        parsed.push((parsed_version, version.clone()));
    }
    parsed.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(parsed.into_iter().map(|(_, version)| version).collect())
}

fn map_listing_error(message: &str) -> AdvertisedError {
    if let Some(version) = message.strip_prefix("duplicate version ") {
        return AdvertisedError::DuplicateVersion {
            version: version.to_owned(),
        };
    }
    AdvertisedError::Malformed
}

fn identity_from(
    name: &str,
    source_host: &str,
    listing_sha256: &str,
    versions: Vec<AdvertisedVersionNode>,
) -> Result<AdvertisedIdentity, AdvertisedError> {
    let body = IdentityBody {
        coverage: ADVERTISED_BY_OFFICIAL_SOURCE,
        listing_sha256,
        name,
        schema: ECOSYSTEM_ADVERTISED_VERSIONS_SCHEMA,
        source_host,
        versions: &versions,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(AdvertisedIdentity {
        coverage: ADVERTISED_BY_OFFICIAL_SOURCE.to_owned(),
        listing_sha256: listing_sha256.to_owned(),
        name: name.to_owned(),
        schema: ECOSYSTEM_ADVERTISED_VERSIONS_SCHEMA.to_owned(),
        source_host: source_host.to_owned(),
        versions,
        document_sha256: PackageCache::digest(&bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SUPPLIED_VERSIONS_ONLY;
    use std::collections::BTreeMap;

    const NAME: &str = "hl7.fhir.r4.core";

    enum Reply {
        Body(RegistryHttpResponse),
        Timeout,
    }

    struct Scripted {
        calls: Vec<String>,
        routes: BTreeMap<String, Reply>,
    }

    impl Scripted {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                routes: BTreeMap::new(),
            }
        }

        fn route(mut self, url: &str, reply: Reply) -> Self {
            self.routes.insert(url.to_owned(), reply);
            self
        }
    }

    impl RegistryTransport for Scripted {
        fn get(&mut self, url: &str) -> Result<RegistryHttpResponse, crate::AcquisitionError> {
            self.calls.push(url.to_owned());
            match self.routes.get(url) {
                Some(Reply::Body(body)) => Ok(body.clone()),
                Some(Reply::Timeout) => Err(crate::AcquisitionError::Timeout),
                None => Err(crate::AcquisitionError::Unverifiable),
            }
        }
    }

    fn http(
        status: u16,
        body: Vec<u8>,
        content_length: Option<u64>,
        location: Option<String>,
    ) -> RegistryHttpResponse {
        RegistryHttpResponse {
            status,
            location,
            content_type: "application/json".to_owned(),
            etag: String::new(),
            last_modified: String::new(),
            content_length,
            body,
            truncated: false,
        }
    }

    fn gzip(byte: u8) -> Vec<u8> {
        vec![0x1f, 0x8b, byte]
    }

    fn limits() -> AdvertisedLimits {
        AdvertisedLimits {
            metadata_bytes: 64 * 1024,
            archive_bytes: 32,
            max_versions: MAX_ADVERTISED_VERSIONS,
        }
    }

    fn request<'a>() -> AdvertisedRequest<'a> {
        AdvertisedRequest {
            name: NAME,
            expected_listing_sha256: None,
            expected_archives: &[],
            retrieved_at: "2020-01-01T00:00:00Z",
            limits: limits(),
        }
    }

    fn listing(body: &str) -> RegistryHttpResponse {
        http(200, body.as_bytes().to_vec(), None, None)
    }

    fn primary(body: &str) -> Scripted {
        let versions = ["1.9.0", "1.10.0"];
        let mut script = Scripted::new().route(
            &format!("{PRIMARY_BASE}/{NAME}"),
            Reply::Body(listing(body)),
        );
        for (index, version) in versions.iter().enumerate() {
            script = script.route(
                &format!("{PRIMARY_BASE}/{NAME}/{version}"),
                Reply::Body(http(200, gzip(index as u8), None, None)),
            );
        }
        script
    }

    #[test]
    fn official_bounds_are_explicit() {
        let limits = AdvertisedLimits::official();
        assert_eq!(limits.metadata_bytes, 4 * 1024 * 1024);
        assert_eq!(limits.archive_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.max_versions, 256);
    }

    #[test]
    fn authoritative_listing_is_ordered_and_distinct_from_supplied_versions() {
        let body = r#"{"versions":{"1.10.0":{},"1.9.0":{}}}"#;
        let mut transport = primary(body);
        let observed =
            acquire_advertised_versions(&mut transport, None, &request()).expect("acquire");
        assert_eq!(observed.identity.versions[0].version, "1.9.0");
        assert_eq!(observed.identity.versions[1].version, "1.10.0");
        assert_eq!(
            observed.identity.listing_sha256,
            PackageCache::digest(body.as_bytes())
        );
        assert_eq!(observed.identity.coverage, ADVERTISED_BY_OFFICIAL_SOURCE);
        assert_ne!(observed.identity.coverage, SUPPLIED_VERSIONS_ONLY);
        assert_ne!(
            observed.identity.schema,
            crate::ECOSYSTEM_PACKAGE_HISTORY_SCHEMA
        );
        let encoded = serde_json::to_string(&IdentityBody {
            coverage: &observed.identity.coverage,
            listing_sha256: &observed.identity.listing_sha256,
            name: &observed.identity.name,
            schema: &observed.identity.schema,
            source_host: &observed.identity.source_host,
            versions: &observed.identity.versions,
        })
        .expect("json");
        assert!(!encoded.contains("retrieved_at"));
        for label in ["PROVEN_COMPATIBLE", "PROVEN_BREAKING", "ALLOW", "BLOCK"] {
            assert!(!encoded.contains(label));
        }
        let replayed = replay_advertised_versions(
            NAME,
            PRIMARY_HOST,
            &observed.listing_bytes,
            &observed.identity.listing_sha256,
            &[
                StoredArchive {
                    name: NAME,
                    version: "1.10.0",
                    bytes: &observed.archives[1].bytes,
                    archive_sha256: &observed.identity.versions[1].archive_sha256,
                },
                StoredArchive {
                    name: NAME,
                    version: "1.9.0",
                    bytes: &observed.archives[0].bytes,
                    archive_sha256: &observed.identity.versions[0].archive_sha256,
                },
            ],
            MAX_ADVERTISED_VERSIONS,
        )
        .expect("replay");
        assert_eq!(replayed.document_sha256, observed.identity.document_sha256);
    }

    #[test]
    fn closed_listing_and_archive_failures() {
        let metadata = format!("{PRIMARY_BASE}/{NAME}");
        let mut duplicate = Scripted::new().route(
            &metadata,
            Reply::Body(listing(r#"{"versions":{"1.0.0":{},"1.0.0":{}}}"#)),
        );
        let error = acquire_advertised_versions(&mut duplicate, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::DuplicateVersion { .. }));

        let mut latest = Scripted::new().route(
            &metadata,
            Reply::Body(listing(r#"{"versions":{"latest":{}}}"#)),
        );
        let error = acquire_advertised_versions(&mut latest, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::InvalidVersion));

        let error = require_authorized_registry_url("https://evil.example/package").unwrap_err();
        assert!(matches!(error, crate::AcquisitionError::UnauthorizedHost));

        let mut oversized = Scripted::new().route(
            &metadata,
            Reply::Body(http(200, b"{}".to_vec(), Some(70_000), None)),
        );
        let error = acquire_advertised_versions(&mut oversized, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::Oversized));

        let mut versions = String::from(r#"{"versions":{"#);
        for index in 1..=257 {
            if index > 1 {
                versions.push(',');
            }
            versions.push_str(&format!("\"1.0.{index}\":{{}}"));
        }
        versions.push_str("}}");
        let mut too_many = Scripted::new().route(&metadata, Reply::Body(listing(&versions)));
        let error = acquire_advertised_versions(&mut too_many, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::TooManyVersions));
        assert_eq!(too_many.calls.len(), 1);

        let mut redirect = Scripted::new().route(
            &metadata,
            Reply::Body(http(
                302,
                Vec::new(),
                None,
                Some("https://evil.example/list".to_owned()),
            )),
        );
        let error = acquire_advertised_versions(&mut redirect, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::Redirect));
        assert!(redirect
            .calls
            .iter()
            .all(|url| !url.contains("evil.example")));

        let mut timed = Scripted::new()
            .route(&metadata, Reply::Timeout)
            .route(&format!("{SECONDARY_BASE}/{NAME}"), Reply::Timeout);
        let error = acquire_advertised_versions(&mut timed, None, &request()).unwrap_err();
        assert!(matches!(error, AdvertisedError::Timeout));
    }

    #[test]
    fn digest_name_version_and_tamper_fail() {
        let body = r#"{"versions":{"1.0.0":{}}}"#;
        let archive_url = format!("{PRIMARY_BASE}/{NAME}/1.0.0");
        let digest = PackageCache::digest(&gzip(1));
        let mut wrong = Scripted::new()
            .route(
                &format!("{PRIMARY_BASE}/{NAME}"),
                Reply::Body(listing(body)),
            )
            .route(&archive_url, Reply::Body(http(200, gzip(1), None, None)));
        let mut mismatched = request();
        let expected = [(
            "1.0.0",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )];
        mismatched.expected_archives = &expected;
        let error = acquire_advertised_versions(&mut wrong, None, &mismatched).unwrap_err();
        assert!(matches!(error, AdvertisedError::DigestMismatch));

        let mut transport = Scripted::new()
            .route(
                &format!("{PRIMARY_BASE}/{NAME}"),
                Reply::Body(listing(body)),
            )
            .route(&archive_url, Reply::Body(http(200, gzip(1), None, None)));
        let observed =
            acquire_advertised_versions(&mut transport, None, &request()).expect("acquire");
        let wrong_name = replay_advertised_versions(
            NAME,
            PRIMARY_HOST,
            &observed.listing_bytes,
            &observed.identity.listing_sha256,
            &[StoredArchive {
                name: "hl7.fhir.us.core",
                version: "1.0.0",
                bytes: &observed.archives[0].bytes,
                archive_sha256: &digest,
            }],
            MAX_ADVERTISED_VERSIONS,
        )
        .unwrap_err();
        assert!(matches!(wrong_name, AdvertisedError::NameMismatch));

        let wrong_version = replay_advertised_versions(
            NAME,
            PRIMARY_HOST,
            &observed.listing_bytes,
            &observed.identity.listing_sha256,
            &[StoredArchive {
                name: NAME,
                version: "9.9.9",
                bytes: &observed.archives[0].bytes,
                archive_sha256: &digest,
            }],
            MAX_ADVERTISED_VERSIONS,
        )
        .unwrap_err();
        assert!(matches!(wrong_version, AdvertisedError::VersionMismatch));

        let mut tampered_listing = observed.listing_bytes.clone();
        tampered_listing[0] ^= 0x01;
        assert!(replay_advertised_versions(
            NAME,
            PRIMARY_HOST,
            &tampered_listing,
            &observed.identity.listing_sha256,
            &[StoredArchive {
                name: NAME,
                version: "1.0.0",
                bytes: &observed.archives[0].bytes,
                archive_sha256: &digest,
            }],
            MAX_ADVERTISED_VERSIONS,
        )
        .is_err());

        let mut tampered_archive = observed.archives[0].bytes.clone();
        tampered_archive[2] ^= 0x01;
        assert!(replay_advertised_versions(
            NAME,
            PRIMARY_HOST,
            &observed.listing_bytes,
            &observed.identity.listing_sha256,
            &[StoredArchive {
                name: NAME,
                version: "1.0.0",
                bytes: &tampered_archive,
                archive_sha256: &digest,
            }],
            MAX_ADVERTISED_VERSIONS,
        )
        .is_err());

        let root = tempfile::tempdir().expect("tempdir");
        let cache = PackageCache::new(root.path());
        let mut stored = Scripted::new()
            .route(
                &format!("{PRIMARY_BASE}/{NAME}"),
                Reply::Body(listing(body)),
            )
            .route(&archive_url, Reply::Body(http(200, gzip(1), None, None)));
        let observed =
            acquire_advertised_versions(&mut stored, Some(&cache), &request()).expect("cache");
        assert_eq!(
            cache
                .read_verified(&observed.identity.listing_sha256)
                .expect("listing"),
            observed.listing_bytes
        );
        let path = root.path().join("sha256").join(format!(
            "{}.tgz",
            observed.identity.versions[0].archive_sha256
        ));
        let mut file = std::fs::read(&path).expect("archive file");
        file[0] ^= 0x01;
        std::fs::write(&path, file).expect("tamper file");
        assert!(cache
            .read_verified(&observed.identity.versions[0].archive_sha256)
            .is_err());
    }
}
