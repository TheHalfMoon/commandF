use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const OPTIONAL_INPUT_LIMIT_BYTES: u64 = 64 * 1024 * 1024;

const BEFORE_HEX: &str = concat!(
    "1f8b08000000000002ffed944d4fc3300c86fb5350cea31f63b452cf70e60037c4216bbd35d0a655924e43d3fe3beed66d6c",
    "abc4013409789f1edc388df336b1ddc8ec4dce2968b6d67fb5b5f67e98908927938d654e6d1826f1e1bdf347517c137957a1",
    "77015aeba4e1edbdffc94a68599148052d65d594e4f78920466241c6aa5af35ce4877ec89e9c1ad239e94c9115e96abdf6c0",
    "2fa7bfeee0d1993673ada13b9a29ad1c5ffcf52e25bedb13beaaff241e9fd4ff6d128e51ff97a97f43b66e4d464fef4dd707",
    "0612812b5fe58716c1c3d6943c2e9c6bd220d8a5496de64369141c969d7794bef9dcef3fe1db702d37172133a7169de74de9",
    "6ef39d4cf6c8a97586e7453a93a5a591705be90f534b66217bc953693fffc35e6e51261ba9b3429941bdc7617232aa1fa422",
    "ab75b7b5d2ae93aa65638b9a65ac049554117bd3e7d5f6ac8e8334d21567ce4a71c890ad5c762722d6a3f3b57e7f1e43210e",
    "73a7917627c262b23aa78d7036eb177ed0b3010000000000000000000000000000000000803fc7075ec49c6300280000",
);

const AFTER_HEX: &str = concat!(
    "1f8b08000000000002ffed944d4fc3300c86fb5350cea31f636c52cf70e60037c4216bbd35d0a555924e4353ff3b6ed7ad6c",
    "abc4013409789f1edc388df336b15dcae44d2e292877d67fb585f67e9890994e26ad654e6d18cea6fd7be38fa2e94de45d85",
    "de05a8ac9386b7f7fe275ba1e58a442c682357654e7e97086224d664ac2a34cf457ee487ec49a9249d924e1459116febda03",
    "bf9ceeba834767aac45586ee68a1b4727cf1d7fb94f86e4ff8aafe67d3f149fddfcec231eaff32f56fc8169549e8e9bd6cfa",
    "c0402270e5abb46f113cac4ccee3ccb9320e827d9a146639944641bfecbca374cde7fef009df86abb8b9089938b56e3c6f4a",
    "379bef65b247cead333c2fe285cc2d8d84db497f985b326bd9499e4bfbf91f0e72b37cd64a5d64ca0cea3d0e939251dd2016",
    "49a19bad95768d95768d542d4b9b152c632b28a715b1377edeeeceea3848295d76e65c290e19b2959be644443d3a5feb77e73114a2",
    "9f6b23457da4fd89b098a448a915cea67ee1073d1b000000000000000000000000000000000000f8737c00934f6565002800",
    "00",
);

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
        .collect()
}

fn hex_digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("invalid test hex digit"),
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("commandf-{label}-{}-{nonce}", std::process::id()))
}

fn write_locked_state(root: &Path, archive: &[u8], version: &str) -> (PathBuf, PathBuf) {
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    fs::create_dir_all(root).expect("create state root");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(archive).expect("cache synthetic archive");
    let lockfile = Lockfile::new(
        vec![format!("example.package@{version}")],
        vec![LockedPackage {
            name: "example.package".to_owned(),
            version: version.to_owned(),
            sha256: digest,
            source: "synthetic-test".to_owned(),
            dependencies: BTreeMap::new(),
        }],
    );
    fs::write(&lock_path, lockfile.to_bytes().expect("serialize lock")).expect("write lock");
    (lock_path, cache_path)
}

fn changed_states(dir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let before = decode_hex(BEFORE_HEX);
    let after = decode_hex(AFTER_HEX);
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"), &before, "1.0.0");
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"), &after, "1.1.0");
    (before_lock, before_cache, after_lock, after_cache)
}

fn run_gate(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
    flag: &str,
    input: &Path,
) -> Output {
    commandf()
        .args([
            "gate",
            "example.package",
            "--before-lock",
            before_lock.to_str().expect("UTF-8 path"),
            "--before-cache",
            before_cache.to_str().expect("UTF-8 path"),
            "--after-lock",
            after_lock.to_str().expect("UTF-8 path"),
            "--after-cache",
            after_cache.to_str().expect("UTF-8 path"),
            flag,
            input.to_str().expect("UTF-8 path"),
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf gate must execute")
}

fn assert_oversized_optional_input_is_rejected(flag: &str, label: &str) {
    let dir = unique_temp_dir(label);
    let (before_lock, before_cache, after_lock, after_cache) = changed_states(&dir);
    let oversized_path = dir.join("oversized.json");
    let oversized = File::create(&oversized_path).expect("create sparse oversized input");
    oversized
        .set_len(OPTIONAL_INPUT_LIMIT_BYTES + 1)
        .expect("extend sparse oversized input");

    let output = run_gate(
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        flag,
        &oversized_path,
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
    assert!(
        stderr.contains("input exceeds 67108864 byte limit"),
        "unexpected diagnostic: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn oversized_baseline_exits_one_with_bounded_read_diagnostic() {
    assert_oversized_optional_input_is_rejected("--baseline", "gate-oversized-baseline");
}

#[test]
fn oversized_suppressions_exit_one_with_bounded_read_diagnostic() {
    assert_oversized_optional_input_is_rejected(
        "--suppressions",
        "gate-oversized-suppressions",
    );
}
