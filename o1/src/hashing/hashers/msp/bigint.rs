//! Implements [`Hasher`] for the integer size larger than 64-bit.
//!
//! # Notes
//!
//! Internally it treats big integers as vectors uses the [`multiply_shift_u8`] hash function.

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::{
    pair_multiply_shift_u128, pair_multiply_shift_vector_u128,
    pair_multiply_shift_vector_u128_const,
};
use crate::utils::xorshift::generate_random_array;
use o1_core::Hasher;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

const SEED_LEN: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct BigIntState<T>
where
    T: Clone + Default,
{
    pub(super) num_bits: u32,
    seed: [u64; SEED_LEN],
    _type: std::marker::PhantomData<T>,
}

impl<T> Default for BigIntState<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: [0; SEED_LEN],
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> BigIntState<T>
where
    T: Default + Clone,
{
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let seed = rng.random();

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed,
            _type: std::marker::PhantomData,
        }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let seed = generate_random_array!(u64, SEED_LEN, seed);

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed,
            _type: std::marker::PhantomData,
        }
    }
}

/// Generates [`Hasher`] and implementations for "big" integer types.
macro_rules! impl_multiply_shift_big_int {
    ($($T:ty),*) => {
        $(
            impl Hasher<$T> for MSPHasher<$T> {
                type State = BigIntState<$T>;

                fn make_state(seed: u64, num_buckets: u32) -> BigIntState<$T> {
                    BigIntState::from_seed(seed, num_buckets)
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
                fn hash(&self, value: &$T) -> u32 {
                    pair_multiply_shift_u128(
                        *value as u128,
                        self.state.num_bits,
                        &self.state.seed,
                    )
                }
            }

            impl MSPHasher<$T> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> BigIntState<$T> {
                    BigIntState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = BigIntState::<$T>::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<$T>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &$T) -> u32 {
                    pair_multiply_shift_u128(
                        *value as u128,
                        self.state.num_bits,
                        &self.state.seed,
                    )
                }
            }
        )*
    };
}

impl_multiply_shift_big_int!(u128, i128);

#[derive(Debug, Clone, Copy)]
pub struct BigIntArrayState<const N: usize> {
    num_bits: u32,
    seed: u64,
    // 4 seed-values per 128-bit element.
    value_seed: [[u64; 4]; N],
}

impl<const N: usize> Default for BigIntArrayState<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
            value_seed: [[0; 4]; N],
        }
    }
}

impl<const N: usize> BigIntArrayState<N> {
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        let seed = rng.random();
        let value_seed = rng.random();

        Self {
            num_bits,
            seed,
            value_seed,
        }
    }

    const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        let mut value_seed = [[0u64; 4]; N];
        let mut i = 0;
        while i < N {
            value_seed[i] = generate_random_array!(u64, 4, seed);
            i += 1;
        }

        Self {
            num_bits,
            seed,
            value_seed,
        }
    }

    const fn value_seed_as_slice(&self) -> &[u64] {
        unsafe { core::slice::from_raw_parts(self.value_seed.as_ptr() as *const u64, N * 4) }
    }
}

macro_rules! impl_bigint_array_hasher {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> Hasher<[$t; N]> for MSPHasher<[$t; N]> {
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
                    let value_u: &[u128; N] = unsafe { &*(value as *const [$t; N] as *const [u128; N]) };
                    pair_multiply_shift_vector_u128(
                        value_u.as_slice(),
                        self.state.num_bits,
                        self.state.seed,
                        self.state.value_seed_as_slice(),
                    )
                }
            }

            impl<const N: usize> MSPHasher<[$t; N]> {
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
                    let value_u: &[u128; N] = unsafe { &*(value as *const [$t; N] as *const [u128; N]) };
                    pair_multiply_shift_vector_u128_const(
                        value_u.as_slice(),
                        self.state.num_bits,
                        self.state.seed,
                        self.state.value_seed_as_slice(),
                    )
                }
            }
        )*
    };
}

impl_bigint_array_hasher!(u128, i128);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_testing::generate_hasher_tests;

    generate_hasher_tests!(MSPHasher<u128>, u128, |rng: &mut ChaCha20Rng| rng
        .random::<u128>());
    generate_hasher_tests!(MSPHasher<i128>, i128, |rng: &mut ChaCha20Rng| rng
        .random::<i128>());
    generate_hasher_tests!(MSPHasher<[u128; 8]>, [u128; 8], |rng: &mut ChaCha20Rng| rng
        .random::<[u128; 8]>());
    generate_hasher_tests!(MSPHasher<[i128; 8]>, [i128; 8], |rng: &mut ChaCha20Rng| rng
        .random::<[i128; 8]>());
}
