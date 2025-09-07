//! Implements Hasher for unbounded strings and byte slices using the XXH3 hash function.

use super::core::XXH3Hasher;
use crate::hashing::common::{extract_bits_64, num_bits_for_buckets, num_buckets_for_bits};
use o1_core::Hasher;
use xxhash_rust::const_xxh3::xxh3_64_with_seed as xxh3_64_with_seed_const;
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[derive(Debug, Default, Clone, Copy)]
pub struct StringState {
    num_bits: u32,
    seed: u64,
}

impl StringState {
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        StringState { num_bits, seed }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        StringState { num_bits, seed }
    }
}

#[inline]
fn hash(state: &StringState, value: &[u8]) -> u32 {
    debug_assert!(
        (1..=32).contains(&state.num_bits),
        r#""num_bits" must be [1, 32]"#
    );

    let hash_value = xxh3_64_with_seed(value, state.seed);
    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

#[inline]
const fn hash_const(state: &StringState, value: &[u8]) -> u32 {
    debug_assert!(
        state.num_bits >= 1 && state.num_bits <= 32,
        r#""num_bits" must be [1, 32]"#
    );

    let hash_value = xxh3_64_with_seed_const(value, state.seed);
    extract_bits_64::<{ u64::BITS }>(hash_value, state.num_bits)
}

impl Hasher<&[u8]> for XXH3Hasher<&[u8]> {
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

impl XXH3Hasher<&[u8]> {
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

impl Hasher<String> for XXH3Hasher<String> {
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

impl<'a> Hasher<&'a str> for XXH3Hasher<&'a str> {
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

impl<'a> XXH3Hasher<&'a str> {
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
    use o1_testing::generate::Generate;
    use o1_testing::generate_hasher_tests;

    generate_hasher_tests!(XXH3Hasher<&str>, &'static str, |rng| {
        String::generate(
            rng,
            &<String as Generate<ChaCha20Rng>>::GenerateParams::default(),
        )
        .leak()
    });

    generate_hasher_tests!(XXH3Hasher<&[u8]>, &'static [u8], |rng| {
        String::generate(
            rng,
            &<String as Generate<ChaCha20Rng>>::GenerateParams::default(),
        )
        .into_bytes()
        .leak()
    });
}
