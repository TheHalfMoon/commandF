use std::collections::{BTreeMap, BTreeSet};

use semver::Version;
use serde::Deserialize;
use thiserror::Error;

use crate::source::MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES;
use crate::{
    project_snapshot, EcosystemSnapshot, PackageCache, PackageError, PackageName, SnapshotError,
    SnapshotPackage, IMMUTABLE_RELEASE,
};

pub const ECOSYSTEM_REGISTRY_RETRIEVAL_SCHEMA: &str = "commandf.ecosystem-registry-retrieval/v1";
pub const PRIMARY_HOST: &str = "packages.fhir.org";
pub const SECONDARY_HOST: &str = "packages2.fhir.org";
pub const PRIMARY_BASE: &str = "https://packages.fhir.org";
pub const SECONDARY_BASE: &str = "https://packages2.fhir.org/packages";
pub const SECONDARY_TARBALL_BASE: &str = "https://packages2.fhir.org/web";
pub const METADATA_LIMIT: u64 = 4 * 1024 * 1024;
pub const ARCHIVE_LIMIT: u64 = MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES;
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcquisitionLimits {
    pub metadata_bytes: u64,
    pub archive_bytes: u64,
}

impl AcquisitionLimits {
    pub fn official() -> Self {
        Self {
            metadata_bytes: METADATA_LIMIT,
            archive_bytes: ARCHIVE_LIMIT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegistryHttpResponse {
    pub status: u16,
    pub location: Option<String>,
    pub content_type: String,
    pub etag: String,
    pub last_modified: String,
    pub content_length: Option<u64>,
    pub body: Vec<u8>,
    pub truncated: bool,
}

pub trait RegistryTransport {
    fn get(&mut self, url: &str) -> Result<RegistryHttpResponse, AcquisitionError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcquisitionRequest<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub expected_sha256: Option<&'a str>,
    pub retrieved_at: &'a str,
    pub existing: &'a [SnapshotPackage],
    pub limits: AcquisitionLimits,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct RegistryRetrieval {
    pub archive_sha256: String,
    pub content_type: String,
    pub etag: String,
    pub http_status: u16,
    pub last_modified: String,
    pub name: String,
    pub request_url: String,
    pub retrieved_at: String,
    pub schema: String,
    pub source_host: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcquiredPackage {
    pub bytes: Vec<u8>,
    pub retrieval: RegistryRetrieval,
    pub snapshot: EcosystemSnapshot,
}

#[derive(Debug, Error)]
pub enum AcquisitionError {
    #[error("package version must be an exact semantic version")]
    InvalidVersion,

    #[error("registry host is not an authorized official source")]
    UnauthorizedHost,

    #[error("registry acquisition timed out")]
    Timeout,

    #[error("registry metadata or archive is malformed")]
    Malformed,

    #[error("registry redirect is outside the pinned official tarball")]
    Redirect,

    #[error("registry response exceeded the byte limit")]
    Oversized,

    #[error("registry response was truncated")]
    Truncated,

    #[error("registry archive digest does not match the expected digest")]
    DigestMismatch,

    #[error("the same registry URL was requested more than once")]
    Retry,

    #[error("registry acquisition could not be verified")]
    Unverifiable,

    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),
}

#[derive(Deserialize)]
struct RegistryMetadata {
    versions: BTreeMap<String, serde_json::Value>,
}

enum SelectedHost {
    Primary,
    Secondary,
}

pub fn require_authorized_registry_url(url: &str) -> Result<(), AcquisitionError> {
    let host = https_host(url)?;
    if host == PRIMARY_HOST || host == SECONDARY_HOST {
        return Ok(());
    }
    Err(AcquisitionError::UnauthorizedHost)
}

pub fn verify_acquired_bytes(bytes: &[u8], archive_sha256: &str) -> Result<(), AcquisitionError> {
    if PackageCache::digest(bytes) != archive_sha256 {
        return Err(AcquisitionError::DigestMismatch);
    }
    Ok(())
}

pub fn replay_acquired_package(
    bytes: &[u8],
    name: &str,
    version: &str,
    source_host: &str,
    archive_sha256: &str,
    existing: &[SnapshotPackage],
) -> Result<EcosystemSnapshot, AcquisitionError> {
    validate_identity(name, version)?;
    if source_host != PRIMARY_HOST && source_host != SECONDARY_HOST {
        return Err(AcquisitionError::UnauthorizedHost);
    }
    verify_acquired_bytes(bytes, archive_sha256)?;
    project_observation(name, version, archive_sha256, source_host, existing)
}

pub fn acquire_official_package<T: RegistryTransport>(
    transport: &mut T,
    cache: Option<&PackageCache>,
    request: &AcquisitionRequest<'_>,
) -> Result<AcquiredPackage, AcquisitionError> {
    validate_identity(request.name, request.version)?;
    let mut seen = BTreeSet::new();
    let host = select_host(transport, &mut seen, request)?;
    let archive = fetch_archive(transport, &mut seen, request, &host)?;
    let digest = PackageCache::digest(&archive.body);
    if let Some(expected) = request.expected_sha256 {
        if expected != digest {
            return Err(AcquisitionError::DigestMismatch);
        }
    }
    let source_host = https_host(&archive.request_url)?.to_owned();
    let snapshot = project_observation(
        request.name,
        request.version,
        &digest,
        &source_host,
        request.existing,
    )?;
    if let Some(cache) = cache {
        let stored = cache.put(&archive.body)?;
        if stored != digest {
            return Err(AcquisitionError::DigestMismatch);
        }
    }
    let retrieval = RegistryRetrieval {
        archive_sha256: digest,
        content_type: archive.content_type,
        etag: archive.etag,
        http_status: archive.status,
        last_modified: archive.last_modified,
        name: request.name.to_owned(),
        request_url: archive.request_url,
        retrieved_at: request.retrieved_at.to_owned(),
        schema: ECOSYSTEM_REGISTRY_RETRIEVAL_SCHEMA.to_owned(),
        source_host,
        version: request.version.to_owned(),
    };
    Ok(AcquiredPackage {
        bytes: archive.body,
        retrieval,
        snapshot,
    })
}

struct ArchiveBody {
    body: Vec<u8>,
    request_url: String,
    status: u16,
    content_type: String,
    etag: String,
    last_modified: String,
}

fn validate_identity(name: &str, version: &str) -> Result<(), AcquisitionError> {
    PackageName::parse(name)?;
    if version == "latest" || Version::parse(version).is_err() {
        return Err(AcquisitionError::InvalidVersion);
    }
    Ok(())
}

fn select_host<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    request: &AcquisitionRequest<'_>,
) -> Result<SelectedHost, AcquisitionError> {
    let primary = format!("{PRIMARY_BASE}/{}", request.name);
    if metadata_has_version(transport, seen, &primary, request).is_ok() {
        return Ok(SelectedHost::Primary);
    }
    let secondary = format!("{SECONDARY_BASE}/{}", request.name);
    metadata_has_version(transport, seen, &secondary, request)?;
    Ok(SelectedHost::Secondary)
}

fn metadata_has_version<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    url: &str,
    request: &AcquisitionRequest<'_>,
) -> Result<(), AcquisitionError> {
    let response = fetch_once(transport, seen, url)?;
    if !(200..300).contains(&response.status) {
        return Err(AcquisitionError::Unverifiable);
    }
    accept_body(
        &response.body,
        response.content_length,
        response.truncated,
        request.limits.metadata_bytes,
    )?;
    let metadata: RegistryMetadata =
        serde_json::from_slice(&response.body).map_err(|_| AcquisitionError::Malformed)?;
    if metadata.versions.contains_key(request.version) {
        return Ok(());
    }
    Err(AcquisitionError::Malformed)
}

fn fetch_archive<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    request: &AcquisitionRequest<'_>,
    host: &SelectedHost,
) -> Result<ArchiveBody, AcquisitionError> {
    let archive_url = match host {
        SelectedHost::Primary => format!("{PRIMARY_BASE}/{}/{}", request.name, request.version),
        SelectedHost::Secondary => {
            format!("{SECONDARY_BASE}/{}/{}", request.name, request.version)
        }
    };
    let response = fetch_once(transport, seen, &archive_url)?;
    let final_response = match response.status {
        302 => follow_pinned_redirect(transport, seen, host, request, response)?,
        status if (200..300).contains(&status) => response,
        _ => return Err(AcquisitionError::Unverifiable),
    };
    if !(200..300).contains(&final_response.status) {
        return Err(AcquisitionError::Unverifiable);
    }
    accept_body(
        &final_response.body,
        final_response.content_length,
        final_response.truncated,
        request.limits.archive_bytes,
    )?;
    if !final_response.body.starts_with(&GZIP_MAGIC) {
        return Err(AcquisitionError::Malformed);
    }
    Ok(ArchiveBody {
        body: final_response.body,
        request_url: final_response.request_url,
        status: final_response.status,
        content_type: final_response.content_type,
        etag: final_response.etag,
        last_modified: final_response.last_modified,
    })
}

fn follow_pinned_redirect<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    host: &SelectedHost,
    request: &AcquisitionRequest<'_>,
    response: HttpFetch,
) -> Result<HttpFetch, AcquisitionError> {
    if !matches!(host, SelectedHost::Secondary) {
        return Err(AcquisitionError::Redirect);
    }
    let expected = format!(
        "{SECONDARY_TARBALL_BASE}/{}-{}.tgz",
        request.name, request.version
    );
    if response.location.as_deref() != Some(expected.as_str()) {
        return Err(AcquisitionError::Redirect);
    }
    let redirected = fetch_once(transport, seen, &expected)?;
    if !(200..300).contains(&redirected.status) {
        return Err(AcquisitionError::Redirect);
    }
    Ok(redirected)
}

