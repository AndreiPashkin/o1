//! Implements Hasher for 32-bit and smaller integers using the XXH3 hash function.

use super::core::XXH3Hasher;
use crate::hashing::common::{extract_bits_64, num_bits_for_buckets, num_buckets_for_bits};
use o1_core::Hasher;
use xxhash_rust::const_xxh3::xxh3_64_with_seed as xxh3_64_with_seed_const;
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[derive(Debug, Default, Clone, Copy)]
pub struct SmallIntState {
    num_bits: u32,
    seed: u64,
}

impl SmallIntState {
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
fn hash(state: &SmallIntState, value: u32) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    let bytes = value.to_le_bytes();
    let hash_value = xxh3_64_with_seed(bytes.as_slice(), state.seed);

    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

#[inline]
const fn hash_const(state: &SmallIntState, value: u32) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );
    let bytes = value.to_le_bytes();
    let hash_value = xxh3_64_with_seed_const(bytes.as_slice(), state.seed);

    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

impl Hasher<u32> for XXH3Hasher<u32> {
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

impl XXH3Hasher<u32> {
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
        hash_const(&self.state, *value)
    }
}

/// Generates Hasher impls for other small integer types by upcasting to u32.
macro_rules! impl_xxh3_small_int {
    ($($k:ty),*) => {
        $(
            impl Hasher<$k> for XXH3Hasher<$k> {
                type State = SmallIntState;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    SmallIntState::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::State::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &$k) -> u32 {
                    hash(&self.state, (*value) as u32)
                }
            }

            impl XXH3Hasher<$k> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> SmallIntState {
                    SmallIntState::from_seed_const(seed, num_buckets)
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = SmallIntState::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<$k>>::State) -> Self { Self { state } }
                pub const fn num_buckets_const(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                pub const fn hash_const(&self, value: &$k) -> u32 {
                    hash_const(&self.state, (*value) as u32)
                }
            }
        )*
    };
}

impl_xxh3_small_int!(i32, u16, i16, u8, i8);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_xxh3_small_int!(usize, isize);

#[derive(Debug, Clone, Copy)]
pub struct SmallArrayState<const N: usize> {
    num_bits: u32,
    seed: u64,
}

impl<const N: usize> Default for SmallArrayState<N> {
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: 0,
        }
    }
}

impl<const N: usize> SmallArrayState<N> {
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

macro_rules! impl_smallint_array_hasher {
    ($(($t:ty, $S:expr)),*) => {
        $(
            impl<const N: usize> Hasher<[$t; N]> for XXH3Hasher<[$t; N]> {
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
                    debug_assert!(
                        (1..=32).contains(&self.state.num_bits),
                        r#""num_bits" must be [1, 32]"#
                    );
                    let bytes_len = N * $S;
                    let bytes = unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, bytes_len) };
                    let hash_value = xxh3_64_with_seed(bytes, self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }

            impl<const N: usize> XXH3Hasher<[$t; N]> {
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
                    debug_assert!(
                        self.state.num_bits >= 1 && self.state.num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );
                    let mut byte_array = [[0u8; $S]; N];
                    let mut i = 0;
                    while i < N {
                        byte_array[i] = value[i].to_le_bytes();
                        i += 1;
                    }
                    let bytes = unsafe { core::slice::from_raw_parts(byte_array.as_ptr() as *const u8, N * $S) };
                    let hash_value = xxh3_64_with_seed_const(bytes, self.state.seed);
                    extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
                }
            }
        )*
    };
}

impl_smallint_array_hasher!((u32, 4), (i32, 4), (u16, 2), (i16, 2), (u8, 1), (i8, 1));
#[cfg(target_pointer_width = "32")]
impl_smallint_array_hasher!((usize, 4), (isize, 4));
#[cfg(target_pointer_width = "16")]
impl_smallint_array_hasher!((usize, 2), (isize, 2));

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;

    generate_hasher_tests!(XXH3Hasher<u32>, u32, |rng: &mut ChaCha20Rng| rng
        .random::<u32>());
    generate_hasher_tests!(XXH3Hasher<i32>, i32, |rng: &mut ChaCha20Rng| rng
        .random::<i32>());
    generate_hasher_tests!(XXH3Hasher<u16>, u16, |rng: &mut ChaCha20Rng| rng
        .random::<u16>());
    generate_hasher_tests!(XXH3Hasher<i16>, i16, |rng: &mut ChaCha20Rng| rng
        .random::<i16>());
    generate_hasher_tests!(XXH3Hasher<u8>, u8, |rng: &mut ChaCha20Rng| rng
        .random::<u8>());
    generate_hasher_tests!(XXH3Hasher<i8>, i8, |rng: &mut ChaCha20Rng| rng
        .random::<i8>());
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        XXH3Hasher<usize>,
        usize,
        |rng: &mut ChaCha20Rng| rng.random::<u32>() as usize
    );
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        XXH3Hasher<isize>,
        isize,
        |rng: &mut ChaCha20Rng| rng.random::<i32>() as isize
    );

    generate_hasher_tests!(XXH3Hasher<[u32; 32]>, [u32; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[u32; 32]>()
    });
    generate_hasher_tests!(XXH3Hasher<[i32; 32]>, [i32; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[i32; 32]>()
    });
    generate_hasher_tests!(XXH3Hasher<[u16; 32]>, [u16; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[u16; 32]>()
    });
    generate_hasher_tests!(XXH3Hasher<[i16; 32]>, [i16; 32], |rng: &mut ChaCha20Rng| {
        rng.random::<[i16; 32]>()
    });
    generate_hasher_tests!(XXH3Hasher<[u8; 32]>, [u8; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[u8; 32]>());
    generate_hasher_tests!(XXH3Hasher<[i8; 32]>, [i8; 32], |rng: &mut ChaCha20Rng| rng
        .random::<[i8; 32]>());
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        XXH3Hasher<[usize; 32]>,
        [usize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[u32; 32]>() as *const [u32; 32] as *const [usize; 32])
        }
    );
    #[cfg(target_pointer_width = "32")]
    generate_hasher_tests!(
        XXH3Hasher<[isize; 32]>,
        [isize; 32],
        |rng: &mut ChaCha20Rng| unsafe {
            *(&rng.random::<[i32; 32]>() as *const [i32; 32] as *const [isize; 32])
        }
    );
}
