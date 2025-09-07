//! Implements [`Hasher`] for all integer types of size equal to or smaller than 32-bits.
//! Casts non-`u32` inputs to `u32` and then uses [`multiply_shift`] hash function.
//!
//! # Notes
//!
//! - It is obviously not optimal to hash 8-bit and 16-bit integers like this - by upcasting them
//!   first, there should be specialized hash functions for these cases, so it's a TODO.

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::{
    multiply_shift, pair_multiply_shift, pair_multiply_shift_vector_u8,
};
use crate::utils::xorshift::generate_random_array;
use o1_core::Hasher;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Default, Clone, Copy)]
pub struct SmallIntState {
    num_bits: u32,
    seed: [u64; 2],
}

impl SmallIntState {
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let seed: [u64; 2] = rng.random();
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        Self { num_bits, seed }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let mut seed: [u64; 2] = generate_random_array!(u64, 2, seed);
        seed[0] |= 1;
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        Self { num_bits, seed }
    }
}

#[inline]
const fn hash(state: &SmallIntState, value: u32) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );
    multiply_shift(value, state.num_bits, &state.seed)
}

impl Hasher<u32> for MSPHasher<u32> {
    type State = SmallIntState;

    fn make_state(seed: u64, num_buckets: u32) -> Self::State {
        SmallIntState::from_seed(seed, num_buckets)
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
    fn hash(&self, value: &u32) -> u32 {
        hash(&self.state, *value)
    }
}

impl MSPHasher<u32> {
    pub const fn make_state_const(seed: u64, num_buckets: u32) -> SmallIntState {
        SmallIntState::from_seed_const(seed, num_buckets)
    }
    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        let state = SmallIntState::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state_const(state: <Self as Hasher<u32>>::State) -> Self {
        Self { state }
    }
    pub const fn num_buckets_const(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash_const(&self, value: &u32) -> u32 {
        hash(&self.state, *value)
    }
}

/// Generates [`Hasher`] and implementations for all other "small" integer types.
///
/// The generated impls simply call the `u32` implementation and cast the input to `u32`.
macro_rules! impl_multiply_shift_small_int {
    ($($k:ty),*) => {
        $(
            impl Hasher<$k> for MSPHasher<$k> {
                type State = SmallIntState;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    SmallIntState::from_seed(seed, num_buckets)
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
                fn hash(&self, value: &$k) -> u32 {
                    hash(&self.state, (*value) as u32)
                }
            }

            impl MSPHasher<$k> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> SmallIntState {
                    SmallIntState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = SmallIntState::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<$k>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &$k) -> u32 {
                    hash(&self.state, (*value) as u32)
                }
            }
        )*
    };
}

impl_multiply_shift_small_int!(i32, u16, i16, u8, i8);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_multiply_shift_small_int!(usize, isize);

// -----------------------------------------------------------------------------
// Fixed-size array support for small integer types (<= 32 bits)
// -----------------------------------------------------------------------------

// We mirror the approach used for 64-bit arrays in int64.rs:
// - Maintain a global seed (u64) and per-position seeds (two u64 values per element)
// - Hash arrays by treating them as a byte vector and using the same vector-u8
//   multiply-shift scheme for both runtime and const paths

#[derive(Debug, Clone, Copy)]
pub struct SmallArrayState<const N: usize> {
    num_bits: u32,
    seed: u64,
    // 2 seed-values per element.
    value_seed: [[u64; 2]; N],
}

impl<const N: usize> Default for SmallArrayState<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
            value_seed: [[0; 2]; N],
        }
    }
}

impl<const N: usize> SmallArrayState<N> {
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

