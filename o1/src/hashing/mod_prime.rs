//! Implementation of the multiply-mod-prime hashing scheme from [Carter & Wegman, 1979].
//!
//! Primarily serves as a "ground truth" for the empirical testing suite.
//!
//! [Carter & Wegman, 1979]: https://doi.org/10.1016/0022-0000(79)90044-8

use crate::hashing::common::num_buckets_for_bits;

/// Hashes a 64-bit value into a 32-bit hash using the multiply-mod-prime scheme.
#[allow(dead_code)]
pub fn mod_prime(x: u64, num_bits: u32, seed: &[u128; 2]) -> u32 {
    const P_E: u32 = 89;
    const P: u128 = (1u128 << P_E) - 1;

    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        seed[0] > 0 && (seed[0]) < P,
        r#""seed[0]" must be in the range [1, {}-1]"#,
        P,
    );

    (seed[0].wrapping_mul(x as u128).wrapping_add(seed[1])
        % P
        % num_buckets_for_bits(num_bits) as u128) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::num_bits_for_buckets;
    use o1_test::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha20Rng;

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_mod_prime_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, u64>(
            &mut rng,
            &|rng, num_buckets| {
                let mut seed = [0_u128; 2];
                seed[0] = rng.random_range(1..2_u128.pow(89) - 1);
                seed[1] = rng.random_range(0..2_u128.pow(89) - 1);

                let num_bits = num_bits_for_buckets(num_buckets as u32);

                (
                    Box::new(move |value: &u64| mod_prime(*value, num_bits, &seed) as usize),
                    num_buckets_for_bits(num_bits) as usize,
                )
            },
            16,
            15,
            1000,
            0.01,
        );
    }
}
