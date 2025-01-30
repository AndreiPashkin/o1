//! Implementation of the multiply-shift hashing scheme (multiply-add-shift to be precise)
//! from [Dietzfelbinger (1996)], [Dietzfelbinger et al. (1997)] and [Thorup (2015)].
//!
//! Multiply-shift hashing scheme is an evolution of multiply-mod-prime that avoids
//! using expensive arithmetic operations like modulo and division and instead exploits integer
//! overflow and also bitshift operation.
//!
//! [Dietzfelbinger (1996)]: https://link.springer.com/chapter/10.1007/3-540-60922-9_46
//! [Dietzfelbinger et al. (1997)]: https://linkinghub.elsevier.com/retrieve/pii/S0196677497908737
//! [Thorup (2015)]: http://arxiv.org/abs/1504.06804
use crate::hashing::common::extract_bits_32;

// TODO: Consider implementing the weakly-universal version of multiply-shift that returns u64.
// TODO: Generally in the future 64-bit versions will probably be needed too.

/// Hashes a 32-bit unsigned integer using the multiply-shift hashing scheme.
///
/// # Parameters
/// - `value`: The input value.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed.
///
/// # Guarantees
/// - Strong universality.
#[inline]
pub const fn multiply_shift(value: u32, num_bits: u32, seed: &[u64; 2]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    let hash = seed[0].wrapping_mul(value as u64).wrapping_add(seed[1]);
    extract_bits_32(hash, num_bits)
}

/// Hashes a 64-bit unsigned integer using the pair-multiply-shift hashing scheme.
///
/// # Parameters
/// - `value`: The input value.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed.
///
/// # Guarantees
/// - Strong universality.
#[inline]
pub const fn pair_multiply_shift(value: u64, num_bits: u32, seed: &[u64; 3]) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    let hash_value = seed[0]
        .wrapping_add(value)
        .wrapping_mul(seed[1].wrapping_add(value >> 32))
        .wrapping_add(seed[2]);

    extract_bits_32(hash_value, num_bits)
}

/// Hashes a vector of 64-bit unsigned integers of length `LEN` to a 32-bit hash value.
///
/// # Parameters
/// - `LEN`: The length of the input vector.
/// - `value`: The input vector.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed.
///
/// # Guarantees
/// - Strong universality.
#[inline]
pub fn multiply_shift_vector_u64<const LEN: usize>(
    value: &[u64; LEN],
    num_bits: u32,
    seed: (&[u64; LEN], u64),
) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    // TODO: Usage of SIMD instructions suggests itself here.
    // Compute the dot-product of the input vector and the seed vector.
    let mut sum: u64 = seed.1;
    #[allow(clippy::needless_range_loop)]
    for i in 0..LEN {
        sum = sum.wrapping_add(seed.0[i].wrapping_mul(value[i]));
    }

    extract_bits_32(sum, num_bits)
}

/// Hashes a string (a vector of bytes) of length `LEN` to a 32-bit hash value.
///
/// # Parameters
/// - `LEN`: The length of the string.
/// - `value`: The input vector.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `seed`: Random seed.
///
/// # Guarantees
/// - Strong universality.
#[inline]
pub fn multiply_shift_vector_u8<const LEN: usize>(
    value: &[u8; LEN],
    num_bits: u32,
    seed: (&[u64; LEN], u64),
) -> u32 {
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);

    if let Some(value) = value.first_chunk::<4>() {
        // Handle small byte-arrays (4 bytes) - use `multiply_shift()`.
        let value = u32::from_le_bytes(*value);
        let value_seed: u64 = seed.0[0];
        return multiply_shift(value, num_bits, &[value_seed, seed.1]);
    } else if let Some(value) = value.first_chunk::<8>() {
        // Handle small byte-arrays (8 bytes) - use `pair_multiply_shift()`.
        let value = u64::from_le_bytes(*value);
        let value_seed: &[u64; 2] = unsafe { seed.0.first_chunk().unwrap_unchecked() };

        return pair_multiply_shift(value, num_bits, &[value_seed[0], value_seed[1], seed.1]);
    }

    // Compute the dot-product of the input vector and the seed vector.
    let mut sum: u64 = seed.1;
    #[allow(clippy::needless_range_loop)]
    for i in 0..LEN {
        // TODO: Could be optimized by processing the input by 64-bit words.
        // TODO: Same as with the vector version - SIMD instructions could be used.
        sum = sum.wrapping_add((seed.0[i]).wrapping_mul(value[i] as u64));
    }

    extract_bits_32(sum, num_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::num_bits_for_buckets;
    use crate::testing::*;
    use rand::prelude::*;

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_multiply_shift_universality_guarantee() {
        check_universality_guarantee::<u32>(
            UniversalityGuarantee::Strong,
            99,
            &|seed, num_buckets| {
                let mut rng = SmallRng::seed_from_u64(seed);
                let seed: [u64; 2] = rng.gen();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &u32| multiply_shift(*value, num_bits, &seed) as usize)
            },
            &u32::generate_many(&mut SmallRng::from_entropy(), &NumParams::default(), 999999)
                .into_vec(),
        );
    }

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_pair_multiply_shift_universality_guarantee() {
        check_universality_guarantee::<u64>(
            UniversalityGuarantee::Strong,
            99,
            &|seed, num_buckets| {
                let mut rng = SmallRng::seed_from_u64(seed);
                let seed: [u64; 3] = rng.gen();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &u64| pair_multiply_shift(*value, num_bits, &seed) as usize)
            },
            &u64::generate_many(&mut SmallRng::from_entropy(), &NumParams::default(), 999999)
                .into_vec(),
        );
    }

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_multiply_shift_vector_u64_universality_guarantee() {
        check_universality_guarantee::<[u64; 10]>(
            UniversalityGuarantee::Strong,
            99,
            &|seed, num_buckets| {
                let mut rng = SmallRng::seed_from_u64(seed);
                let seed: ([u64; 10], u64) = rng.gen();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &[u64; 10]| {
                    multiply_shift_vector_u64(value, num_bits, (&seed.0, seed.1)) as usize
                })
            },
            &<[u64; 10]>::generate_many(
                &mut SmallRng::from_entropy(),
                &NumParams::default(),
                999999,
            )
            .into_vec(),
        );
    }

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_multiply_shift_vector_u8_universality_guarantee() {
        check_universality_guarantee::<[u8; 10]>(
            UniversalityGuarantee::Strong,
            99,
            &|seed, num_buckets| {
                let mut rng = SmallRng::seed_from_u64(seed);
                let seed: ([u64; 10], u64) = rng.gen();
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &[u8; 10]| {
                    multiply_shift_vector_u8(value, num_bits, (&seed.0, seed.1)) as usize
                })
            },
            &<[u8; 10]>::generate_many(
                &mut SmallRng::from_entropy(),
                &NumParams::default(),
                999999,
            )
            .into_vec(),
        );
    }
}
