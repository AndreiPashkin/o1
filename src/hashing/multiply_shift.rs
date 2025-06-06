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
use std::ptr::copy_nonoverlapping;

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

/// Hashes a vector of 64-bit unsigned integers to a 32-bit hash value.
///
/// Compile-time equivalent of [`pair_multiply_shift_vector_u64`].
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
pub const fn pair_multiply_shift_vector_u64_const(
    value: &[u64],
    num_bits: u32,
    seed: &[u64],
) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        (value.len() * 2 + 1) <= seed.len(),
        r#""seed" must be twice as long as the input "value" + 1"#
    );

    let mut sum = seed[0]; // Initializing the sum with the first seed value.

    let mut i = 0;
    while i < value.len() {
        let v = value[i];

        // Seed offset calculation - need to access seed elements directly
        let s_idx = 1 + i * 2;

        // Treating 64-bit values as a pair of 32-bit values
        let low = v;
        let high = v >> 32;

        sum = sum.wrapping_add(
            seed[s_idx]
                .wrapping_add(high)
                .wrapping_mul(seed[s_idx + 1].wrapping_add(low)),
        );

        i += 1;
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

/// Hashes a string (a vector of bytes) to a 32-bit hash value.
///
/// Compile-time equivalent of [`pair_multiply_shift_vector_u8`].
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
pub const fn pair_multiply_shift_vector_u8_const(value: &[u8], num_bits: u32, seed: &[u64]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        (value.len().div_ceil(4) + 1) <= seed.len(),
        r#""seed" must have 1 element per 4 elements in the input "value" + 1"#
    );

    match value.len() {
        0 => extract_bits_64::<{ u64::BITS }>(seed[0], num_bits),
        1..=3 => {
            let mut padded = [0u8; 4];
            let mut i = 0;
            while i < value.len() {
                padded[i] = value[i];
                i += 1;
            }
            let value = u32::from_le_bytes(padded);
            multiply_shift(value, num_bits, unsafe {
                seed.first_chunk().unwrap_unchecked()
            })
        }
        4 => {
            let mut bytes = [0u8; 4];
            let mut i = 0;
            while i < 4 {
                bytes[i] = value[i];
                i += 1;
            }
            let value = u32::from_le_bytes(bytes);
            multiply_shift(value, num_bits, unsafe {
                seed.first_chunk().unwrap_unchecked()
            })
        }
        5..=7 => {
            let mut padded = [0u8; 8];
            let mut i = 0;
            while i < value.len() {
                padded[i] = value[i];
                i += 1;
            }
            let value = u64::from_le_bytes(padded);
            pair_multiply_shift(value, num_bits, unsafe {
                seed.first_chunk().unwrap_unchecked()
            })
        }
        8 => {
            let mut bytes = [0u8; 8];
            let mut i = 0;
            while i < 8 {
                bytes[i] = value[i];
                i += 1;
            }
            let value = u64::from_le_bytes(bytes);
            pair_multiply_shift(value, num_bits, unsafe {
                seed.first_chunk().unwrap_unchecked()
            })
        }
        _ => {
            let mut sum = seed[0];
            let num_chunks = (value.len() + 7) >> 3;
            let mut chunk_idx = 0;

            while chunk_idx < num_chunks {
                let byte_idx = chunk_idx << 3;
                let remaining_bytes = value.len() - byte_idx;
                let bytes_to_copy = if remaining_bytes < 8 {
                    remaining_bytes
                } else {
                    8
                };

                let mut bytes = [0u8; 8];

                unsafe {
                    copy_nonoverlapping(
                        value.as_ptr().add(byte_idx),
                        bytes.as_mut_ptr(),
                        bytes_to_copy,
                    );
                }

                let v = u64::from_le_bytes(bytes);

                let s_idx = 1 + chunk_idx * 2;

                let low = v;
                let high = v >> 32;

                sum = sum.wrapping_add(
                    seed[s_idx]
                        .wrapping_add(high)
                        .wrapping_mul(seed[s_idx + 1].wrapping_add(low)),
                );

                chunk_idx += 1;
            }

            extract_bits_64::<{ u64::BITS }>(sum, num_bits)
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

    #[test]
    fn test_pair_multiply_shift_vector_u64_const_equivalence() {
        let mut rng = ChaCha20Rng::from_os_rng();

        for vec_len in [1, 4, 8, 32, 256] {
            let non_const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                let mut seed = vec![0; vec_len * 2 + 1];
                seed.fill_with(|| rng.random());

                (
                    Box::new(move |value: &Vec<u64>| {
                        pair_multiply_shift_vector_u64(value.as_slice(), num_bits, &seed) as usize
                    }) as Box<dyn Fn(&Vec<u64>) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            let const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                let mut seed = vec![0; vec_len * 2 + 1];
                seed.fill_with(|| rng.random());

                (
                    Box::new(move |value: &Vec<u64>| {
                        pair_multiply_shift_vector_u64_const(value.as_slice(), num_bits, &seed)
                            as usize
                    }) as Box<dyn Fn(&Vec<u64>) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            equivalence(
                &mut rng,
                &non_const_family,
                &const_family,
                &|rng: &mut ChaCha20Rng| {
                    let mut key = vec![0_u64; vec_len];
                    key.fill_with(|| rng.random());
                    key
                },
                1000,
                99,
            );
        }
    }

    #[test]
    fn test_pair_multiply_shift_vector_u8_const_equivalence() {
        let mut rng = ChaCha20Rng::from_os_rng();

        for vec_len in [0_usize, 1, 3, 4, 5, 7, 8, 9, 16, 32, 64, 128] {
            let non_const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);

                // For larger arrays, we need to account for how pair_multiply_shift_vector_u8
                // internally converts bytes to u64s before calling pair_multiply_shift_vector_u64
                let u64_count = (vec_len + 7) >> 3; // ceiling(vec_len/8)
                let seed_len = if vec_len <= 8 {
                    3 // For short arrays (0-8 bytes), a seed of 3 u64s is enough
                } else {
                    // For long arrays, we need to match what pair_multiply_shift_vector_u64 expects
                    u64_count * 2 + 1
                };

                let mut seed = vec![0u64; seed_len];
                seed.fill_with(|| rng.random());

                (
                    Box::new(move |value: &Vec<u8>| {
                        pair_multiply_shift_vector_u8(value.as_slice(), num_bits, &seed) as usize
                    }) as Box<dyn Fn(&Vec<u8>) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            let const_family = |seed: u64, num_buckets: usize| {
                let mut rng = ChaCha20Rng::seed_from_u64(seed);
                let num_bits = num_bits_for_buckets(num_buckets as u32);

                // Use the same seed length calculation as above for consistency
                let u64_count = (vec_len + 7) >> 3; // ceiling(vec_len/8)
                let seed_len = if vec_len <= 8 {
                    3 // For short arrays (0-8 bytes), a seed of 3 u64s is enough
                } else {
                    // For long arrays, we need to match what pair_multiply_shift_vector_u64 expects
                    u64_count * 2 + 1
                };

                let mut seed = vec![0u64; seed_len];
                seed.fill_with(|| rng.random());

                (
                    Box::new(move |value: &Vec<u8>| {
                        pair_multiply_shift_vector_u8_const(value.as_slice(), num_bits, &seed)
                            as usize
                    }) as Box<dyn Fn(&Vec<u8>) -> usize>,
                    num_buckets_for_bits(num_bits) as usize,
                )
            };

            equivalence(
                &mut rng,
                &non_const_family,
                &const_family,
                &|rng: &mut ChaCha20Rng| {
                    let mut key = vec![0u8; vec_len];
                    if vec_len > 0 {
                        key.fill_with(|| rng.random::<u8>());
                    }
                    key
                },
                1000,
                99,
            );
        }
    }
}