struct HttpFetch {
    status: u16,
    location: Option<String>,
    content_type: String,
    etag: String,
    last_modified: String,
    content_length: Option<u64>,
    body: Vec<u8>,
    truncated: bool,
    request_url: String,
}

fn fetch_once<T: RegistryTransport>(
    transport: &mut T,
    seen: &mut BTreeSet<String>,
    url: &str,
) -> Result<HttpFetch, AcquisitionError> {
    require_authorized_registry_url(url)?;
    if !seen.insert(url.to_owned()) {
        return Err(AcquisitionError::Retry);
    }
    let response = transport.get(url)?;
    Ok(HttpFetch {
        status: response.status,
        location: response.location,
        content_type: response.content_type,
        etag: response.etag,
        last_modified: response.last_modified,
        content_length: response.content_length,
        body: response.body,
        truncated: response.truncated,
        request_url: url.to_owned(),
    })
}

fn accept_body(
    body: &[u8],
    content_length: Option<u64>,
    truncated: bool,
    limit: u64,
) -> Result<(), AcquisitionError> {
    if content_length.is_some_and(|length| length > limit) || body.len() as u64 > limit {
        return Err(AcquisitionError::Oversized);
    }
    if truncated {
        return Err(AcquisitionError::Truncated);
    }
    Ok(())
}

