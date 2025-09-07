//! Implements Hasher for u64 and i64 using [`pair_multiply_shift`] hash-function.

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::{
    pair_multiply_shift, pair_multiply_shift_vector_u64, pair_multiply_shift_vector_u64_const,
};
use crate::utils::xorshift::generate_random_array;
use o1_core::Hasher;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Default, Clone, Copy)]
pub struct U64State {
    num_bits: u32,
    seed: [u64; 3],
}

impl U64State {
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let seed: [u64; 3] = rng.random();
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        Self { num_bits, seed }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let seed: [u64; 3] = generate_random_array!(u64, 3, seed);
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
    pair_multiply_shift(value, state.num_bits, &state.seed)
}

#[inline]
const fn hash_const(state: &U64State, value: u64) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );
    pair_multiply_shift(value, state.num_bits, &state.seed)
}

macro_rules! impl_multiply_shift_int_64 {
    ($($int_type:ty),*) => {
        $(
            impl Hasher<$int_type> for MSPHasher<$int_type> {
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

            impl MSPHasher<$int_type> {
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

impl_multiply_shift_int_64!(u64, i64);
#[cfg(target_pointer_width = "64")]
impl_multiply_shift_int_64!(usize, isize);

// Array state for fixed-size arrays of u64/i64
// Each u64 element needs a pair of u64 seeds (2 values)
#[derive(Debug, Clone, Copy)]
pub struct Array64State<const N: usize> {
    num_bits: u32,
    seed: u64,
    // 2 seed-values per 64-bit element.
    value_seed: [[u64; 2]; N],
}

impl<const N: usize> Default for Array64State<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
            value_seed: [[0; 2]; N],
        }
    }
}

impl<const N: usize> Array64State<N> {
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
            num_bits > 0 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#,
        );

        let mut value_seed = [[0; 2]; N];
        let mut i = 0;
        while i < N {
            let pair = &mut value_seed[i];
            *pair = generate_random_array!(u64, 2, seed);
            i += 1;
        }

        Self {
            num_bits,
            seed,
            value_seed,
        }
    }

    const fn value_seed_as_slice(&self) -> &[u64] {
        unsafe { std::slice::from_raw_parts(self.value_seed.as_ptr() as *const u64, N * 2) }
    }
}

#[inline]
fn hash_array<const N: usize>(state: &Array64State<N>, array: &[u64; N]) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    pair_multiply_shift_vector_u64(
        array,
        state.num_bits,
        state.seed,
        state.value_seed_as_slice(),
    )
}

#[inline]
const fn hash_array_const<const N: usize>(state: &Array64State<N>, array: &[u64; N]) -> u32 {
    debug_assert!(
        state.num_bits > 0 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#,
    );
    pair_multiply_shift_vector_u64_const(
        array,
        state.num_bits,
        state.seed,
        state.value_seed_as_slice(),
    )
}

// Generic implementation for arrays
macro_rules! impl_for_array {
    ($($type:ty),*) => {
        $(
            impl <const N: usize>Hasher<[$type; N]> for MSPHasher<[$type; N]> {
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
                    let value = unsafe { &*(value as *const [$type; N] as *const [u64; N]) };
                    hash_array(&self.state, value)
                }
            }

            impl <const N: usize>MSPHasher<[$type; N]> {
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
                    let value = unsafe { &*(value as *const [$type; N] as *const [u64; N]) };
                    hash_array_const(&self.state, value)
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

    generate_hasher_tests!(MSPHasher<u64>, u64, |rng: &mut ChaCha20Rng| rng
        .random::<u64>());
    generate_hasher_tests!(MSPHasher<i64>, i64, |rng: &mut ChaCha20Rng| rng
        .random::<i64>());
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        MSPHasher<usize>,
        usize,
        |rng: &mut ChaCha20Rng| rng.random::<u64>() as usize
    );
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        MSPHasher<isize>,
        isize,
        |rng: &mut ChaCha20Rng| rng.random::<i64>() as isize
    );

    generate_hasher_tests!(MSPHasher<[u64; 32]>, [u64; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[u64; 32]>());
    generate_hasher_tests!(MSPHasher<[i64; 32]>, [i64; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[i64; 32]>());
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        MSPHasher<[usize; 32]>,
        [usize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[u64; 32]>() as *const [u64; 32] as *const [usize; 32])
        }
    );
    #[cfg(target_pointer_width = "64")]
    generate_hasher_tests!(
        MSPHasher<[isize; 32]>,
        [isize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[i64; 32]>() as *const [i64; 32] as *const [isize; 32])
        }
    );
}
