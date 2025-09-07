//! Implements Hasher for integers larger than 64-bit (u128, i128) using the XXH3 hash function.

use super::core::XXH3Hasher;
use crate::hashing::common::{extract_bits_64, num_bits_for_buckets, num_buckets_for_bits};
use o1_core::Hasher;
use xxhash_rust::const_xxh3::xxh3_64_with_seed as xxh3_64_with_seed_const;
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[derive(Debug, Clone, Copy)]
pub struct BigIntState<T>
where
    T: Clone + Default,
{
    pub(super) num_bits: u32,
    seed: u64,
    _type: core::marker::PhantomData<T>,
}

impl<T> Default for BigIntState<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
            _type: core::marker::PhantomData,
        }
    }
}

impl<T> BigIntState<T>
where
    T: Default + Clone,
{
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed,
            _type: core::marker::PhantomData,
        }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed,
            _type: core::marker::PhantomData,
        }
    }
}

macro_rules! impl_xxh3_big_int {
    ($($T:ty),*) => {
        $(
            impl Hasher<$T> for XXH3Hasher<$T> {
                type State = BigIntState<$T>;

                fn make_state(seed: u64, num_buckets: u32) -> BigIntState<$T> {
                    BigIntState::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::State::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &$T) -> u32 {
                    let bytes = value.to_le_bytes();
                    let hash_value = xxh3_64_with_seed(bytes.as_slice(), self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }

            impl XXH3Hasher<$T> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> BigIntState<$T> {
                    BigIntState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = BigIntState::<$T>::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<$T>>::State) -> Self { Self { state } }
                pub const fn num_buckets_const(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                pub const fn hash_const(&self, value: &$T) -> u32 {
                    let bytes = value.to_le_bytes();
                    let hash_value = xxh3_64_with_seed_const(bytes.as_slice(), self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }
        )*
    };
}

impl_xxh3_big_int!(u128, i128);

#[derive(Debug, Clone, Copy)]
pub struct BigIntArrayState<const N: usize> {
    num_bits: u32,
    seed: u64,
}

impl<const N: usize> Default for BigIntArrayState<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
        }
    }
}

impl<const N: usize> BigIntArrayState<N> {
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
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        Self { num_bits, seed }
    }
}

#[inline]
fn hash_array<const N: usize, T>(state: &BigIntArrayState<N>, value: &[T; N]) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    let bytes_len = N * core::mem::size_of::<T>();
    let bytes = unsafe { core::slice::from_raw_parts(value.as_ptr() as *const u8, bytes_len) };
    let hash_value = xxh3_64_with_seed(bytes, state.seed);
    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

macro_rules! impl_bigint_array_hasher {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> Hasher<[$t; N]> for XXH3Hasher<[$t; N]> {
                type State = BigIntArrayState<N>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    BigIntArrayState::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = BigIntArrayState::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &[$t; N]) -> u32 {
                    hash_array::<N, $t>(&self.state, value)
                }
            }

            impl<const N: usize> XXH3Hasher<[$t; N]> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> <Self as Hasher<[$t; N]>>::State {
                    BigIntArrayState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = BigIntArrayState::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<[$t; N]>>::State) -> Self { Self { state } }
                pub const fn num_buckets_const(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                pub const fn hash_const(&self, value: &[$t; N]) -> u32 {
                    debug_assert!(
                        self.state.num_bits >= 1 && self.state.num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );
                    let mut byte_array = [[0u8; 16]; N];
                    let mut i = 0;
                    while i < N {
                        byte_array[i] = value[i].to_le_bytes();
                        i += 1;
                    }
                    let bytes = unsafe {
                        core::slice::from_raw_parts(byte_array.as_ptr() as *const u8, N * 16)
                    };
                    let hash_value = xxh3_64_with_seed_const(bytes, self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }
        )*
    };
}

impl_bigint_array_hasher!(u128, i128);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;

    generate_hasher_tests!(XXH3Hasher<u128>, u128, |rng: &mut ChaCha20Rng| rng
        .random::<u128>());
    generate_hasher_tests!(XXH3Hasher<i128>, i128, |rng: &mut ChaCha20Rng| rng
        .random::<i128>());
    generate_hasher_tests!(XXH3Hasher<[u128; 8]>, [u128; 8], |rng: &mut ChaCha20Rng| {
        rng.random::<[u128; 8]>()
    });
    generate_hasher_tests!(XXH3Hasher<[i128; 8]>, [i128; 8], |rng: &mut ChaCha20Rng| {
        rng.random::<[i128; 8]>()
    });
}
