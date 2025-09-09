//! Implements [`Hasher`] for `Option<T>` where `T` is a primitive type using XXH3.
//!
//! The implementation delegates to the existing [`XXH3Hasher<T>`] and exposes
//! a matching const-time interface.

use super::core::XXH3Hasher;
use crate::hashing::common::{extract_bits_64, num_bits_for_buckets, num_buckets_for_bits};
use o1_core::Hasher;
use xxhash_rust::const_xxh3::xxh3_64_with_seed as xxh3_64_with_seed_const;
use xxhash_rust::xxh3::xxh3_64_with_seed;

/// State for hashing `Option<T>` values.
#[derive(Debug, Clone, Copy)]
pub struct OptionState<T>
where
    T: Eq,
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Copy + core::fmt::Debug + Default,
{
    seed: u64,
    inner: <XXH3Hasher<T> as Hasher<T>>::State,
    num_bits: u32,
}

impl<T> OptionState<T>
where
    T: Eq,
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Copy + core::fmt::Debug + Default,
{
    fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
        let inner = <XXH3Hasher<T> as Hasher<T>>::make_state(seed, num_buckets);
        let num_bits = num_bits_for_buckets(num_buckets);
        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#,
        );
        Self {
            seed: seed.wrapping_add(2000),
            inner,
            num_bits,
        }
    }
}

impl<T> Default for OptionState<T>
where
    T: Eq,
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Copy + core::fmt::Debug + Default,
{
    fn default() -> Self {
        Self {
            seed: 0,
            inner: <XXH3Hasher<T> as Hasher<T>>::State::default(),
            num_bits: 0,
        }
    }
}

macro_rules! impl_option_xxh3 {
    ($($t:ty),*) => {
        $(
        impl Hasher<Option<$t>> for XXH3Hasher<Option<$t>> {
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
                debug_assert!(
                    (1..=32).contains(&self.state.num_bits),
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<$t>::from_state(self.state.inner);
                        let hash = inner.hash(v);
                        buf[1..5].copy_from_slice(&hash.to_le_bytes());
                        5
                    }
                };
                let hash_value = xxh3_64_with_seed(&buf[..len], self.state.seed);
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }

        impl XXH3Hasher<Option<$t>> {
            pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<$t> {
                debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                let inner = XXH3Hasher::<$t>::make_state_const(seed, num_buckets);
                let num_bits = num_bits_for_buckets(num_buckets);
                debug_assert!(
                    num_bits >= 1 && num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#,
                );
                OptionState { seed: seed.wrapping_add(2000), inner, num_bits }
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
                debug_assert!(
                    self.state.num_bits >= 1 && self.state.num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<$t>::from_state_const(self.state.inner);
                        let hash = inner.hash_const(v);
                        let hash_bytes = hash.to_le_bytes();
                        buf[1] = hash_bytes[0];
                        buf[2] = hash_bytes[1];
                        buf[3] = hash_bytes[2];
                        buf[4] = hash_bytes[3];
                        5
                    }
                };
                let hash_value = match len {
                    1 => {
                        let slice = core::slice::from_ref(&buf[0]);
                        xxh3_64_with_seed_const(slice, self.state.seed)
                    }
                    _ => xxh3_64_with_seed_const(&buf, self.state.seed),
                };
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }
        )*
    };
}

