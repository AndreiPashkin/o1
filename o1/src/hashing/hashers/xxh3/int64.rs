//! Implements Hasher for u64 and i64 using the XXH3 hash function.

use super::core::XXH3Hasher;
use crate::hashing::common::{extract_bits_64, num_bits_for_buckets, num_buckets_for_bits};
use o1_core::Hasher;
use xxhash_rust::const_xxh3::xxh3_64_with_seed as xxh3_64_with_seed_const;
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[derive(Debug, Default, Clone, Copy)]
pub struct U64State {
    num_bits: u32,
    seed: u64,
}

impl U64State {
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );
        Self { num_bits, seed }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );
        Self { num_bits, seed }
    }
}

#[inline]
fn hash(state: &U64State, value: u64) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    let bytes = value.to_le_bytes();
    let hash_value = xxh3_64_with_seed(bytes.as_slice(), state.seed);

    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

#[inline]
const fn hash_const(state: &U64State, value: u64) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );
    let bytes = value.to_le_bytes();
    let hash_value = xxh3_64_with_seed_const(bytes.as_slice(), state.seed);

    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

macro_rules! impl_xxh3_int_64 {
    ($($int_type:ty),*) => {
        $(
            impl Hasher<$int_type> for XXH3Hasher<$int_type> {
                type State = U64State;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    U64State::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::State::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self {
                    Self { state }
                }
                fn state(&self) -> &Self::State {
                    &self.state
                }
                fn num_buckets(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                fn hash(&self, value: &$int_type) -> u32 {
                    hash(&self.state, *value as u64)
                }
            }

            impl XXH3Hasher<$int_type> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> U64State {
                    U64State::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = U64State::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<$int_type>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &$int_type) -> u32 {
                    hash_const(&self.state, *value as u64)
                }
            }
        )*
    };
}

impl_xxh3_int_64!(u64, i64);
#[cfg(target_pointer_width = "64")]
impl_xxh3_int_64!(usize, isize);

/// Array state for fixed-size arrays of u64/i64.
#[derive(Debug, Clone, Copy)]
pub struct Array64State<const N: usize> {
    num_bits: u32,
    seed: u64,
}

impl<const N: usize> Default for Array64State<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
        }
    }
}

impl<const N: usize> Array64State<N> {
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        Self { num_bits, seed }
    }

    const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits > 0 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#,
        );

        Self { num_bits, seed }
    }
}

macro_rules! impl_for_array {
    ($($type:ty),*) => {
        $(
            impl <const N: usize>Hasher<[$type; N]> for XXH3Hasher<[$type; N]> {
                type State = Array64State<N>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    Array64State::from_seed(seed, num_buckets)
                }

                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::State::from_seed(seed, num_buckets);
                    Self { state }
                }

                fn from_state(state: Self::State) -> Self {
                    Self { state }
                }

                fn state(&self) -> &Self::State {
                    &self.state
                }

                fn num_buckets(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }

                fn hash(&self, value: &[$type; N]) -> u32 {
                    debug_assert!(
                        (1..=32).contains(&self.state.num_bits),
                        r#""num_bits" must be [1, 32]"#
                    );
                    let bytes_len = N * core::mem::size_of::<$type>();
                    let bytes = unsafe {
                        core::slice::from_raw_parts(value.as_ptr() as *const u8, bytes_len)
                    };
                    let hash_value = xxh3_64_with_seed(bytes, self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }

            impl <const N: usize>XXH3Hasher<[$type; N]> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> <Self as Hasher<[$type; N]>>::State {
                    Array64State::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = Array64State::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<[$type; N]>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &[$type; N]) -> u32 {
                    debug_assert!(
                        self.state.num_bits >= 1 && self.state.num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );
                    let mut byte_array = [[0u8; 8]; N];
                    let mut i = 0;
                    while i < N {
                        byte_array[i] = value[i].to_le_bytes();
                        i += 1;
                    }
                    let bytes = unsafe {
                        core::slice::from_raw_parts(byte_array.as_ptr() as *const u8, N * 8)
                    };
                    let hash_value = xxh3_64_with_seed_const(bytes, self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }
        )*
    };
}

impl_for_array!(u64, i64);
#[cfg(target_pointer_width = "64")]
impl_for_array!(usize, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;

    generate_hasher_tests!(XXH3Hasher<u64>, u64, |rng: &mut ChaCha20Rng| rng
        .random::<u64>());
    generate_hasher_tests!(XXH3Hasher<i64>, i64, |rng: &mut ChaCha20Rng| rng
        .random::<i64>());
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        XXH3Hasher<usize>,
        usize,
        |rng: &mut ChaCha20Rng| rng.random::<u64>() as usize
    );
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        XXH3Hasher<isize>,
        isize,
        |rng: &mut ChaCha20Rng| rng.random::<i64>() as isize
    );

    generate_hasher_tests!(XXH3Hasher<[u64; 32]>, [u64; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[u64; 32]>()
    });
    generate_hasher_tests!(XXH3Hasher<[i64; 32]>, [i64; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[i64; 32]>()
    });
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        XXH3Hasher<[usize; 32]>,
        [usize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[u64; 32]>() as *const [u64; 32] as *const [usize; 32])
        }
    );
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        XXH3Hasher<[isize; 32]>,
        [isize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[i64; 32]>() as *const [i64; 32] as *const [isize; 32])
        }
    );
}
