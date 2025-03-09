//! Implements intentionally flawed hash function - mainly to serve as negative examples for the
//! empirical testing suite.

/// Hashes a 64-bit value into a 32-bit hash by extracting the lowest `num_bits` bits.
///
/// Intentionally produces collisions.
///
/// # Parameters
///
/// - `x`: The input value.
/// - `num_bits`: Number of bits in the output hash. Hash range would be equal to `2 ** num_bits`.
#[allow(dead_code)]
pub fn lowest_bits(x: u64, num_bits: u32) -> u32 {
    (x & ((1 << num_bits) - 1)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::common::num_bits_for_buckets;
    use crate::testing::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha20Rng;

    #[test]
    #[should_panic]
    #[cfg_attr(not(feature = "_slow-tests"), ignore)]
    fn test_lowest_bits_strong_universality_guarantee() {
        let mut rng = ChaCha20Rng::from_os_rng();

        strong_universality::<ChaCha20Rng, u64>(
            &mut rng,
            &|_, num_buckets| {
                let num_bits = num_bits_for_buckets(num_buckets as u32);
                Box::new(move |value: &u64| lowest_bits(*value, num_bits) as usize)
            },
            16,
            &|num_buckets| num_buckets.next_power_of_two(),
            15,
            1000,
            0.01,
        );
    }
}
