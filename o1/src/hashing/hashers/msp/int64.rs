//! Implements Hasher for u64 and i64 using [`pair_multiply_shift`] hash-function.

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::pair_multiply_shift;
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

impl Hasher<u64> for MSPHasher<u64> {
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
    fn hash(&self, value: &u64) -> u32 {
        hash(&self.state, *value)
    }
}

impl MSPHasher<u64> {
    pub const fn make_state_const(seed: u64, num_buckets: u32) -> U64State {
        U64State::from_seed_const(seed, num_buckets)
    }
    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        let state = U64State::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state_const(state: <Self as Hasher<u64>>::State) -> Self {
        Self { state }
    }
    pub const fn num_buckets_const(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash_const(&self, value: &u64) -> u32 {
        hash_const(&self.state, *value)
    }
}

impl Hasher<i64> for MSPHasher<i64> {
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
    fn hash(&self, value: &i64) -> u32 {
        hash(&self.state, *value as u64)
    }
}

impl MSPHasher<i64> {
    pub const fn make_state_const(seed: u64, num_buckets: u32) -> U64State {
        U64State::from_seed_const(seed, num_buckets)
    }
    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        let state = U64State::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state_const(state: <Self as Hasher<i64>>::State) -> Self {
        Self { state }
    }
    pub const fn num_buckets_const(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash_const(&self, value: &i64) -> u32 {
        hash_const(&self.state, *value as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::super::smallint::tests::impl_test_msp_hasher_equivalence;
    use super::*;
    use compose_idents::compose_idents;
    use o1_testing::equivalence::hasher_equivalence;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    impl_test_msp_hasher_equivalence!(u64);
    impl_test_msp_hasher_equivalence!(i64);
}
