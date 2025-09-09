//! Implements [`Hasher`] for `Option<T>` where `T` is a primitive type.
//!
//! The implementation delegates to the existing [`MSPHasher<T>`].

use super::core::MSPHasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::multiply_shift::{multiply_shift, pair_multiply_shift};
use crate::utils::xorshift::generate_random_array;
use o1_core::Hasher;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

/// State for hashing `Option<T>` values.
#[derive(Debug, Clone, Copy)]
pub struct OptionState<T>
where
    T: Eq,
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Copy + Clone + core::fmt::Debug + Default,
{
    tag_seed: [u64; 2],
    combiner_seed: [u64; 3],
    inner: <MSPHasher<T> as Hasher<T>>::State,
    num_bits: u32,
}

impl<T> OptionState<T>
where
    T: Eq,
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Copy + Clone + core::fmt::Debug + Default,
{
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed.wrapping_add(1000));
        let tag_seed: [u64; 2] = rng.random();
        let combiner_seed: [u64; 3] = rng.random();
        let inner = <MSPHasher<T> as Hasher<T>>::make_state(seed.wrapping_add(2000), num_buckets);
        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        Self {
            tag_seed,
            combiner_seed,
            inner,
            num_bits,
        }
    }
}

impl<T> Default for OptionState<T>
where
    T: Eq,
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Copy + Clone + core::fmt::Debug + Default,
{
    fn default() -> Self {
        Self {
            tag_seed: [0; 2],
            combiner_seed: [0; 3],
            inner: <MSPHasher<T> as Hasher<T>>::State::default(),
            num_bits: 0,
        }
    }
}

