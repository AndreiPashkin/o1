//! Implements [`Hasher`] for the integer size larger than 64-bit.
//!
//! # Notes
//!
//! Internally it treats big integers as vectors uses the [`multiply_shift_u8`] hash function.

use super::core::MSPHasher;
use crate::core::Hasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::pair_multiply_shift_vector_u8;
use core::mem::size_of;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Clone)]
pub struct BigIntState<T>
where
    T: Clone + Default,
{
    pub(super) num_bits: u32,
    seed: Vec<u64>,
    _type: std::marker::PhantomData<T>,
}

impl<T> Default for BigIntState<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: vec![0; size_of::<T>().div_ceil(4) + 1],
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
        let seed: Vec<u64> = (0..size_of::<T>().div_ceil(4) + 1)
            .map(|_| rng.random())
            .collect();

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
}

/// Generates [`Hasher`] and implementations for "big" integer types.
macro_rules! impl_multiply_shift_big_int {
    ($($T:ty),*) => {
        $(
            impl Hasher<$T> for MSPHasher<$T> {
                type State = BigIntState<$T>;

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
                    pair_multiply_shift_vector_u8(
                        &value.to_le_bytes(),
                        self.state.num_bits,
                        &self.state.seed,
                    )
                }
            }
        )*
    };
}

impl_multiply_shift_big_int!(u128, i128, usize, isize);