        let mut value_seed = [[0u64; 2]; N];
        let mut i = 0;
        while i < N {
            value_seed[i] = generate_random_array!(u64, 2, seed);
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

macro_rules! impl_smallint_array_hasher {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> Hasher<[$t; N]> for MSPHasher<[$t; N]> {
                type State = SmallArrayState<N>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    SmallArrayState::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = SmallArrayState::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &[$t; N]) -> u32 {
                    let bytes_len = N * core::mem::size_of::<$t>();
                    let bytes = unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, bytes_len) };
                    pair_multiply_shift_vector_u8(
                        bytes,
                        self.state.num_bits,
                        self.state.seed,
                        self.state.value_seed_as_slice(),
                    )
                }
            }

            impl<const N: usize> MSPHasher<[$t; N]> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> <Self as Hasher<[$t; N]>>::State {
                    SmallArrayState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = SmallArrayState::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<[$t; N]>>::State) -> Self { Self { state } }
                pub const fn num_buckets_const(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                pub const fn hash_const(&self, value: &[$t; N]) -> u32 {
                    use core::ptr::copy_nonoverlapping;
                    let bytes_len = N * core::mem::size_of::<$t>();
                    if bytes_len == 0 {
                        return crate::hashing::common::extract_bits_64::<{ u64::BITS }>(self.state.seed, self.state.num_bits);
                    }
                    if bytes_len <= 3 {
                        let mut padded = [0u8; 4];
                        unsafe { copy_nonoverlapping(value.as_ptr() as *const u8, padded.as_mut_ptr(), bytes_len) }
                        let v = u32::from_le_bytes(padded);
                        let seed_arr = [self.state.seed, self.state.value_seed_as_slice()[0]];
                        return multiply_shift(v, self.state.num_bits, &seed_arr);
                    }
                    if bytes_len == 4 {
                        let mut bytes = [0u8; 4];
                        unsafe { copy_nonoverlapping(value.as_ptr() as *const u8, bytes.as_mut_ptr(), 4) }
                        let v = u32::from_le_bytes(bytes);
                        let seed_arr = [self.state.seed, self.state.value_seed_as_slice()[0]];
                        return multiply_shift(v, self.state.num_bits, &seed_arr);
                    }
                    if bytes_len <= 7 {
                        let mut padded = [0u8; 8];
                        unsafe { copy_nonoverlapping(value.as_ptr() as *const u8, padded.as_mut_ptr(), bytes_len) }
                        let v = u64::from_le_bytes(padded);
                        let seeds = self.state.value_seed_as_slice();
                        let seed_arr = [self.state.seed, seeds[0], seeds[1]];
                        return pair_multiply_shift(v, self.state.num_bits, &seed_arr);
                    }
                    if bytes_len == 8 {
                        let mut bytes = [0u8; 8];
                        unsafe { copy_nonoverlapping(value.as_ptr() as *const u8, bytes.as_mut_ptr(), 8) }
                        let v = u64::from_le_bytes(bytes);
                        let seeds = self.state.value_seed_as_slice();
                        let seed_arr = [self.state.seed, seeds[0], seeds[1]];
                        return pair_multiply_shift(v, self.state.num_bits, &seed_arr);
                    }

                    // General case: chunk accumulation
                    let mut sum = self.state.seed;
                    let num_chunks = (bytes_len + 7) >> 3;
                    let mut i = 0;
                    let seeds = self.state.value_seed_as_slice();
                    while i < num_chunks {
                        let byte_idx = i << 3;
                        let remaining = bytes_len - byte_idx;
                        let to_copy = if remaining < 8 { remaining } else { 8 };
                        let mut bytes = [0u8; 8];
                        unsafe {
                            copy_nonoverlapping(
                                (value.as_ptr() as *const u8).add(byte_idx),
                                bytes.as_mut_ptr(),
                                to_copy,
                            );
                        }
                        let v = u64::from_le_bytes(bytes);
                        let low = v;
                        let high = v >> 32;
                        sum = sum.wrapping_add(
                            seeds[i * 2]
                                .wrapping_add(high)
                                .wrapping_mul(seeds[i * 2 + 1].wrapping_add(low)),
                        );
                        i += 1;
                    }
                    crate::hashing::common::extract_bits_64::<{ u64::BITS }>(sum, self.state.num_bits)
                }
            }
        )*
    };
}

impl_smallint_array_hasher!(u32, i32, u16, i16, u8, i8);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_smallint_array_hasher!(usize, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;

    generate_hasher_tests!(MSPHasher<u32>, u32, |rng: &mut ChaCha20Rng| rng
        .random::<u32>());
    generate_hasher_tests!(MSPHasher<i32>, i32, |rng: &mut ChaCha20Rng| rng
        .random::<i32>());
    generate_hasher_tests!(MSPHasher<u16>, u16, |rng: &mut ChaCha20Rng| rng
        .random::<u16>());
    generate_hasher_tests!(MSPHasher<i16>, i16, |rng: &mut ChaCha20Rng| rng
        .random::<i16>());
    generate_hasher_tests!(MSPHasher<u8>, u8, |rng: &mut ChaCha20Rng| rng
        .random::<u8>());
    generate_hasher_tests!(MSPHasher<i8>, i8, |rng: &mut ChaCha20Rng| rng
        .random::<i8>());
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        MSPHasher<usize>,
        usize,
        |rng: &mut ChaCha20Rng| rng.random::<u32>() as usize
    );
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        MSPHasher<isize>,
        isize,
        |rng: &mut ChaCha20Rng| rng.random::<i32>() as isize
    );
    #[cfg(target_pointer_width = "16")]
    generate_hasher_tests!(
        MSPHasher<usize>,
        usize,
        |rng: &mut ChaCha20Rng| rng.random::<u16>() as usize
    );
    #[cfg(target_pointer_width = "16")]
    generate_hasher_tests!(
        MSPHasher<isize>,
        isize,
        |rng: &mut ChaCha20Rng| rng.random::<i16>() as isize
    );

    generate_hasher_tests!(MSPHasher<[u32; 32]>, [u32; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[u32; 32]>());
    generate_hasher_tests!(MSPHasher<[i32; 32]>, [i32; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[i32; 32]>());
    generate_hasher_tests!(MSPHasher<[u16; 64]>, [u16; 64], |rng: &mut ChaCha20Rng| rng
        .random::<[u16; 64]>());
    generate_hasher_tests!(MSPHasher<[u8; 128]>, [u8; 128], |rng: &mut ChaCha20Rng| rng
        .random::<[u8; 128]>());
}
