//! Implements Hasher for unbounded strings represented as `&[u8]`.
//!
//! # Notes
//!
//! Internally it uses the [`polynomial`] hash function.

use super::core::MSPHasher;
use crate::core::Hasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::pair_multiply_shift_vector_u8;
use crate::hashing::polynomial::{polynomial, PolynomialSeed};
use rand::{Rng, RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

#[derive(Debug, Default, Clone)]
pub struct StringState {
    num_bits: u32,
    mul_shift_seed: Box<[u64]>,
    polynomial_seed: PolynomialSeed,
}

const N: u32 = 89;
const P: u128 = 2_u128.pow(N) - 1;
const MAX_STR_VECTOR_LEN: usize = 256;

#[inline]
fn make_state(seed: u64, num_buckets: u32) -> StringState {
    debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

    let num_bits = num_bits_for_buckets(num_buckets);

    debug_assert!(
        (1..=32).contains(&num_bits),
        r#""num_bits" must be [1, 32]"#
    );

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let mut polynomial_seed = [0_u64; 132];
    polynomial_seed[0] = rng.random_range(1..P) as u64;
    polynomial_seed[1..].fill_with(|| rng.random_range(0..P) as u64);

    let mul_shift_seed: Box<[u64]> = (0..MAX_STR_VECTOR_LEN.div_ceil(4) + 1)
        .map(|_| rng.next_u64())
        .collect::<Vec<u64>>()
        .into_boxed_slice();

    StringState {
        num_bits,
        mul_shift_seed,
        polynomial_seed: polynomial_seed.into(),
    }
}

#[inline]
fn hash(state: &StringState, value: &[u8]) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );
    if value.len() <= MAX_STR_VECTOR_LEN {
        pair_multiply_shift_vector_u8(value, state.num_bits, &state.mul_shift_seed)
    } else {
        polynomial(value, state.num_bits, &state.polynomial_seed)
    }
}

impl Hasher<&[u8]> for MSPHasher<&[u8]> {
    type State = StringState;

    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = make_state(seed, num_buckets);
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

impl Hasher<String> for MSPHasher<String> {
    type State = StringState;

    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = make_state(seed, num_buckets);
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

impl Hasher<&str> for MSPHasher<&str> {
    type State = StringState;

    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        let state = make_state(seed, num_buckets);
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
