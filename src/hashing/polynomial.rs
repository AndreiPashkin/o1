//! Polynomial hashing implementation for arbitrary-size values based
//! on [Dietzfelbinger et al. (1992)] and [Thorup (2015)].
//!
//! The core idea of polynomial hashing is to treat the input vector as coefficients for a
//! polynomial and then compute it efficiently using Horner's rule and optimized modulo operation
//! known for Mersenne primes.
//!
//! [Dietzfelbinger et al. (1992)]: https://link.springer.com/chapter/10.1007/3-540-55719-9_77
//! [Thorup (2015)]: http://arxiv.org/abs/1504.06804

use crate::hashing::common::{extract_bits_32, mod_mersenne_prime};
use crate::hashing::multiply_shift::pair_multiply_shift;
use core::mem::align_of;

/// Combines two independent hash values (hash1 and hash2) by concatenating them
/// into a 64-bit value. This ensures strong universality by using two distinct
/// pair-multiply-shift hash functions.
#[inline(always)]
const fn concat_pair_multiply_shift(value: u64, h1_seed: &[u64; 3], h2_seed: &[u64; 3]) -> u64 {
    let hash1 = pair_multiply_shift(value, 32, h1_seed);
    let hash2 = pair_multiply_shift(value, 32, h2_seed);

    // TODO: I wonder if usage of independent hash functions could be replaced with
    //       more advanced concatenation (TAOCP v3 p. 519)
    //       to avoid excessive generation of random numbers?
    ((hash1 as u64) << 32) | (hash2 as u64)
}

/// Hashes a 32-bit unsigned integer using the multiply-shift hashing scheme.
///
/// # Parameters
/// - `value`: The input bytes.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
/// - `p`: Large Mersenne prime. `2 ** 89 − 1` could be a practical value.
/// - `p_e`: Exponent of the Mersenne prime.
/// - `seed`: Random seed.
///
/// # Guarantees
/// - Strong universality.
#[inline]
pub fn polynomial(value: &[u8], num_bits: u32, p: u128, p_e: u32, seed: &[u64; 8]) -> u32 {
    // TODO: Clarify the constraints up to which the function gives strong
    //       universality guarantees.
    let a: u64 = seed[0];
    let b: u64 = seed[1];

    // TODO: Replace with something cleaner?
    let (h1_seed, h2_seed): (&[u64; 3], &[u64; 3]) = unsafe {
        (
            &*(seed.as_ptr().add(2) as *const [u64; 3]),
            &*(seed.as_ptr().add(5) as *const [u64; 3]),
        )
    };

    // TODO: Add more assertions to all functions to clarify constraints
    //       for other parameters (like `a` and `p`).
    debug_assert!(num_bits <= 32, r#""num_bits" must be <= 32"#);
    debug_assert!(
        a > 0 && (a as u128) < p,
        r#""seed[0]" must be in the range [1, p-1]"#
    );
    debug_assert!(
        (a as u128) < p,
        r#""seed[1]" must be in the range [0, p-1]"#
    );

    let mut hash_value: u64 = 0;

    let mut value = value;

    // Handle misaligned pointer:
    // - Process the misaligned leading part
    // - Leave the aligned part for further processing
    let prefix_len = align_of::<u64>() - value.as_ptr() as usize % align_of::<u64>();
    if prefix_len > 0 {
        let prefix_len = prefix_len.min(value.len());

        let mut prefix_word: u64 = 0;
        let mut prefix_hash: u64 = 0;

        for (i, &byte) in value[..prefix_len].iter().enumerate() {
            prefix_word |= (byte as u64) << (8 * i);
        }
        prefix_hash ^= concat_pair_multiply_shift(prefix_word, h1_seed, h2_seed);

        // Apply polynomial hashing using Horner’s rule
        hash_value = prefix_hash;
        value = &value[prefix_len..];
    }

    if !value.is_empty() {
        // Process the leading aligned part by reinterpreting it as a slice of u64
        let aligned_len = value.len() & !7; // Largest multiple of 8 less than or equal to s.len()
        let words_count = aligned_len / 8;
        // TODO: Not sure if this sort trick is really beneficial here. Need to benchmark.
        let words: &[u64] =
            unsafe { core::slice::from_raw_parts(value.as_ptr() as *const u64, words_count) };

        for (i, chunk) in words.chunks(32).enumerate() {
            let mut chunk_hash: u64 = 0;

            for word in chunk {
                chunk_hash ^= concat_pair_multiply_shift(*word, h1_seed, h2_seed);
            }
            // Apply polynomial hashing using Horner’s rule
            if i == 0 && prefix_len == 0 {
                hash_value = chunk_hash;
            } else {
                hash_value =
                    mod_mersenne_prime(hash_value.wrapping_mul(a).wrapping_add(chunk_hash), p, p_e);
            }
        }

        // TODO: This could be optimized by matching against each of the 7 corner cases
        //       of possible remainders.
        // Process the unaligned remainder part
        let mut remainder_hash: u64 = 0;
        let mut remainder_word: u64 = 0;
        let remainder_len = value.len() - aligned_len;
        for i in 0..remainder_len {
            remainder_word |= (value[aligned_len + i] as u64) << (8 * i);
        }
        remainder_hash ^= concat_pair_multiply_shift(remainder_word, h1_seed, h2_seed);

        // Apply polynomial hashing using Horner’s rule once again
        hash_value = mod_mersenne_prime(
            hash_value.wrapping_mul(a).wrapping_add(remainder_hash),
            p,
            p_e,
        );
    }

    // Add the random constant b and reduce modulo p
    hash_value = mod_mersenne_prime(hash_value.wrapping_add(b), p, p_e);

    extract_bits_32(hash_value, num_bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::num_bits_for_buckets;
    use crate::testing::*;
    use rand::prelude::SmallRng;
    use rand::{Rng, SeedableRng};

    #[test]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_polynomial_universality_guarantee() {
        let p_e = 89;
        let p = 2_u128.pow(p_e) - 1;
        let keys = String::generate_many(
            &mut SmallRng::from_entropy(),
            &StringParams::new(3, 99),
            999999,
        )
        .into_vec();
        let num_trials = 99;
        let family = Box::new(|seed: u64, num_buckets: usize| {
            let mut rng = SmallRng::seed_from_u64(seed);
            let seed: [u64; 8] = [
                rng.gen_range(1..=p - 1) as u64,
                rng.gen_range(0..=p - 1) as u64,
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
                rng.gen(),
            ];

            Box::new(move |key: &String| {
                polynomial(
                    key.as_bytes(),
                    num_bits_for_buckets(num_buckets as u32),
                    p,
                    p_e,
                    &seed,
                ) as usize
            }) as Box<dyn Fn(&String) -> usize>
        });

        check_universality_guarantee::<String>(
            UniversalityGuarantee::Strong,
            num_trials,
            &family,
            &keys,
        );
    }
}
