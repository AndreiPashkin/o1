//! Polynomial hashing implementation for arbitrary-size strings based on
//! [Dietzfelbinger et al. (1992)] and [Thorup (2015)].
//!
//! The core idea of polynomial hashing is to treat the input vector as coefficients for a
//! polynomial and then compute it efficiently using Horner's rule and a modulo operation optimized
//! for Mersenne primes.
//!
//! [Dietzfelbinger et al. (1992)]: https://doi.org/10.1007/3-540-55719-9_77
//! [Thorup (2015)]: https://doi.org/10.48550/arXiv.1504.06804

use crate::hashing::common::{extract_bits_128, extract_bits_64};
use crate::hashing::multiply_shift::pair_multiply_shift_vector_u64;
use crate::hashing::multiply_shift::pair_multiply_shift_vector_u64_const;
use crate::utils::bit_hacks::mod_mersenne_prime;
use std::ptr::copy_nonoverlapping;

/// The type for the underlying seed value for [`PolynomialSeed`].
pub type PolynomialSeedValue = [u64; 1 + 1 + 64 + 1 + 64 + 1];

/// Seed value for the [`polynomial`] hashing function.
#[derive(Debug, Clone, Copy)]
pub struct PolynomialSeed(PolynomialSeedValue);

impl From<PolynomialSeedValue> for PolynomialSeed {
    fn from(seed: PolynomialSeedValue) -> Self {
        PolynomialSeed(seed)
    }
}

impl PolynomialSeed {
    pub const fn new(
        a: u64,
        b: u64,
        h1_a: [u64; 64],
        h1_a_d: u64,
        h2_a: [u64; 64],
        h2_a_d: u64,
    ) -> Self {
        let mut seed = [0_u64; 132];

        seed[0] = a;
        seed[1] = b;
        unsafe {
            copy_nonoverlapping(h1_a.as_ptr(), seed.as_mut_ptr().add(2), 64);
        }
        seed[2 + 64] = h1_a_d;
        unsafe {
            copy_nonoverlapping(h2_a.as_ptr(), seed.as_mut_ptr().add(2 + 64 + 1), 64);
        }
        seed[2 + 64 + 1 + 64] = h2_a_d;

        PolynomialSeed(seed)
    }

    pub const fn from_slice(slice: &[u64]) -> Self {
        let mut seed = [0_u64; 132];
        debug_assert!(slice.len() == 132, "Slice must have length of 132");
        unsafe {
            copy_nonoverlapping(slice.as_ptr(), seed.as_mut_ptr(), 132);
        }
        PolynomialSeed(seed)
    }
}

