//! Implements [`Hasher`] for the integer size larger than 64-bit.
//!
//! # Notes
//!
//! Internally it treats big integers as vectors uses the [`multiply_shift_u8`] hash function.

use super::core::MSPHasher;
use crate::core::Hasher;
use crate::hashing::common::{num_bits_for_buckets, num_buckets_for_bits};
use crate::hashing::hashers::ConstMSPHasher;
use crate::hashing::multiply_shift::{
    pair_multiply_shift_vector_u8, pair_multiply_shift_vector_u8_const,
};
use crate::random::xorshift::generate_random_array;
use core::mem::size_of;
use rand::Rng;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

const MAX_SEED_SIZE: usize = {
    let u128_size = size_of::<u128>();
    let usize_size = size_of::<usize>();

    let max_size = if u128_size > usize_size {
        u128_size
    } else {
        usize_size
    };
    max_size.div_ceil(4) + 1
};

#[derive(Debug, Clone, Copy)]
pub struct BigIntState<T>
where
    T: Clone + Default,
{
    pub(super) num_bits: u32,
    seed: [u64; MAX_SEED_SIZE],
    seed_len: usize,
    _type: std::marker::PhantomData<T>,
}

impl<T> Default for BigIntState<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            num_bits: 0,
            seed: [0; MAX_SEED_SIZE],
            seed_len: size_of::<T>().div_ceil(4) + 1,
            _type: std::marker::PhantomData,
        }
    }
}

impl<T> BigIntState<T>
where
    T: Default + Clone,
{
    pub fn from_seed(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let seed_len = size_of::<T>().div_ceil(4) + 1;
        let mut seed_arr = [0_u64; MAX_SEED_SIZE];
        seed_arr[0..seed_len].fill_with(|| rng.random());

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            (1..=32).contains(&num_bits),
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed: seed_arr,
            seed_len,
            _type: std::marker::PhantomData,
        }
    }

    pub const fn from_seed_const(seed: u64, num_buckets: u32) -> Self {
        debug_assert!(num_buckets > 0, r#""num_buckets" must be greater than 0"#);

        let seed_len = size_of::<T>().div_ceil(4) + 1;
        let mut seed_arr = generate_random_array!(u64, MAX_SEED_SIZE, seed);

        let mut i = seed_len;
        while i < MAX_SEED_SIZE {
            seed_arr[i] = 0;
            i += 1;
        }

        let num_bits = num_bits_for_buckets(num_buckets);

        debug_assert!(
            num_bits >= 1 && num_bits <= 32,
            r#""num_bits" must be [1, 32]"#
        );

        BigIntState {
            num_bits,
            seed: seed_arr,
            seed_len,
            _type: std::marker::PhantomData,
        }
    }

    #[inline]
    pub const fn seed(&self) -> &[u64] {
        unsafe { core::slice::from_raw_parts(self.seed.as_ptr(), self.seed_len) }
    }
}

/// Generates [`Hasher`] and implementations for "big" integer types.
macro_rules! impl_multiply_shift_big_int {
    ($($T:ty),*) => {
        $(
            impl Hasher<$T> for MSPHasher<$T> {
                type State = BigIntState<$T>;

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
                fn hash(&self, value: &$T) -> u32 {
                    pair_multiply_shift_vector_u8(
                        &value.to_le_bytes(),
                        self.state.num_bits,
                        self.state.seed(),
                    )
                }
            }

            impl ConstMSPHasher<$T, MSPHasher<$T>> {
                pub const fn from_seed(seed: u64, num_buckets: u32) -> Self {
                    let state = BigIntState::<$T>::from_seed_const(seed, num_buckets);
                    Self { state }
                }
                pub const fn from_state(state: BigIntState<$T>) -> Self {
                    Self { state }
                }
                pub const fn state(&self) -> &BigIntState<$T> {
                    &self.state
                }
                pub const fn num_buckets(&self) -> u32 {
                    num_buckets_for_bits(self.state.num_bits)
                }
                pub const fn hash(&self, value: &$T) -> u32 {
                    let bytes = value.to_le_bytes();
                    pair_multiply_shift_vector_u8_const(
                        &bytes,
                        self.state.num_bits,
                        &self.state.seed(),
                    )
                }
            }
        )*
    };
}

impl_multiply_shift_big_int!(u128, i128, usize, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::equivalence::hasher_equivalence;
    use compose_idents::compose_idents;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    macro_rules! impl_test_msp_hasher_equivalence_bigint {
        ($type:ty, $gen_expr:expr) => {
            compose_idents!(test_fn = [test_msp_hasher_equivalence_, $type]; {
                #[test]
                fn test_fn() {
                    hasher_equivalence!(
                        MSPHasher<$type>,
                        ConstMSPHasher<$type, MSPHasher<$type>>,
                        $type,
                        &mut ChaCha20Rng::from_os_rng(),
                        |rng| $gen_expr(rng),
                        1 << 16,
                        999
                    );
                }
            });
        };
    }

    impl_test_msp_hasher_equivalence_bigint!(u128, |rng: &mut ChaCha20Rng| rng.random::<u128>());
    impl_test_msp_hasher_equivalence_bigint!(i128, |rng: &mut ChaCha20Rng| rng.random::<i128>());
    impl_test_msp_hasher_equivalence_bigint!(usize, |rng: &mut ChaCha20Rng| rng.random::<u64>()
        as usize);
    impl_test_msp_hasher_equivalence_bigint!(isize, |rng: &mut ChaCha20Rng| rng.random::<i64>()
        as isize);
}