macro_rules! impl_option_xxh3_array {
    ($($t:ty),*) => {
        $(
        impl<const N: usize> Hasher<Option<[$t; N]>> for XXH3Hasher<Option<[$t; N]>> {
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
                debug_assert!(
                    (1..=32).contains(&self.state.num_bits),
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<[$t; N]>::from_state(self.state.inner);
                        let hash = inner.hash(v);
                        buf[1..5].copy_from_slice(&hash.to_le_bytes());
                        5
                    }
                };
                let hash_value = xxh3_64_with_seed(&buf[..len], self.state.seed);
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }

        impl<const N: usize> XXH3Hasher<Option<[$t; N]>> {
            pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<[$t; N]> {
                debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                let inner = XXH3Hasher::<[$t; N]>::make_state_const(seed, num_buckets);
                let num_bits = num_bits_for_buckets(num_buckets);
                debug_assert!(
                    num_bits >= 1 && num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#,
                );
                OptionState { seed: seed.wrapping_add(2000), inner, num_bits }
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
                debug_assert!(
                    self.state.num_bits >= 1 && self.state.num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<[$t; N]>::from_state_const(self.state.inner);
                        let hash = inner.hash_const(v);
                        let hash_bytes = hash.to_le_bytes();
                        buf[1] = hash_bytes[0];
                        buf[2] = hash_bytes[1];
                        buf[3] = hash_bytes[2];
                        buf[4] = hash_bytes[3];
                        5
                    }
                };
                let hash_value = match len {
                    1 => {
                        let slice = core::slice::from_ref(&buf[0]);
                        xxh3_64_with_seed_const(slice, self.state.seed)
                    }
                    _ => xxh3_64_with_seed_const(&buf, self.state.seed),
                };
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }
        )*
    };
}

macro_rules! impl_option_xxh3_ref {
    ($($t:ty),*) => {
        $(
        impl<'a> Hasher<Option<$t>> for XXH3Hasher<Option<$t>> {
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
                debug_assert!(
                    (1..=32).contains(&self.state.num_bits),
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<$t>::from_state(self.state.inner);
                        let hash = inner.hash(v);
                        buf[1..5].copy_from_slice(&hash.to_le_bytes());
                        5
                    }
                };
                let hash_value = xxh3_64_with_seed(&buf[..len], self.state.seed);
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }

        impl<'a> XXH3Hasher<Option<$t>> {
            pub const fn make_state_const(seed: u64, num_buckets: u32) -> OptionState<$t> {
                debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);
                let inner = XXH3Hasher::<$t>::make_state_const(seed, num_buckets);
                let num_bits = num_bits_for_buckets(num_buckets);
                debug_assert!(
                    num_bits >= 1 && num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#,
                );
                OptionState { seed: seed.wrapping_add(2000), inner, num_bits }
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
                debug_assert!(
                    self.state.num_bits >= 1 && self.state.num_bits <= 32,
                    r#""num_bits" must be [1, 32]"#
                );
                let mut buf = [0u8; 5];
                let len = match value {
                    None => { buf[0] = 0; 1 }
                    Some(v) => {
                        buf[0] = 1;
                        let inner = XXH3Hasher::<$t>::from_state_const(self.state.inner);
                        let hash = inner.hash_const(v);
                        let hash_bytes = hash.to_le_bytes();
                        buf[1] = hash_bytes[0];
                        buf[2] = hash_bytes[1];
                        buf[3] = hash_bytes[2];
                        buf[4] = hash_bytes[3];
                        5
                    }
                };
                let hash_value = match len {
                    1 => {
                        let slice = core::slice::from_ref(&buf[0]);
                        xxh3_64_with_seed_const(slice, self.state.seed)
                    }
                    _ => xxh3_64_with_seed_const(&buf, self.state.seed),
                };
                extract_bits_64::<{ u64::BITS }>(hash_value, self.state.num_bits)
            }
        }
        )*
    };
}

impl_option_xxh3!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
#[cfg(target_pointer_width = "64")]
impl_option_xxh3!(usize, isize);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_option_xxh3!(usize, isize);

impl_option_xxh3_array!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);
#[cfg(target_pointer_width = "64")]
impl_option_xxh3_array!(usize, isize);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
impl_option_xxh3_array!(usize, isize);

impl_option_xxh3_ref!(&'a [u8], &'a str);

#[cfg(test)]
mod tests {
    use super::*;
    use o1_test::generate_hasher_tests;
    use rand::RngCore;

    generate_hasher_tests!(
        XXH3Hasher<Option<u32>>,
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
        XXH3Hasher<Option<u64>>,
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
        XXH3Hasher<Option<u128>>,
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
        XXH3Hasher<Option<&'static str>>,
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
        XXH3Hasher<Option<&'static [u8]>>,
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
        XXH3Hasher<Option<[u32; 32]>>,
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
