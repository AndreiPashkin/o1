//! Implementation of the multiply-shift hashing scheme (multiply-add-shift to be precise)
//! from [Dietzfelbinger (1996)], [Dietzfelbinger et al. (1997)] and [Thorup (2015)].
//!
//! Multiply-shift hashing scheme is an evolution of multiply-mod-prime that avoids
//! using expensive arithmetic operations like modulo and division and instead exploits integer
//! overflow and also bitshift operation.
//!
//! [Dietzfelbinger (1996)]: https://doi.org/10.1007/3-540-60922-9_46
//! [Dietzfelbinger et al. (1997)]: https://doi.org/10.1006/jagm.1997.0873
//! [Thorup (2015)]: https://doi.org/10.48550/arXiv.1504.06804

use crate::hashing::common::extract_bits_64;

// TODO: Consider implementing the weakly-universal version of multiply-shift that returns u64.
// TODO: Generally in the future 64-bit versions will probably be needed too.

/// Hashes a 32-bit unsigned integer using the multiply-shift hashing scheme.
///
/// # Parameters
///
/// - `value`: The input value.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed. The first element must be greater than 0.
///
/// # Guarantees
///
/// - Strong universality.
#[inline]
pub const fn multiply_shift(value: u32, num_bits: u32, seed: &[u64; 2]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(seed[0] > 0, r#""seed[0]" must be > 0"#);

    let hash = seed[0].wrapping_mul(value as u64).wrapping_add(seed[1]);
    extract_bits_64::<{ u64::BITS }>(hash, num_bits)
}

/// Hashes a 64-bit unsigned integer using the pair-multiply-shift hashing scheme.
///
/// # Parameters
///
/// - `value`: The input value.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed.
///
/// # Guarantees
///
/// - Strong universality.
#[inline]
pub const fn pair_multiply_shift(value: u64, num_bits: u32, seed: &[u64; 3]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    let hash_value = seed[0]
        .wrapping_add(value)
        .wrapping_mul(seed[1].wrapping_add(value >> 32))
        .wrapping_add(seed[2]);

    extract_bits_64::<{ u64::BITS }>(hash_value, num_bits)
}

/// Hashes a vector of 64-bit unsigned integers to a 32-bit hash value.
///
/// # Parameters
///
/// - `value`: The input vector with length up to `d`.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed. The length of the seed vector must be `d * 2 + 1`.
///
/// # Guarantees
///
/// - Strong universality.
#[inline]
pub fn pair_multiply_shift_vector_u64(value: &[u64], num_bits: u32, seed: &[u64]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        (value.len() * 2 + 1) <= seed.len(),
        r#""seed" must be twice as long as the input "value" + 1"#,
    );

    let mut sum = seed[0]; // Initializing the sum with the first seed value.

    let seed = &seed[1..];

    for (i, &v) in value.iter().enumerate() {
        let s = &seed[i * 2..i * 2 + 2];

        // Treating 64-bit values as a pairs of 32-bit values.
        let low = v;
        let high = v >> 32;

        sum = sum.wrapping_add(s[0].wrapping_add(high).wrapping_mul(s[1].wrapping_add(low)));
    }

    extract_bits_64::<{ u64::BITS }>(sum, num_bits)
}

/// Hashes a string (a vector of bytes) to a 32-bit hash value.
///
/// # Parameters
///
/// - `value`: The input vector with length up to `d`.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed of length `d.div_ceil(4) + 1`.
///
/// # Guarantees
///
/// - Strong universality.
#[inline]
pub fn pair_multiply_shift_vector_u8(value: &[u8], num_bits: u32, seed: &[u64]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        (value.len().div_ceil(4) + 1) <= seed.len(),
        r#""seed" must have 1 element per 4 elements in the input "value" + 1"#,
    );

    match value.len() {
        0 => extract_bits_64::<{ u64::BITS }>(seed[0], num_bits),
        1..=3 => {
            let mut padded = [0; 4];
            padded[..value.len()].copy_from_slice(value);
            let value = u32::from_le_bytes(padded);
            let seed = unsafe { seed.first_chunk().unwrap_unchecked() };
            multiply_shift(value, num_bits, seed)
        }
        4 => {
            let value = unsafe { value.first_chunk::<4>().unwrap_unchecked() };
            let value = u32::from_le_bytes(*value);
            let seed = unsafe { seed.first_chunk().unwrap_unchecked() };
            multiply_shift(value, num_bits, seed)
        }
        5..=7 => {
            let mut padded = [0; 8];
            padded[..value.len()].copy_from_slice(value);

            let value = u64::from_le_bytes(padded);
            let seed = unsafe { seed.first_chunk().unwrap_unchecked() };
            pair_multiply_shift(value, num_bits, seed)
        }
        8 => {
            let value = unsafe { value.first_chunk::<8>().unwrap_unchecked() };
            let value = u64::from_le_bytes(*value);
            let seed = unsafe { seed.first_chunk().unwrap_unchecked() };

            pair_multiply_shift(value, num_bits, seed)
        }
        _ => {
            let c = value.len();
            let d = (c + 7) >> 3;

            // TODO: This could be optimized by using a pre-allocated buffer.
            let mut x = vec![0_u64; d];
            let x_bytes =
                unsafe { std::slice::from_raw_parts_mut(x.as_mut_ptr() as *mut u8, d * 8) };
            x_bytes[..c].copy_from_slice(value);

            pair_multiply_shift_vector_u64(x.as_slice(), num_bits, seed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
    use crate::testing::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha20Rng;

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_multiply_shift_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, u32>(
            &mut rng,
            &|rng, num_buckets| {
                let mut seed = [0_u64; 2];
                seed[0] = rng.random_range(1..=u64::MAX);
                seed[1] = rng.random_range(0..=u64::MAX);

                let num_bits = num_bits_for_buckets(num_buckets as u32);
                (
                    Box::new(move |value: &u32| multiply_shift(*value, num_bits, &seed) as usize),
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
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_pair_multiply_shift_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, u64>(
            &mut rng,
            &|rng, num_buckets| {
                let seed: [u64; 3] = rng.random();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                (
                    Box::new(move |value: &u64| {
                        pair_multiply_shift(*value, num_bits, &seed) as usize
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
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_pair_multiply_shift_vector_u64_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, [u64; 32]>(
            &mut rng,
            &|rng, num_buckets| {
                let seed: [u64; 32 * 2 + 1] = rng.random();

                let num_bits = num_bits_for_buckets(num_buckets as u32);
                (
                    Box::new(move |value: &[u64; 32]| {
                        pair_multiply_shift_vector_u64(value, num_bits, &seed) as usize
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
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_multiply_shift_vector_u8_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, [u8; 32]>(
            &mut rng,
            &|rng, num_buckets| {
                let seed: [u64; 32_usize.div_ceil(4) + 1] = rng.random();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                (
                    Box::new(move |value: &[u8; 32]| {
                        pair_multiply_shift_vector_u8(value, num_bits, &seed) as usize
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
}