macro_rules! impl_option_msp {
    ($($t:ty),*) => {
        $(
            impl Hasher<Option<$t>> for MSPHasher<Option<$t>> {
                type State = OptionState<$t>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    OptionState::<$t>::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = OptionState::<$t>::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &Option<$t>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<$t>::from_state(self.state.inner);
                            inner.hash(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }

            impl MSPHasher<Option<$t>> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<$t> {
                    debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                    let mut tag_seed: [u64; 2] =
                        generate_random_array!(u64, 2, seed.wrapping_add(1000));
                    tag_seed[0] |= 1;
                    let mut combiner_seed: [u64; 3] =
                        generate_random_array!(u64, 3, seed.wrapping_add(2000));
                    combiner_seed[0] |= 1;
                    let inner =
                        MSPHasher::<$t>::make_state_const(seed.wrapping_add(3000), num_buckets);
                    let num_bits = num_bits_for_buckets(num_buckets);

                    debug_assert!(
                        num_bits >= 1 && num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );

                    OptionState { tag_seed, combiner_seed, inner, num_bits }
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::make_state_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<Option<$t>>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &Option<$t>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<$t>::from_state_const(self.state.inner);
                            inner.hash_const(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }
        )*
    };
}

macro_rules! impl_option_msp_array {
    ($($t:ty),*) => {
        $(
            impl<const N: usize> Hasher<Option<[$t; N]>> for MSPHasher<Option<[$t; N]>>
            where
                [$t; N]: Eq,
                MSPHasher<[$t; N]>: Hasher<[$t; N]>,
                <MSPHasher<[$t; N]> as Hasher<[$t; N]>>::State:
                    Copy + Clone + core::fmt::Debug + Default,
            {
                type State = OptionState<[$t; N]>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    OptionState::<[$t; N]>::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = OptionState::<[$t; N]>::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &Option<[$t; N]>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<[$t; N]>::from_state(self.state.inner);
                            inner.hash(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }

            impl<const N: usize> MSPHasher<Option<[$t; N]>> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<[$t; N]> {
                    debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                    let mut tag_seed: [u64; 2] =
                        generate_random_array!(u64, 2, seed.wrapping_add(1000));
                    tag_seed[0] |= 1;
                    let mut combiner_seed: [u64; 3] =
                        generate_random_array!(u64, 3, seed.wrapping_add(2000));
                    combiner_seed[0] |= 1;
                    let inner = MSPHasher::<[$t; N]>::make_state_const(
                        seed.wrapping_add(3000),
                        num_buckets,
                    );
                    let num_bits = num_bits_for_buckets(num_buckets);

                    debug_assert!(
                        num_bits >= 1 && num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );

                    OptionState { tag_seed, combiner_seed, inner, num_bits }
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::make_state_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(
                    state: <Self as Hasher<Option<[$t; N]>>>::State,
                ) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &Option<[$t; N]>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<[$t; N]>::from_state_const(self.state.inner);
                            inner.hash_const(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }
        )*
    };
}

macro_rules! impl_option_msp_ref {
    ($($t:ty),*) => {
        $(
            impl<'a> Hasher<Option<$t>> for MSPHasher<Option<$t>>
            where
                MSPHasher<$t>: Hasher<$t>,
                <MSPHasher<$t> as Hasher<$t>>::State:
                    Copy + Clone + core::fmt::Debug + Default,
            {
                type State = OptionState<$t>;

                fn make_state(seed: u64, num_buckets: u32) -> Self::State {
                    OptionState::<$t>::from_seed(seed, num_buckets)
                }
                fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = OptionState::<$t>::from_seed(seed, num_buckets);
                    Self { state }
                }
                fn from_state(state: Self::State) -> Self { Self { state } }
                fn state(&self) -> &Self::State { &self.state }
                fn num_buckets(&self) -> u32 { num_buckets_for_bits(self.state.num_bits) }
                fn hash(&self, value: &Option<$t>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<$t>::from_state(self.state.inner);
                            inner.hash(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }

            impl<'a> MSPHasher<Option<$t>> {
                pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<$t> {
                    debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                    let mut tag_seed: [u64; 2] =
                        generate_random_array!(u64, 2, seed.wrapping_add(1000));
                    tag_seed[0] |= 1;
                    let mut combiner_seed: [u64; 3] =
                        generate_random_array!(u64, 3, seed.wrapping_add(2000));
                    combiner_seed[0] |= 1;
                    let inner =
                        MSPHasher::<$t>::make_state_const(seed.wrapping_add(3000), num_buckets);
                    let num_bits = num_bits_for_buckets(num_buckets);

                    debug_assert!(
                        num_bits >= 1 && num_bits <= 32,
                        r#""num_bits" must be [1, 32]"#
                    );

                    OptionState { tag_seed, combiner_seed, inner, num_bits }
                }
                pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
                    let state = Self::make_state_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state_const(state: <Self as Hasher<Option<$t>>>::State) -> Self {
                    Self { state }
                }
                pub const fn num_buckets_const(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash_const(&self, value: &Option<$t>) -> u32 {
                    let tag_hash = multiply_shift(
                        match value { None => 0u32, Some(_) => 1u32 },
                        self.state.num_bits,
                        &self.state.tag_seed,
                    );
                    let inner_hash = match value {
                        None => 0u32,
                        Some(v) => {
                            let inner = MSPHasher::<$t>::from_state_const(self.state.inner);
                            inner.hash_const(v)
                        }
                    };
                    let combined = ((tag_hash as u64) << 32) | inner_hash as u64;
                    pair_multiply_shift(combined, self.state.num_bits, &self.state.combiner_seed)
                }
            }
        )*
    };
}

impl_option_msp!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
#[cfg(target_pointer_width = "64")]
impl_option_msp!(usize, isize);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_option_msp!(usize, isize);

impl_option_msp_array!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
#[cfg(target_pointer_width = "64")]
impl_option_msp_array!(usize, isize);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_option_msp_array!(usize, isize);

impl_option_msp_ref!(&'a [u8], &'a str);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;
    use rand::RngCore;

    generate_hasher_tests!(
        MSPHasher<Option<u32>>,
        Option<u32>,
        |rng: &mut ChaCha20Rng| {
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(rng.random::<u32>())
            }
        }
    );

    generate_hasher_tests!(
        MSPHasher<Option<u64>>,
        Option<u64>,
        |rng: &mut ChaCha20Rng| {
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(rng.random::<u64>())
            }
        }
    );

    generate_hasher_tests!(
        MSPHasher<Option<u128>>,
        Option<u128>,
        |rng: &mut ChaCha20Rng| {
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(rng.random::<u128>())
            }
        }
    );

    generate_hasher_tests!(
        MSPHasher<Option<&'static str>>,
        Option<&'static str>,
        |rng: &mut ChaCha20Rng| {
            let options = ["alpha", "beta", "gamma", "delta"];
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(options[(rng.next_u32() as usize) % options.len()])
            }
        }
    );

    generate_hasher_tests!(
        MSPHasher<Option<&'static [u8]>>,
        Option<&'static [u8]>,
        |rng: &mut ChaCha20Rng| {
            let options: [&'static [u8]; 4] = [b"one", b"two", b"three", b"four"];
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(options[(rng.next_u32() as usize) % options.len()])
            }
        }
    );

    generate_hasher_tests!(
        MSPHasher<Option<[u32; 32]>>,
        Option<[u32; 32]>,
        |rng: &mut ChaCha20Rng| {
            let choice: u32 = rng.random();
            if choice % 10 < 3 {
                None
            } else {
                Some(rng.random::<[u32; 32]>())
            }
        }
    );
}
