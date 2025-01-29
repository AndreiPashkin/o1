//! Data generation utilities useful for testing and benchmarking.
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashSet;
use std::hash::Hash;

/// A type that has capacity to generate random values and slices of values.
pub trait Generate<R: Rng>: Sized {
    /// Parameters for data-generation specific for the type.
    type GenerateParams;

    /// Generate a random value of the type.
    fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self;

    /// Generate a slice of **unique** random values of the type.
    fn generate_many(rng: &mut R, params: &Self::GenerateParams, size: usize) -> Box<[Self]>
    where
        Self: Hash + Eq,
    {
        let mut seen = HashSet::new();
        while seen.len() < size {
            seen.insert(Self::generate(rng, params));
        }
        seen.into_iter().collect()
    }
}

pub struct NumParams<T> {
    min: T,
    max: T,
}

impl<T> NumParams<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

macro_rules! impl_generate_num {
    ($($type:ty),*) => {
        $(
            impl Default for NumParams<$type> {
                fn default() -> Self {
                    Self { min: <$type>::MIN, max: <$type>::MAX }
                }
            }

            impl<R: Rng> Generate<R> for $type {
                type GenerateParams = NumParams<$type>;

                fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self {
                    rng.gen_range(params.min..=params.max)
                }
            }

            impl<const SIZE: usize, R: Rng> Generate<R> for [$type; SIZE] {
                type GenerateParams = NumParams<$type>;

                fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self {
                    let mut array = [0; SIZE];
                    let mut seen = HashSet::new();
                    while seen.len() < SIZE {
                        seen.insert(rng.gen_range(params.min..=params.max));
                    }
                    for (i, value) in seen.into_iter().enumerate() {
                        array[i] = value;
                    }
                    array
                }
            }
        )*
    };
}

impl_generate_num!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

pub struct StringParams {
    min_length: usize,
    max_length: usize,
}

impl StringParams {
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl Default for StringParams {
    fn default() -> Self {
        Self {
            min_length: 1,
            max_length: 256,
        }
    }
}

impl<R: Rng> Generate<R> for String {
    type GenerateParams = StringParams;

    fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self {
        let length = rng.gen_range(params.min_length..=params.max_length);
        let iter = rng.sample_iter(&Alphanumeric);
        iter.take(length).map(char::from).collect()
    }
}
