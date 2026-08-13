use commandf_pkg::{PackageCache, PackageError, PackageName, PackageRequest, PackageSource, Resolver};
use semver::Version;
use tempfile::tempdir;

const MISSING_MANIFEST: &[u8] = &[31,139,8,0,84,220,125,106,2,255,237,205,65,10,130,80,20,5,208,191,20,87,144,154,226,126,62,34,69,129,138,218,40,220,123,143,70,209,56,130,232,156,201,125,92,46,188,57,247,215,124,26,202,105,59,15,203,225,178,78,99,250,184,42,116,109,251,204,240,158,225,248,114,71,95,215,77,204,139,42,125,193,109,221,242,18,239,211,127,186,239,9,0,0,0,0,0,0,0,0,128,31,244,0,173,65,150,181,0,40,0,0];
const IDENTITY_MISMATCH: &[u8] = &[31,139,8,0,84,220,125,106,2,255,237,205,77,10,194,48,16,64,225,28,37,204,90,98,82,107,23,222,38,148,224,31,77,74,82,221,20,239,110,170,11,193,181,136,224,251,54,111,152,89,204,232,251,179,223,135,245,248,172,57,149,20,213,135,217,170,107,219,71,171,247,90,187,109,94,243,178,119,174,107,54,74,91,245,5,151,50,249,92,223,171,255,52,75,244,67,144,157,22,223,15,193,164,233,16,178,172,180,92,67,46,199,20,151,131,51,214,88,185,41,0,0,0,0,0,0,0,0,0,0,0,0,0,192,15,185,3,159,155,157,68,0,40,0,0];

struct FixedSource(&'static [u8]);

impl PackageSource for FixedSource {
    fn source_id(&self) -> String { "fixture".to_owned() }
    fn available_versions(&self, _name: &PackageName) -> Result<Vec<Version>, PackageError> {
        Ok(vec![Version::new(1, 0, 0)])
    }
    fn archive(&self, _name: &PackageName, _version: &Version) -> Result<Vec<u8>, PackageError> {
        Ok(self.0.to_vec())
    }
}

fn resolve(bytes: &'static [u8]) -> PackageError {
    let dir = tempdir().unwrap();
    Resolver::new(&FixedSource(bytes), &PackageCache::new(dir.path()))
        .resolve(vec![PackageRequest::parse("acme.root@1.0.0").unwrap()])
        .unwrap_err()
}

#[test]
fn missing_manifest_fails_closed() {
    assert!(matches!(resolve(MISSING_MANIFEST), PackageError::MissingManifest));
}

#[test]
fn identity_mismatch_fails_closed() {
    assert!(matches!(resolve(IDENTITY_MISMATCH), PackageError::IdentityMismatch { .. }));
}