impl From<&[u64]> for PolynomialSeed {
    fn from(seed: &[u64]) -> Self {
        PolynomialSeed::from_slice(seed)
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
///           seed value should be greater than `0`.
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
fn hash_chunk(chunk: &[u64], h1_seed: &[u64], h2_seed: &[u64]) -> u64 {
    let chunk_hash_high = pair_multiply_shift_vector_u64(chunk, 32, h1_seed[0], &h1_seed[1..]);
    let chunk_hash_low = pair_multiply_shift_vector_u64(chunk, 32, h2_seed[0], &h2_seed[1..]);
    ((chunk_hash_high as u64) << 32) | (chunk_hash_low as u64)
}

/// Const version of the polynomial hash function.
///
/// Compile-time equivalent of [`polynomial`].
///
/// # Parameters
///
/// - `value`: The input bytes.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed values. It should have length of `1 + 1 + 64 + 1 + 64 + 1`,
///           so `132` in total. All the seed values should be less than 2 ** 89 - 1. And the first
///           seed value should be greater than `0`.
///
/// # Guarantees
///
/// - Strongly universal.
#[inline]
pub const fn polynomial_const(value: &[u8], num_bits: u32, seed: &PolynomialSeed) -> u32 {
    const P_E: u32 = 89;
    const P: u128 = (1_u128 << P_E) - 1;

    let seed = seed.0;

    let a = seed[0];
    let b = seed[1];

    let h1_seed = unsafe { std::slice::from_raw_parts(seed.as_ptr().add(2), 65) };
    let h2_seed = unsafe { std::slice::from_raw_parts(seed.as_ptr().add(2 + 65), 65) };

    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        a > 0 && (a as u128) < P,
        r#""seed[0]" must be in the range [1, 618970019642690137449562111-1]"#,
    );
    let mut i = 1;
    while i < seed.len() {
        debug_assert!(
            (seed[i] as u128) < P,
            r#""seed[...]" must be in the range [0, 618970019642690137449562111-1]"#,
        );
        i += 1;
    }
    debug_assert!(
        h1_seed.len() == 64 + 1,
        r#""seed[2..2 + (64 + 1)]" must have length 65"#,
    );
    debug_assert!(
        h2_seed.len() == 64 + 1,
        r#""seed[2 + (64 + 1)..(2 + (64 + 1)) + 64 + 1]" must have length 65"#,
    );

    if value.is_empty() {
        return extract_bits_64::<64>(b, num_bits);
    }

    let num_chunks = value.len() >> 8;
    let remainder_len = value.len() & 0xFF;

    let mut buffer = [0_u64; 32];

    let mut hash_value = b as u128;

    if num_chunks > 0 {
        let mut j = 0;
        while j < 256 && j < value.len() {
            let byte_idx = j;
            let buffer_idx = j >> 3;
            let shift = (j & 0x7) << 3;

            buffer[buffer_idx] |= (value[byte_idx] as u64) << shift;
            j += 1;
        }

        hash_value += hash_chunk_const(&buffer, h1_seed, h2_seed) as u128;

        let mut i = 1;
        while i < num_chunks {
            let mut k = 0;
            while k < buffer.len() {
                buffer[k] = 0;
                k += 1;
            }

            let mut j = 0;
            while j < 256 && (i * 256 + j) < value.len() {
                let byte_idx = i * 256 + j;
                let buffer_idx = j >> 3;
                let shift = (j & 0x7) << 3;

                buffer[buffer_idx] |= (value[byte_idx] as u64) << shift;
                j += 1;
            }

            let chunk_hash = hash_chunk_const(&buffer, h1_seed, h2_seed);

            hash_value = mod_mersenne_prime::<P_E, P>(
                hash_value
                    .wrapping_mul(a as u128)
                    .wrapping_add(chunk_hash as u128),
            );

            i += 1;
        }
    }

    if remainder_len > 0 {
        let mut k = 0;
        while k < buffer.len() {
            buffer[k] = 0;
            k += 1;
        }

        let mut j = 0;
        let start_idx = value.len() - remainder_len;
        while j < remainder_len {
            let byte_idx = start_idx + j;
            let buffer_idx = j >> 3;
            let shift = (j & 0x7) << 3;

            buffer[buffer_idx] |= (value[byte_idx] as u64) << shift;
            j += 1;
        }

        let chunk_hash = hash_chunk_const(&buffer, h1_seed, h2_seed);
        hash_value = mod_mersenne_prime::<P_E, P>(
            hash_value
                .wrapping_mul(a as u128)
                .wrapping_add(chunk_hash as u128),
        );
    }

    hash_value = mod_mersenne_prime::<P_E, P>(hash_value.wrapping_mul(a as u128));

    extract_bits_128::<{ P_E }>(hash_value, num_bits)
}

/// Compile-time counterpart of [`hash_chunk`].
const fn hash_chunk_const(chunk: &[u64], h1_seed: &[u64], h2_seed: &[u64]) -> u64 {
    // In const contexts, we can't use slice patterns like [1..], so we need to use raw pointers
    let h1_seed_value = h1_seed[0];
    let h1_seed_rest =
        unsafe { std::slice::from_raw_parts(h1_seed.as_ptr().add(1), h1_seed.len() - 1) };

    let h2_seed_value = h2_seed[0];
    let h2_seed_rest =
        unsafe { std::slice::from_raw_parts(h2_seed.as_ptr().add(1), h2_seed.len() - 1) };

    let chunk_hash_high =
        pair_multiply_shift_vector_u64_const(chunk, 32, h1_seed_value, h1_seed_rest);
    let chunk_hash_low =
        pair_multiply_shift_vector_u64_const(chunk, 32, h2_seed_value, h2_seed_rest);
    ((chunk_hash_high as u64) << 32) | (chunk_hash_low as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
    use o1_testing::*;
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
                (
                    Box::new(move |value: &String| {
                        polynomial(value.as_bytes(), num_bits, &seed.into()) as usize
                    }),
                    num_buckets_for_bits(num_bits) as usize,
                )
            },
            16,
            15,
            1000,
            0.01,
        );
    }

    #[test]
    fn test_polynomial_const_equivalence() {
        let mut rng = ChaCha20Rng::from_os_rng();

        for str_len in [0, 1, 4, 8, 16, 255, 256, 257, 512, 1024] {
            let non_const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);

                let mut seed = [0u64; 1 + 1 + 64 + 1 + 64 + 1];
                seed.fill_with(|| rng.random());
                seed[0] = 1 + (seed[0] % (u64::MAX - 1));

                let seed = PolynomialSeed::from(seed);

                (
                    Box::new(move |value: &String| {
                        polynomial(value.as_bytes(), num_bits, &seed) as usize
                    }) as Box<dyn Fn(&String) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            let const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);

                // Generate exactly the same seed as in non_const_family
                let mut seed = [0u64; 1 + 1 + 64 + 1 + 64 + 1];
                seed.fill_with(|| rng.random());
                seed[0] = 1 + (seed[0] % (u64::MAX - 1));

                let seed = PolynomialSeed::from(seed);

                (
                    Box::new(move |value: &String| {
                        polynomial_const(value.as_bytes(), num_bits, &seed) as usize
                    }) as Box<dyn Fn(&String) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            // Generate random strings of the specified length
            equivalence(
                &mut rng,
                &non_const_family,
                &const_family,
                &|rng: &mut ChaCha20Rng| {
                    let bytes: Vec<u8> = (0..str_len).map(|_| rng.random::<u8>()).collect();
                    String::from_utf8_lossy(&bytes).to_string()
                },
                1000,
                99,
            );
        }
    }
}
