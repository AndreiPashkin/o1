//! Data generation utilities useful for testing and benchmarking.
use rand::distr::Alphanumeric;
use rand::Rng;
use std::collections::HashSet;
use std::hash::Hash;

/// Provides capabilities to generate random values of the implementer-type.
pub trait Generate<R: Rng>: Sized {
    /// Parameters for data-generation specific for the type.
    type GenerateParams: Default;

    /// Generates a single random value of the type.
    fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self;

    /// Generates a slice of **unique** random values of the type.
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

/// Parameters for [`Generate`] implementations that generate numeric values.
pub struct NumParams<T> {
    min: T,
    max: T,
}

impl<T> NumParams<T> {
    /// Creates a new instance of [`NumParams`] with the specified minimum and maximum values.
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

            impl<R: Rng> Generate<R> for $type
            {
                type GenerateParams = NumParams<$type>;

                fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self {
                    rng.random_range(params.min..=params.max)
                }
            }

            impl<const SIZE: usize, R: Rng> Generate<R> for [$type; SIZE] {
                type GenerateParams = NumParams<$type>;

                fn generate(rng: &mut R, params: &Self::GenerateParams) -> Self {
                    let mut array = [0; SIZE];
                    for i in 0..SIZE {
                        array[i] = <$type as Generate<R>>::generate(rng, params);
                    }
                    array
                }
            }
        )*
    };
}

impl_generate_num!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

/// Parameters for [`Generate`] implementations that generate strings.
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
        let length = rng.random_range(params.min_length..=params.max_length);
        let iter = rng.sample_iter(&Alphanumeric);
        iter.take(length).map(char::from).collect()
    }
}

/// Provides capabilities to derive new random values by introducing minimal random changes.
///
/// Useful for generating random values that are very similar with each other.
pub trait Jitter<R: Rng>: Sized {
    fn jitter(&self, rng: &mut R) -> Option<Self>;
}

macro_rules! impl_jitter_num {
    ($($type:ty),*) => {
        $(
            impl<R: Rng> Jitter<R> for $type
            {
                fn jitter(&self, rng: &mut R) -> Option<Self> {
                    let mut value = *self;
                    let bit_idx = rng.random_range(0..Self::BITS);
                    let bit_val = (value >> bit_idx & 1) == 0;  // Reads the bit and inverts it
                    if bit_val {
                        value |= 1 << bit_idx;
                    } else {
                        value &= !(1 << bit_idx);
                    }
                    Some(value)
                }
            }

            impl<const SIZE: usize, R: Rng> Jitter<R> for [$type; SIZE] {
                fn jitter(&self, rng: &mut R) -> Option<Self> {
                    let mut value = *self;
                    let idx = rng.random_range(0..SIZE);
                    if let Some(elem) = value[idx].jitter(rng) {
                        value[idx] = elem;
                    } else {
                        return None;
                    }
                    Some(value)
                }
            }
        )*
    };
}

impl_jitter_num!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

impl<R: Rng> Jitter<R> for String {
    fn jitter(&self, rng: &mut R) -> Option<Self> {
        if self.is_empty() {
            return None;
        }

        let mut chars: Vec<char> = self.chars().collect();

        // TODO: investigate if it makes sense to try different characters before increasing
        //       the number of bits to change.
        let char_idx = rng.random_range(0..chars.len());
        let original_char = chars[char_idx];

        let code_point = original_char as u32;

        for bits_to_change in 1..=u32::BITS {
            const MAX_TRIALS: usize = 99;

            for _ in 0..MAX_TRIALS {
                let mut new_code_point = code_point;
                let mut changed_bits = 0;

                while changed_bits < bits_to_change {
                    let bit_idx = rng.random_range(0..u32::BITS);
                    let bit_mask = 1u32 << bit_idx;

                    if (new_code_point ^ code_point) & bit_mask == 0 {
                        new_code_point ^= bit_mask;
                        changed_bits += 1;
                    }
                }

                if let Some(new_char) = std::char::from_u32(new_code_point) {
                    if new_char != original_char {
                        chars[char_idx] = new_char;
                        return Some(chars.iter().collect());
                    }
                }
            }
        }

        None
    }
}
