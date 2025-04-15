//! Implements [`Hasher`] for all integer types of size equal to or smaller than 32-bits.
//! Casts non-`u32` inputs to `u32` and then uses [`multiply_shift`] hash function.
//!
//! # Notes
//!
//! - It is obviously not optimal to hash 8-bit and 16-bit integers like this - by upcasting them
//!   first, there should be specialized hash functions for these cases, so it's a TODO.

use super::core::MSPHasher;
use crate::core::Hasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::hashers::ConstMSPHasher;
use crate::hashing::multiply_shift::multiply_shift;
use crate::random::xorshift::generate_random_array;
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

/// Generates [`Hasher`] and implementations for all other "small" integer types.
///
/// The generated impls simply call the `u32` implementation and cast the input to `u32`.
macro_rules! impl_multiply_shift_small_int {
    ($($k:ty),*) => {
        $(
            impl Hasher<$k> for MSPHasher<$k> {
                type State = SmallIntState;

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
        )*
    };
}

impl_multiply_shift_small_int!(i32, u16, i16, u8, i8);

impl ConstMSPHasher<u32, MSPHasher<u32>> {
    pub const fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = SmallIntState::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state(state: SmallIntState) -> Self {
        Self { state }
    }
    pub const fn state(&self) -> &SmallIntState {
        &self.state
    }
    pub const fn num_buckets(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash(&self, value: &u32) -> u32 {
        hash(&self.state, *value)
    }
}

/// Generates const-hasher implementations for all other "small" integer types.
///
/// The generated impls simply call the `u32` implementation and cast the input to `u32`.
macro_rules! impl_multiply_shift_small_int_const {
    ($($k:ty),*) => {
        $(
            impl ConstMSPHasher<$k, MSPHasher<$k>> {
                pub const fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = SmallIntState::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state(state: SmallIntState) -> Self {
                    Self { state }
                }
                pub const fn state(&self) -> &SmallIntState {
                    &self.state
                }
                pub const fn num_buckets(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash(&self, value: &$k) -> u32 {
                    hash(&self.state, *value as u32)
                }
            }
        )*
    };
}

impl_multiply_shift_small_int_const!(i32, u16, i16, u8, i8);

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::testing::equivalence::hasher_equivalence;
    use compose_idents::compose_idents;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    macro_rules! impl_test_msp_hasher_equivalence {
        ($type:ty) => {
            compose_idents!(test_fn = [test_msp_hasher_equivalence_, $type]; {
                #[test]
                fn test_fn() {
                    hasher_equivalence!(
                        MSPHasher<$type>,
                        ConstMSPHasher<$type, MSPHasher<$type>>,
                        $type,
                        &mut ChaCha20Rng::from_os_rng(),
                        |rng| {
                            let value: $type = rng.random();
                            value
                        },
                        1 << 16,
                        999
                    );
                }
            });
        };
    }
    pub(crate) use impl_test_msp_hasher_equivalence;

    impl_test_msp_hasher_equivalence!(u32);
    impl_test_msp_hasher_equivalence!(i32);
    impl_test_msp_hasher_equivalence!(u16);
    impl_test_msp_hasher_equivalence!(i16);
    impl_test_msp_hasher_equivalence!(u8);
    impl_test_msp_hasher_equivalence!(i8);
}
