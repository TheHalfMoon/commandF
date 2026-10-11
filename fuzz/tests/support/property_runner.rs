use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};

pub const CASE_COUNT: u32 = 256;
pub const MAX_SHRINK_ITERS: u32 = 4096;

pub fn runner(seed_hex: &str) -> TestRunner {
    let seed = decode_seed(seed_hex);
    let config = Config {
        cases: CASE_COUNT,
        max_shrink_iters: MAX_SHRINK_ITERS,
        failure_persistence: None,
        rng_algorithm: RngAlgorithm::ChaCha,
        ..Config::default()
    };
    TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &seed))
}

fn decode_seed(seed_hex: &str) -> [u8; 32] {
    assert_eq!(
        seed_hex.len(),
        64,
        "AF-02 property seed must be 256-bit hex"
    );
    let mut seed = [0_u8; 32];
    for (index, byte) in seed.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&seed_hex[offset..offset + 2], 16)
            .expect("AF-02 property seed must be lowercase hexadecimal");
    }
    seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_seed_runner_contract_is_closed() {
        let runner = runner("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
        assert_eq!(runner.config().cases, CASE_COUNT);
        assert_eq!(runner.config().max_shrink_iters, MAX_SHRINK_ITERS);
        assert!(runner.config().failure_persistence.is_none());
        assert_eq!(runner.config().rng_algorithm, RngAlgorithm::ChaCha);
    }
}