fn https_host(url: &str) -> Result<&str, AcquisitionError> {
    let rest = url
        .strip_prefix("https://")
        .ok_or(AcquisitionError::UnauthorizedHost)?;
    let (host, _) = rest
        .split_once('/')
        .ok_or(AcquisitionError::UnauthorizedHost)?;
    if host.is_empty() || host.contains('@') || host.contains(':') {
        return Err(AcquisitionError::UnauthorizedHost);
    }
    Ok(host)
}

fn project_observation(
    name: &str,
    version: &str,
    archive_sha256: &str,
    source_host: &str,
    existing: &[SnapshotPackage],
) -> Result<EcosystemSnapshot, AcquisitionError> {
    let mut packages = existing.to_vec();
    packages.push(SnapshotPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        archive_sha256: archive_sha256.to_owned(),
        source_id: source_host.to_owned(),
        mutability: IMMUTABLE_RELEASE.to_owned(),
    });
    Ok(project_snapshot(packages, Vec::new())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IMMUTABLE_RELEASE;

    const NAME: &str = "hl7.fhir.r4.core";
    const VERSION: &str = "4.0.1";

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
        fn get(&mut self, url: &str) -> Result<RegistryHttpResponse, AcquisitionError> {
            self.calls.push(url.to_owned());
            match self.routes.get(url) {
                Some(Reply::Body(body)) => Ok(body.clone()),
                Some(Reply::Timeout) => Err(AcquisitionError::Timeout),
                None => Err(AcquisitionError::Unverifiable),
            }
        }
    }

    fn metadata() -> RegistryHttpResponse {
        http(
            200,
            None,
            br#"{"versions":{"4.0.1":{}}}"#.to_vec(),
            false,
            None,
        )
    }

    fn archive(body: Vec<u8>) -> RegistryHttpResponse {
        http(200, None, body, false, None)
    }

    fn http(
        status: u16,
        location: Option<String>,
        body: Vec<u8>,
        truncated: bool,
        content_length: Option<u64>,
    ) -> RegistryHttpResponse {
        RegistryHttpResponse {
            status,
            location,
            content_type: "application/gzip".to_owned(),
            etag: "\"v1\"".to_owned(),
            last_modified: "Wed, 01 Jan 2020 00:00:00 GMT".to_owned(),
            content_length,
            body,
            truncated,
        }
    }

    fn gzip_body() -> Vec<u8> {
        vec![0x1f, 0x8b, 0x08, 0x00]
    }

    fn limits() -> AcquisitionLimits {
        AcquisitionLimits {
            metadata_bytes: 1024,
            archive_bytes: 32,
        }
    }

    fn request<'a>(expected: Option<&'a str>) -> AcquisitionRequest<'a> {
        AcquisitionRequest {
            name: NAME,
            version: VERSION,
            expected_sha256: expected,
            retrieved_at: "2020-01-01T00:00:00Z",
            existing: &[],
            limits: limits(),
        }
    }

    fn primary_success() -> Scripted {
        let metadata_url = format!("{PRIMARY_BASE}/{NAME}");
        let archive_url = format!("{PRIMARY_BASE}/{NAME}/{VERSION}");
        Scripted::new()
            .route(&metadata_url, Reply::Body(metadata()))
            .route(&archive_url, Reply::Body(archive(gzip_body())))
    }

    #[test]
    fn official_limits_match_the_existing_registry_bounds() {
        let limits = AcquisitionLimits::official();
        assert_eq!(limits.metadata_bytes, 4 * 1024 * 1024);
        assert_eq!(limits.archive_bytes, 128 * 1024 * 1024);
        assert_eq!(REQUEST_TIMEOUT_SECS, 30);
    }

    #[test]
    fn exact_official_response_is_stable_and_host_preserving() {
        let mut transport = primary_success();
        let first =
            acquire_official_package(&mut transport, None, &request(None)).expect("acquire");
        let mut again = primary_success();
        let mut later = request(None);
        later.retrieved_at = "2024-05-01T00:00:00Z";
        let second = acquire_official_package(&mut again, None, &later).expect("acquire again");
        assert_eq!(
            PackageCache::digest(&first.bytes),
            first.retrieval.archive_sha256
        );
        assert_eq!(
            first.snapshot.snapshot_sha256,
            second.snapshot.snapshot_sha256
        );
        assert_eq!(first.retrieval.source_host, PRIMARY_HOST);
        assert_eq!(first.retrieval.retrieved_at, "2020-01-01T00:00:00Z");
        assert_ne!(first.retrieval.retrieved_at, second.retrieval.retrieved_at);
        assert_eq!(first.snapshot.packages[0].mutability, IMMUTABLE_RELEASE);
        let retrieval_json = serde_json::to_vec(&first.retrieval).expect("retrieval json");
        let encoded = String::from_utf8(retrieval_json).expect("utf8");
        for label in [
            "PROVEN_COMPATIBLE",
            "PROVEN_BREAKING",
            "ALLOW",
            "BLOCK",
            "WARN",
            "ABSTAIN",
        ] {
            assert!(!encoded.contains(label));
            assert!(!format!("{:?}", first.snapshot).contains(label));
        }
    }

    #[test]
    fn oversized_timeout_malformed_redirect_and_truncated_fail() {
        let metadata_url = format!("{PRIMARY_BASE}/{NAME}");
        let archive_url = format!("{PRIMARY_BASE}/{NAME}/{VERSION}");
        let mut oversized = primary_success().route(
            &archive_url,
            Reply::Body(http(200, None, gzip_body(), false, Some(33))),
        );
        let error = acquire_official_package(&mut oversized, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Oversized));

        let mut timed = Scripted::new()
            .route(&metadata_url, Reply::Timeout)
            .route(&format!("{SECONDARY_BASE}/{NAME}"), Reply::Timeout);
        let error = acquire_official_package(&mut timed, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Timeout));

        let mut malformed = Scripted::new()
            .route(
                &metadata_url,
                Reply::Body(http(200, None, b"not-json".to_vec(), false, None)),
            )
            .route(
                &format!("{SECONDARY_BASE}/{NAME}"),
                Reply::Body(http(200, None, b"{}".to_vec(), false, None)),
            );
        let error = acquire_official_package(&mut malformed, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Malformed));

        let mut not_gzip =
            primary_success().route(&archive_url, Reply::Body(archive(b"PK".to_vec())));
        let error = acquire_official_package(&mut not_gzip, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Malformed));

        let secondary_archive = format!("{SECONDARY_BASE}/{NAME}/{VERSION}");
        let mut redirect = Scripted::new()
            .route(
                &metadata_url,
                Reply::Body(http(500, None, Vec::new(), false, None)),
            )
            .route(&format!("{SECONDARY_BASE}/{NAME}"), Reply::Body(metadata()))
            .route(
                &secondary_archive,
                Reply::Body(http(
                    302,
                    Some("https://evil.example/package.tgz".to_owned()),
                    Vec::new(),
                    false,
                    None,
                )),
            );
        let error = acquire_official_package(&mut redirect, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Redirect));
        assert!(redirect
            .calls
            .iter()
            .all(|url| !url.contains("evil.example")));

        let mut truncated = primary_success().route(
            &archive_url,
            Reply::Body(http(200, None, gzip_body(), true, None)),
        );
        let error = acquire_official_package(&mut truncated, None, &request(None)).unwrap_err();
        assert!(matches!(error, AcquisitionError::Truncated));
    }

    #[test]
    fn unauthorized_host_fails_before_a_request() {
        let transport = Scripted::new();
        let error =
            require_authorized_registry_url("https://evil.example/hl7.fhir.r4.core").unwrap_err();
        assert!(matches!(error, AcquisitionError::UnauthorizedHost));
        assert!(transport.calls.is_empty());
    }

    #[test]
    fn digest_mismatch_duplicate_retry_and_tamper_fail() {
        let digest = PackageCache::digest(&gzip_body());
        let mut wrong = primary_success();
        let error = acquire_official_package(
            &mut wrong,
            None,
            &request(Some(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )),
        )
        .unwrap_err();
        assert!(matches!(error, AcquisitionError::DigestMismatch));

        let existing = vec![SnapshotPackage {
            name: NAME.to_owned(),
            version: VERSION.to_owned(),
            archive_sha256: digest.clone(),
            source_id: PRIMARY_HOST.to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }];
        let mut duplicate_request = request(None);
        duplicate_request.existing = &existing;
        let mut duplicate = primary_success();
        let error = acquire_official_package(&mut duplicate, None, &duplicate_request).unwrap_err();
        assert!(matches!(
            error,
            AcquisitionError::Snapshot(SnapshotError::DuplicatePackage { .. })
        ));

        let metadata_url = format!("{PRIMARY_BASE}/{NAME}");
        let secondary_archive = format!("{SECONDARY_BASE}/{NAME}/{VERSION}");
        let mut fallback = Scripted::new()
            .route(
                &metadata_url,
                Reply::Body(http(500, None, Vec::new(), false, None)),
            )
            .route(&format!("{SECONDARY_BASE}/{NAME}"), Reply::Body(metadata()))
            .route(&secondary_archive, Reply::Body(archive(gzip_body())));
        let acquired =
            acquire_official_package(&mut fallback, None, &request(None)).expect("secondary");
        assert_eq!(acquired.retrieval.source_host, SECONDARY_HOST);
        assert_eq!(
            fallback
                .calls
                .iter()
                .filter(|url| *url == &secondary_archive)
                .count(),
            1
        );
        assert!(verify_acquired_bytes(
            &acquired.bytes,
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        )
        .is_err());

        let cache_root = tempfile::tempdir().expect("tempdir");
        let cache = PackageCache::new(cache_root.path());
        let mut stored = primary_success();
        let acquired = acquire_official_package(&mut stored, Some(&cache), &request(Some(&digest)))
            .expect("cache");
        let reread = cache.read_verified(&digest).expect("verified read");
        assert_eq!(reread, acquired.bytes);
        let path = cache_root
            .path()
            .join("sha256")
            .join(format!("{digest}.tgz"));
        let mut tampered = std::fs::read(&path).expect("read stored");
        tampered[0] ^= 0x01;
        std::fs::write(&path, tampered).expect("tamper");
        assert!(cache.read_verified(&digest).is_err());
    }

    #[test]
    fn offline_replay_uses_verified_bytes_and_no_transport() {
        let mut transport = primary_success();
        let acquired =
            acquire_official_package(&mut transport, None, &request(None)).expect("acquire");
        let replayed = replay_acquired_package(
            &acquired.bytes,
            NAME,
            VERSION,
            &acquired.retrieval.source_host,
            &acquired.retrieval.archive_sha256,
            &[],
        )
        .expect("replay");
        assert_eq!(replayed.snapshot_sha256, acquired.snapshot.snapshot_sha256);
        let mut changed = acquired.bytes.clone();
        changed[2] ^= 0x01;
        assert!(replay_acquired_package(
            &changed,
            NAME,
            VERSION,
            PRIMARY_HOST,
            &acquired.retrieval.archive_sha256,
            &[],
        )
        .is_err());
    }

    #[test]
    fn latest_is_rejected_before_network() {
        let mut transport = primary_success();
        let mut floating = request(None);
        floating.version = "latest";
        let error = acquire_official_package(&mut transport, None, &floating).unwrap_err();
        assert!(matches!(error, AcquisitionError::InvalidVersion));
        assert!(transport.calls.is_empty());
    }
}
