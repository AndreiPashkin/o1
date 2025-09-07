//! Implements Hasher for unbounded strings represented as `&[u8]`.
//!
//! # Notes
//!
//! Internally it uses the [`polynomial`] hash function.

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::{
    pair_multiply_shift_vector_u8, pair_multiply_shift_vector_u8_const,
};
use crate::hashing::polynomial::{polynomial, polynomial_const, PolynomialSeed};
use crate::utils::xorshift::{generate_random, generate_random_array};
use o1_core::Hasher;
use rand::{Rng, RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

const N: u32 = 89;
const P: u128 = 2_u128.pow(N) - 1;
const MAX_STR_VECTOR_LEN: usize = 256;
const MUL_SHIFT_SEED_SIZE: usize = MAX_STR_VECTOR_LEN.div_ceil(4);

#[derive(Debug, Clone, Copy)]
pub struct StringState {
    num_bits: u32,
    mul_shift_seed: u64,
    mul_shift_value_seed: [u64; MUL_SHIFT_SEED_SIZE],
    polynomial_seed: PolynomialSeed,
}

impl Default for StringState {
    fn default() -> Self {
        let mut polynomial_seed_value = [0; 132];
        polynomial_seed_value[0] |= 1;
        let polynomial_seed = PolynomialSeed::from_slice(&polynomial_seed_value);
        Self {
            num_bits: 0,
            mul_shift_seed: 0,
            mul_shift_value_seed: [0; MUL_SHIFT_SEED_SIZE],
            polynomial_seed,
        }
    }
}

impl StringState {
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);

        let mut polynomial_seed_value = [0_u64; 132];
        polynomial_seed_value[0] = rng.random_range(1..P) as u64;
        polynomial_seed_value[1..].fill_with(|| rng.random_range(0..P) as u64);
        let polynomial_seed = PolynomialSeed::from_slice(&polynomial_seed_value);

        let mul_shift_seed = rng.next_u64();
        let mut mul_shift_value_seed = [0_u64; MUL_SHIFT_SEED_SIZE];
        mul_shift_value_seed.fill_with(|| rng.next_u64());

        StringState {
            num_bits,
            mul_shift_seed,
            mul_shift_value_seed,
            polynomial_seed,
        }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        let mul_shift_seed = generate_random!(u64, seed);
        let mul_shift_value_seed = generate_random_array!(u64, MUL_SHIFT_SEED_SIZE, seed);

        let mut polynomial_seed_value = generate_random_array!(u64, 132, seed.wrapping_add(1));
        polynomial_seed_value[0] |= 1;

        let mut i = 0;
        while i < polynomial_seed_value.len() {
            polynomial_seed_value[i] &= (1u64 << (N - 64)) - 1; // Clamp by P
            i += 1;
        }

        let polynomial_seed = PolynomialSeed::from_slice(&polynomial_seed_value);

        StringState {
            num_bits,
            mul_shift_seed,
            mul_shift_value_seed,
            polynomial_seed,
        }
    }
}

#[inline]
fn hash(state: &StringState, value: &[u8]) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    if value.len() <= MAX_STR_VECTOR_LEN {
        pair_multiply_shift_vector_u8(
            value,
            state.num_bits,
            state.mul_shift_seed,
            &state.mul_shift_value_seed,
        )
    } else {
        polynomial(value, state.num_bits, &state.polynomial_seed)
    }
}

#[inline]
const fn hash_const(state: &StringState, value: &[u8]) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );
    if value.len() <= MAX_STR_VECTOR_LEN {
        pair_multiply_shift_vector_u8_const(
            value,
            state.num_bits,
            state.mul_shift_seed,
            &state.mul_shift_value_seed,
        )
    } else {
        polynomial_const(value, state.num_bits, &state.polynomial_seed)
    }
}

impl Hasher<&[u8]> for MSPHasher<&[u8]> {
    type State = StringState;

    fn make_state(seed: u64, num_buckets: u32) -> Self::State {
        StringState::from_seed(seed, num_buckets)
    }
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = StringState::from_seed(seed, num_buckets);
        Self { state }
    }
    fn from_state(state: StringState) -> Self {
        Self { state }
    }
    fn state(&self) -> &Self::State {
        &self.state
    }
    fn num_buckets(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    fn hash(&self, value: &&[u8]) -> u32 {
        hash(&self.state, value)
    }
}

impl MSPHasher<&[u8]> {
    pub const fn make_state_const(seed: u64, num_buckets: u32) -> StringState {
        StringState::from_seed_const(seed, num_buckets)
    }
    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        let state = StringState::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state_const(state: <Self as Hasher<&[u8]>>::State) -> Self {
        Self { state }
    }
    pub const fn num_buckets_const(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash_const(&self, value: &&[u8]) -> u32 {
        hash_const(&self.state, value)
    }
}

impl Hasher<String> for MSPHasher<String> {
    type State = StringState;

    fn make_state(seed: u64, num_buckets: u32) -> Self::State {
        StringState::from_seed(seed, num_buckets)
    }
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = StringState::from_seed(seed, num_buckets);
        Self { state }
    }
    fn from_state(state: StringState) -> Self {
        Self { state }
    }
    fn state(&self) -> &Self::State {
        &self.state
    }
    fn num_buckets(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    fn hash(&self, value: &String) -> u32 {
        hash(&self.state, value.as_bytes())
    }
}

impl<'a> Hasher<&'a str> for MSPHasher<&'a str> {
    type State = StringState;

    fn make_state(seed: u64, num_buckets: u32) -> Self::State {
        StringState::from_seed(seed, num_buckets)
    }
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = StringState::from_seed(seed, num_buckets);
        Self { state }
    }
    fn from_state(state: StringState) -> Self {
        Self { state }
    }
    fn state(&self) -> &Self::State {
        &self.state
    }
    fn num_buckets(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    fn hash(&self, value: &&str) -> u32 {
        hash(&self.state, value.as_bytes())
    }
}

impl<'a> MSPHasher<&'a str> {
    pub const fn make_state_const(seed: u64, num_buckets: u32) -> StringState {
        StringState::from_seed_const(seed, num_buckets)
    }
    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        let state = StringState::from_seed_const(seed, num_buckets);
        Self { state }
    }
    pub const fn from_state_const(state: <Self as Hasher<&'a str>>::State) -> Self {
        Self { state }
    }
    pub const fn num_buckets_const(&self) -> u32 {
        num_buckets_for_bits(self.state.num_bits)
    }
    pub const fn hash_const(&self, value: &&str) -> u32 {
        hash_const(&self.state, value.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate::Generate;
    use o1_test::generate_hasher_tests;

    generate_hasher_tests!(MSPHasher<&str>, &'static str, |rng| {
        String::generate(
            rng,
            &<String as Generate<ChaCha20Rng>>::GenerateParams::default(),
        )
        .leak()
    });

    generate_hasher_tests!(MSPHasher<&[u8]>, &'static [u8], |rng| {
        String::generate(
            rng,
            &<String as Generate<ChaCha20Rng>>::GenerateParams::default(),
        )
        .into_bytes()
        .leak()
    });
}
