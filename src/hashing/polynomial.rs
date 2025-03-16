//! Polynomial hashing implementation for arbitrary-size strings based on
//! [Dietzfelbinger et al. (1992)] and [Thorup (2015)].
//!
//! The core idea of polynomial hashing is to treat the input vector as coefficients for a
//! polynomial and then compute it efficiently using Horner's rule and a modulo operation optimized
//! for Mersenne primes.
//!
//! [Dietzfelbinger et al. (1992)]: https://doi.org/10.1007/3-540-55719-9_77
//! [Thorup (2015)]: https://doi.org/10.48550/arXiv.1504.06804

use crate::hashing::common::{extract_bits_128, extract_bits_64, mod_mersenne_prime};
use crate::hashing::multiply_shift::{pair_multiply_shift_vector_u64, PairMultiplyShiftSeed};

pub type PolynomialSeedValue = [u64; 1 + 1 + 64 + 1 + 64 + 1];
#[derive(Debug, Clone, Copy)]
pub struct PolynomialSeed(PolynomialSeedValue);

impl From<PolynomialSeedValue> for PolynomialSeed {
    fn from(seed: PolynomialSeedValue) -> Self {
        PolynomialSeed(seed)
    }
}

impl PolynomialSeed {
    pub fn new(a: u64, b: u64, h1_a: [u64; 64], h1_a_d: u64, h2_a: [u64; 64], h2_a_d: u64) -> Self {
        let mut seed = [0_u64; 132];

        seed[0] = a;
        seed[1] = b;
        seed[2..2 + 64].copy_from_slice(&h1_a);
        seed[2 + 64] = h1_a_d;
        seed[2 + 64 + 1..2 + 64 + 1 + 64].copy_from_slice(&h2_a);
        seed[2 + 64 + 1 + 64] = h2_a_d;

        PolynomialSeed(seed)
    }
}

impl From<&[u64]> for PolynomialSeed {
    fn from(seed: &[u64]) -> Self {
        let seed = seed.try_into().expect("Seed must have length of 132");
        PolynomialSeed(seed)
    }
}

impl Default for PolynomialSeed {
    fn default() -> Self {
        let mut value = [0_u64; 1 + 1 + 64 + 1 + 64 + 1];
        value[0] = 1;
        PolynomialSeed(value)
    }
}

/// Hashes a 32-bit unsigned integer using the multiply-shift hashing scheme.
///
/// # Parameters
///
/// - `value`: The input bytes.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `p`: Large Mersenne prime. `2 ** 89 − 1` could be a practical value.
/// - `p_e`: Exponent of the Mersenne prime.
/// - `seed`: Random seed values. It should have length of `1 + 1 + 64 + 1 + 64 + 1`,
///           so `132` in total. All the seed values should be less than 2 ** 89 - 1. And the first
///           seed value should be greater or equal to `0`.
///
/// # Guarantees
///
/// - Strongly universal.
///
/// # Notes
///
/// - The implementation splits the input into 256-bit chunks and then applies polynomial hashing
///   to hashes of the chunks.
#[inline]
pub fn polynomial(value: &[u8], num_bits: u32, seed: &PolynomialSeed) -> u32 {
    const P_E: u32 = 89;
    const P: u128 = (1_u128 << P_E) - 1;

    let seed = seed.0;

    let a = seed[0];
    let b = seed[1];
    let h1_seed = &seed[2..2 + (64 + 1)];
    let h2_seed = &seed[2 + (64 + 1)..(2 + (64 + 1)) + 64 + 1];

    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        a > 0 && (a as u128) < P,
        r#""seed[0]" must be in the range [1, {}-1]"#,
        P,
    );
    debug_assert!(
        seed[1..].iter().all(|&x| (x as u128) < P),
        r#""seed[1..]" must be in the range [0, {}-1]"#,
        P,
    );

    debug_assert_eq!(
        h1_seed.len(),
        64 + 1,
        r#""seed[2..2 + (64 + 1)]" have length 65, got {}"#,
        h1_seed.len(),
    );
    debug_assert_eq!(
        h2_seed.len(),
        64 + 1,
        r#""seed[2 + (64 + 1)..(2 + (64 + 1)) + 64 + 1]" have length 65, got {}"#,
        h2_seed.len(),
    );

    if value.is_empty() {
        return extract_bits_64::<64>(b, num_bits);
    }

    let num_chunks = value.len() >> 8;
    let remainder_len = value.len() & 0xFF;

    let mut buffer = [0_u64; 32];
    let buffer_bytes =
        unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * 8) };

    let mut hash_value = b as u128;

    if num_chunks > 0 {
        #[allow(clippy::identity_op)]
        let chunk = &value[0 << 8..(0 + 1) << 8];
        buffer_bytes[..].copy_from_slice(chunk);
        hash_value += hash_chunk(&buffer, h1_seed, h2_seed) as u128;

        for i in 1..num_chunks {
            let chunk = &value[i << 8..(i + 1) << 8];
            buffer_bytes[..].copy_from_slice(chunk);
            let chunk_hash = hash_chunk(&buffer, h1_seed, h2_seed);

            hash_value = mod_mersenne_prime::<P_E, P>(
                hash_value
                    .wrapping_mul(a as u128)
                    .wrapping_add(chunk_hash as u128),
            );
        }
    }

    if remainder_len > 0 {
        let remainder_chunk = &value[value.len() - remainder_len..];
        buffer_bytes[..remainder_len].copy_from_slice(remainder_chunk);
        buffer_bytes[remainder_len..].fill(0);
        let chunk_hash = hash_chunk(&buffer, h1_seed, h2_seed);
        hash_value = mod_mersenne_prime::<P_E, P>(
            hash_value
                .wrapping_mul(a as u128)
                .wrapping_add(chunk_hash as u128),
        );
    }

    hash_value = mod_mersenne_prime::<P_E, P>(hash_value.wrapping_mul(a as u128));

    extract_bits_128::<{ P_E }>(hash_value, num_bits)
}

/// Hashes a 256-long chunk into a 64-bit hash using concatenation of two 32-bit hashes.
fn hash_chunk(
    chunk: &[u64],
    h1_seed: &PairMultiplyShiftSeed,
    h2_seed: &PairMultiplyShiftSeed,
) -> u64 {
    let chunk_hash_high = pair_multiply_shift_vector_u64(chunk, 32, h1_seed);
    let chunk_hash_low = pair_multiply_shift_vector_u64(chunk, 32, h2_seed);
    ((chunk_hash_high as u64) << 32) | (chunk_hash_low as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::num_bits_for_buckets;
    use crate::testing::*;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_polynomial_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, String>(
            &mut rng,
            &|rng, num_buckets| {
                let seed: [u64; 1 + 1 + 64 + 1 + 64 + 1] = rng.random();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &String| {
                    polynomial(value.as_bytes(), num_bits, &seed.into()) as usize
                })
            },
            16,
            &|num_buckets| num_buckets.next_power_of_two(),
            15,
            1000,
            0.01,
        );
    }
}
