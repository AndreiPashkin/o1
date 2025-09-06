//! Provides [`BitArray`] - a bit array that works at compile-time.
//!
//! Use the convenient [`bit_array!`] macro as a factory - it allows to specify array's length in
//! bits.

/// Marker trait for unsigned integer types used for storing bits.
///
/// # Notes
///
/// - Since traits do not support `const fn` methods - used only for associated constants.
pub trait BitStore: Copy + Default {
    /// The number of bits in this storage type.
    const BITS: usize;
    /// Bit mask for performing modulo operations (BITS - 1)
    const BITS_MASK: usize;
    /// Log2 of the number of bits (for shift operations)
    const BITS_LOG2: usize;
    /// A value with all bits set to 1.
    const ALL_ONES: Self;
}

impl BitStore for u8 {
    const BITS: usize = u8::BITS as usize;
    const BITS_MASK: usize = (u8::BITS - 1) as usize;
    const BITS_LOG2: usize = u8::BITS.ilog2() as usize;
    const ALL_ONES: Self = ((1u16 << u8::BITS) - 1) as u8;
}

impl BitStore for u16 {
    const BITS: usize = u16::BITS as usize;
    const BITS_MASK: usize = (u16::BITS - 1) as usize;
    const BITS_LOG2: usize = u16::BITS.ilog2() as usize;
    const ALL_ONES: Self = ((1u32 << u16::BITS) - 1) as u16;
}

impl BitStore for u32 {
    const BITS: usize = u32::BITS as usize;
    const BITS_MASK: usize = (u32::BITS - 1) as usize;
    const BITS_LOG2: usize = u32::BITS.ilog2() as usize;
    const ALL_ONES: Self = ((1u64 << u32::BITS) - 1) as u32;
}

impl BitStore for u64 {
    const BITS: usize = u64::BITS as usize;
    const BITS_MASK: usize = (u64::BITS - 1) as usize;
    const BITS_LOG2: usize = u64::BITS.ilog2() as usize;
    const ALL_ONES: Self = !0;
}

impl BitStore for u128 {
    const BITS: usize = u128::BITS as usize;
    const BITS_MASK: usize = (u128::BITS - 1) as usize;
    const BITS_LOG2: usize = u128::BITS.ilog2() as usize;
    const ALL_ONES: Self = !0;
}

/// A wrapper over a single unsigned integer that provides bit manipulation operations.
///
/// This provides a convenient API for working with bits in a single unsigned integer value,
/// with all operations being available in const contexts.
///
/// # Examples
///
/// ```
/// use o1::utils::bit_array::Bits;
///
/// let mut bits = Bits::<u8>::new();
/// bits.set(0);
/// bits.set(5);
///
/// assert!(bits.get(0).unwrap());
/// assert!(bits.get(5).unwrap());
/// assert!(!bits.get(1).unwrap());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bits<T: BitStore> {
    value: T,
}

/// Iterator over the indices of set bits in a Bits wrapper.
pub struct BitsOnesIter<T: BitStore> {
    /// The remaining value with unprocessed bits
    remaining: T,
}

/// Compile-time iterator over the indices of set bits in a Bits wrapper.
///
/// Mimics the interface of [`Iterator`] without implementing it.
pub struct BitsOnesConstIter<T: BitStore> {
    /// The remaining value with unprocessed bits
    remaining: T,
}

/// Generates a [`Bits`] implementation for the specified type.
macro_rules! impl_bits {
    ($type:ty) => {
        impl Bits<$type> {
            /// Creates a new Bits with all bits set to 0.
            pub const fn new() -> Self {
                Self { value: 0 }
            }

            /// Creates a new Bits with the specified value.
            pub const fn from_value(value: $type) -> Self {
                Self { value }
            }

            /// Returns the raw underlying value.
            pub const fn value(&self) -> $type {
                self.value
            }

            /// Returns the total number of bits.
            pub const fn len(&self) -> usize {
                <$type as BitStore>::BITS
            }

            /// Returns true if the Bits instance is empty.
            pub const fn is_empty(&self) -> bool {
                false
            }

            /// Gets the value of the bit at the specified index.
            pub const fn get(&self, index: usize) -> Option<bool> {
                if index >= self.len() {
                    return None;
                }

                let mask = 1 as $type << index as u32;
                Some((self.value & mask) != 0)
            }

            /// Sets the bit at the specified index to 1.
            pub const fn set(&mut self, index: usize) {
                if index >= self.len() {
                    return;
                }

                let mask = 1 as $type << index as u32;
                self.value |= mask;
            }

            /// Clears the bit at the specified index (sets to 0).
            pub const fn clear(&mut self, index: usize) {
                if index >= self.len() {
                    return;
                }

                let mask = !(1 as $type << index as u32);
                self.value &= mask;
            }

            /// Sets all bits to 0.
            pub const fn clear_all(&mut self) {
                self.value = 0;
            }

            /// Sets all bits to 1.
            pub const fn set_all(&mut self) {
                self.value = <$type as BitStore>::ALL_ONES;
            }

            /// Counts the number of bits set to 1.
            pub const fn count_ones(&self) -> usize {
                self.value.count_ones() as usize
            }

            /// Returns an iterator over the indices of all bits set to 1.
            pub fn iter_ones(&self) -> BitsOnesIter<$type> {
                BitsOnesIter {
                    remaining: self.value,
                }
            }

            /// Returns a const iterator over the indices of all bits set to 1.
            pub const fn iter_ones_const(&self) -> BitsOnesConstIter<$type> {
                BitsOnesConstIter {
                    remaining: self.value,
                }
            }
        }

        impl Iterator for BitsOnesIter<$type> {
            type Item = usize;

            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }

                let trailing_zeros = self.remaining.trailing_zeros() as usize;

                // Clear the bit we just found
                self.remaining &= !(1 as $type << trailing_zeros as u32);

                Some(trailing_zeros)
            }
        }

        impl BitsOnesConstIter<$type> {
            pub const fn next(&mut self) -> Option<usize> {
                if self.remaining == 0 {
                    return None;
                }

                let trailing_zeros = self.remaining.trailing_zeros() as usize;

                // Clear the bit we just found
                self.remaining &= !(1 as $type << trailing_zeros as u32);

                Some(trailing_zeros)
            }
        }
    }
}

impl_bits!(u8);
impl_bits!(u16);
impl_bits!(u32);
impl_bits!(u64);
impl_bits!(u128);

/// A BitArray with compile-time size that supports const operations.
///
/// This implementation allows bit operations at compile time, making it useful
/// for const contexts. It uses a generic storage type T and a fixed number of
/// buckets N.
///
/// Type Parameters:
/// - `T`: The storage type for each bucket (u8, u16, u32, u64, or u128)
/// - `N`: The number of buckets of type T
///
/// # Examples
///
/// ```
/// use o1::utils::bit_array::BitArray;
///
/// let mut arr = BitArray::<u8, 2>::new();
/// arr.set(0);
/// arr.set(5);
///
/// assert!(arr.get(0).unwrap() == true);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitArray<T: BitStore, const N: usize> {
    buckets: [Bits<T>; N],
}

/// Iterator over the indices of set bits in a BitArray.
pub struct BitArrayOnesIter<'a, T: BitStore, const N: usize> {
    bit_array: &'a BitArray<T, N>,
    /// Index of the current bucket
    bucket_idx: usize,
    /// Iterator for the current bucket
    bucket_iter: Option<BitsOnesIter<T>>,
    /// Maximum bucket index
    max_idx: usize,
}

/// Compile-time iterator over the indices of set bits
///
/// Mimics the interface of [`Iterator`] without implementing it.
pub struct BitArrayOnesConstIter<T: BitStore, const N: usize> {
    bit_array: BitArray<T, N>,
    /// Index of the current bucket
    bucket_idx: usize,
    /// Content of the current bucket with iterated ones being unset
    bucket: T,
    /// Maximum bucket index
    max_idx: usize,
}

/// Generates a [`BitArray`] implementation for the specified type.
macro_rules! impl_bit_array {
    ($type:ty) => {
        impl<const N: usize> BitArray<$type, N> {
            const fn index(&self, bit_idx: usize) -> (usize, usize) {
                let bucket_idx = bit_idx >> <$type as BitStore>::BITS_LOG2;
                let bit_idx = bit_idx & <$type as BitStore>::BITS_MASK;

                (bucket_idx, bit_idx)
            }

            /// Creates a new BitArray with all bits set to 0.
            pub const fn new() -> Self {
                Self { buckets: [Bits::<$type>::new(); N] }
            }

            /// Returns the total number of bits in the BitArray.
            pub const fn len(&self) -> usize {
                N * <$type as BitStore>::BITS
            }

            /// Returns true if the BitArray is empty.
            pub const fn is_empty(&self) -> bool {
                N == 0
            }

            /// Gets the value of the bit at the specified index.
            pub const fn get(&self, index: usize) -> Option<bool> {
                if index >= self.len() {
                    return None;
                }

                let (bucket_idx, bit_idx) = self.index(index);
                self.buckets[bucket_idx].get(bit_idx)
            }

            /// Sets the bit at the specified index to 1.
            pub const fn set(&mut self, index: usize) {
                if index >= self.len() {
                    return;
                }

                let (bucket_idx, bit_idx) = self.index(index);
                self.buckets[bucket_idx].set(bit_idx);
            }

            /// Clears the bit at the specified index (sets to 0).
            pub const fn clear(&mut self, index: usize) {
                if index >= self.len() {
                    return;
                }

                let (bucket_idx, bit_idx) = self.index(index);
                self.buckets[bucket_idx].clear(bit_idx);
            }

            /// Sets all bits to 0.
            pub const fn clear_all(&mut self) {
                let mut i = 0;
                while i < N {
                    self.buckets[i].clear_all();
                    i += 1;
                }
            }

            /// Sets all bits to 1.
            pub const fn set_all(&mut self) {
                let mut i = 0;
                while i < N {
                    self.buckets[i].set_all();
                    i += 1;
                }
            }

            /// Counts the number of bits set to 1.
            pub const fn count_ones(&self) -> usize {
                let mut count = 0;
                let mut i = 0;
                while i < N {
                    count += self.buckets[i].count_ones();
                    i += 1;
                }
                count
            }

            /// Returns an iterator over the indices of all bits set to 1.
            pub fn iter_ones(&self) -> BitArrayOnesIter<'_, $type, N> {
                BitArrayOnesIter {
                    bit_array: self,
                    bucket_idx: 0,
                    bucket_iter: if N > 0 { Some(self.buckets[0].iter_ones()) } else { None },
                    max_idx: self.len(),
                }
            }

            /// Returns a const iterator over the indices of all bits set to 1.
            pub const fn iter_ones_const(&self) -> BitArrayOnesConstIter<$type, N> {
                BitArrayOnesConstIter {
                    bit_array: *self,
                    bucket_idx: 0,
                    bucket: if N > 0 { self.buckets[0].value() } else { 0 },
                    max_idx: self.len(),
                }
            }
        }

        impl<const N: usize> Default for BitArray<$type, N> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<'a, const N: usize> Iterator for BitArrayOnesIter<'a, $type, N> {
            type Item = usize;

            fn next(&mut self) -> Option<Self::Item> {
                while self.bucket_idx < N {
                    if let Some(ref mut iter) = self.bucket_iter {
                        if let Some(bit_idx) = iter.next() {
                            let index = self.bucket_idx * <$type as BitStore>::BITS + bit_idx;
                            return Some(index);
                        }
                    }

                    self.bucket_idx += 1;
                    if self.bucket_idx < N {
                        self.bucket_iter = Some(self.bit_array.buckets[self.bucket_idx].iter_ones());
                    } else {
                        self.bucket_iter = None;
                    }
                }

                None
            }
        }

        impl<const N: usize> BitArrayOnesConstIter<$type, N> {
            pub const fn next(&mut self) -> Option<usize> {
                while self.bucket_idx < N {
                    if self.bucket != 0 {
                        let trailing_zeros = self.bucket.trailing_zeros() as usize;

                        let index = self.bucket_idx * <$type as BitStore>::BITS + trailing_zeros;

                        self.bucket &= !(1 as $type << trailing_zeros as u32);

                        return Some(index);
                    }

                    self.bucket_idx += 1;

                    if self.bucket_idx < N {
                        self.bucket = self.bit_array.buckets[self.bucket_idx].value();
                    }
                }

                None
            }
        }
    }
}

impl_bit_array!(u8);
impl_bit_array!(u16);
impl_bit_array!(u32);
impl_bit_array!(u64);
impl_bit_array!(u128);

/// Creates a [`BitArray`].
///
/// # Examples
/// ```rust
/// use o1::utils::bit_array::bit_array;
///
/// let arr = bit_array!(10, u8);
///
/// assert_eq!(arr.len(), 16);
/// assert_eq!(arr.get(1).unwrap(), false);
/// assert_eq!(arr.get(10).unwrap(), false);
/// assert_eq!(arr.get(100), None);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! bit_array {
    ($num_bits:literal, $store:ty) => {{
        use $crate::utils::bit_array::{BitArray, BitStore};

        const NUM_BUCKETS: usize =
            ($num_bits as usize).div_ceil(<$store as BitStore>::BITS as usize);

        BitArray::<$store, NUM_BUCKETS>::new()
    }};
}

pub use bit_array;

/// Creates a [`Bits`] instance with the specified number of bits.
///
/// # Examples
/// ```rust
/// use o1::utils::bit_array::bits;
///
/// let mut b = bits!(u8);
/// b.set(5);
///
/// assert_eq!(b.get(5).unwrap(), true);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! bits {
    ($store:ty) => {{
        use $crate::utils::bit_array::Bits;
        Bits::<$store>::new()
    }};
}

pub use bits;

#[cfg(test)]
mod bits_tests {
    use super::*;

    #[test]
    const fn test_set_and_get() {
        let mut b = bits!(u8);
        b.set(0);
        b.set(5);
        b.set(7);

        assert!(b.get(0).unwrap());
        assert!(!b.get(1).unwrap());
        assert!(b.get(5).unwrap());
        assert!(b.get(7).unwrap());
        assert!(b.get(8).is_none());
    }

    #[test]
    fn test_iter_ones() {
        let mut b = bits!(u16);
        b.set(0);
        b.set(5);
        b.set(10);

        let ones: Vec<usize> = b.iter_ones().collect();
        assert_eq!(ones, vec![0, 5, 10]);
    }

    #[test]
    const fn test_iter_ones_const() {
        let mut b = bits!(u8);
        b.set(0);
        b.set(3);
        b.set(7);

        let mut ones = [0; 3];
        let mut i = 0;
        let mut iter = b.iter_ones_const();
        while let Some(index) = iter.next() {
            ones[i] = index;
            i += 1;
        }
        assert!(ones[0] == 0);
        assert!(ones[1] == 3);
        assert!(ones[2] == 7);
    }

    #[test]
    const fn test_count_ones() {
        let mut b = bits!(u32);
        assert!(b.count_ones() == 0);

        b.set(0);
        b.set(5);
        b.set(15);
        assert!(b.count_ones() == 3);

        b.clear(5);
        assert!(b.count_ones() == 2);

        b.set_all();
        assert!(b.count_ones() == 32);

        b.clear_all();
        assert!(b.count_ones() == 0);
    }

    #[test]
    fn test_clear_all() {
        let mut b8 = bits!(u8);
        b8.set(0);
        b8.set(5);
        b8.set(7);
        assert_eq!(b8.count_ones(), 3);
        b8.clear_all();
        assert_eq!(b8.count_ones(), 0);

        let mut b16 = bits!(u16);
        b16.set(0);
        b16.set(10);
        b16.set(15);
        assert_eq!(b16.count_ones(), 3);
        b16.clear_all();
        assert_eq!(b16.count_ones(), 0);

        let mut b32 = bits!(u32);
        b32.set(0);
        b32.set(15);
        b32.set(31);
        assert_eq!(b32.count_ones(), 3);
        b32.clear_all();
        assert_eq!(b32.count_ones(), 0);

        let mut b64 = bits!(u64);
        b64.set(0);
        b64.set(30);
        b64.set(63);
        assert_eq!(b64.count_ones(), 3);
        b64.clear_all();
        assert_eq!(b64.count_ones(), 0);

        let mut b128 = bits!(u128);
        b128.set(0);
        b128.set(64);
        b128.set(127);
        assert_eq!(b128.count_ones(), 3);
        b128.clear_all();
        assert_eq!(b128.count_ones(), 0);
    }

    #[test]
    fn test_set_all() {
        let mut b8 = bits!(u8);
        b8.set_all();
        assert_eq!(b8.count_ones(), 8);
        for i in 0..8 {
            assert!(b8.get(i).unwrap());
        }

        let mut b16 = bits!(u16);
        b16.set_all();
        assert_eq!(b16.count_ones(), 16);
        for i in 0..16 {
            assert!(b16.get(i).unwrap());
        }

        let mut b32 = bits!(u32);
        b32.set_all();
        assert_eq!(b32.count_ones(), 32);
        for i in 0..32 {
            assert!(b32.get(i).unwrap());
        }

        let mut b64 = bits!(u64);
        b64.set_all();
        assert_eq!(b64.count_ones(), 64);
        assert!(b64.get(0).unwrap());
        assert!(b64.get(31).unwrap());
        assert!(b64.get(32).unwrap());
        assert!(b64.get(63).unwrap());

        let mut b128 = bits!(u128);
        b128.set_all();
        assert_eq!(b128.count_ones(), 128);
        assert!(b128.get(0).unwrap());
        assert!(b128.get(63).unwrap());
        assert!(b128.get(64).unwrap());
        assert!(b128.get(127).unwrap());
    }

    #[test]
    const fn test_clear_all_set_all_const() {
        let mut b = bits!(u8);
        b.set(0);
        b.set(3);
        b.set(7);

        b.clear_all();
        assert!(b.count_ones() == 0);
        assert!(!b.get(0).unwrap());
        assert!(!b.get(3).unwrap());
        assert!(!b.get(7).unwrap());

        b.set_all();
        assert!(b.count_ones() == 8);
        assert!(b.get(0).unwrap());
        assert!(b.get(1).unwrap());
        assert!(b.get(2).unwrap());
        assert!(b.get(3).unwrap());
        assert!(b.get(4).unwrap());
        assert!(b.get(5).unwrap());
        assert!(b.get(6).unwrap());
        assert!(b.get(7).unwrap());
    }
}

#[cfg(test)]
mod bit_array_tests {
    use crate::utils::bit_array::BitArray;

    #[test]
    const fn test_set_and_get() {
        let mut arr = bit_array!(16, u8);
        arr.set(0);
        arr.set(5);
        arr.set(15);

        assert!(arr.get(0).unwrap());
        assert!(!arr.get(1).unwrap());
        assert!(arr.get(5).unwrap());
        assert!(arr.get(15).unwrap());
        assert!(arr.get(16).is_none());
    }

    #[test]
    const fn test_clear() {
        let mut arr = bit_array!(16, u8);
        arr.set(0);
        arr.set(5);
        arr.set(15);

        arr.clear(0);
        assert!(!arr.get(0).unwrap());
        assert!(arr.get(5).unwrap());

        arr.clear(5);
        assert!(!arr.get(5).unwrap());
        assert!(arr.get(15).unwrap());
    }

    #[test]
    const fn test_clear_all_and_set_all() {
        let mut arr = bit_array!(16, u8);

        arr.set_all();
        assert!(arr.get(0).unwrap());
        assert!(arr.get(7).unwrap());
        assert!(arr.get(8).unwrap());
        assert!(arr.get(15).unwrap());

        arr.clear_all();
        assert!(!arr.get(0).unwrap());
        assert!(!arr.get(7).unwrap());
        assert!(!arr.get(8).unwrap());
        assert!(!arr.get(15).unwrap());
    }

    #[test]
    const fn test_count_ones() {
        let mut arr = bit_array!(16, u8);
        assert!(arr.count_ones() == 0);

        arr.set(0);
        assert!(arr.count_ones() == 1);

        arr.set(3);
        arr.set(7);
        assert!(arr.count_ones() == 3);

        arr.set_all();
        assert!(arr.count_ones() == 16);

        arr.clear_all();
        assert!(arr.count_ones() == 0);
    }

    #[test]
    fn test_iter_ones() {
        let mut arr = bit_array!(16, u8);
        arr.set(0);
        arr.set(5);
        arr.set(15);

        let ones: Vec<usize> = arr.iter_ones().collect();
        assert!(ones == vec![0, 5, 15]);

        arr.clear(0);
        let ones: Vec<usize> = arr.iter_ones().collect();
        assert!(ones == vec![5, 15]);
    }

    #[test]
    const fn test_iter_ones_const() {
        let mut arr = bit_array!(16, u8);
        arr.set(0);
        arr.set(5);
        arr.set(15);

        let mut ones = [0; 3];
        let mut i = 0;
        let mut iter = arr.iter_ones_const();
        while let Some(index) = iter.next() {
            ones[i] = index;
            i += 1;
        }
        assert!(ones[0] == 0);
        assert!(ones[1] == 5);
        assert!(ones[2] == 15);

        arr.clear(0);

        let mut ones = [0; 2];
        let mut i = 0;
        let mut iter = arr.iter_ones_const();
        while let Some(index) = iter.next() {
            ones[i] = index;
            i += 1;
        }

        assert!(ones[0] == 5);
        assert!(ones[1] == 15);
    }

    #[test]
    fn test_different_storage_types() {
        let mut arr_u8 = BitArray::<u8, 2>::new();
        arr_u8.set(15);
        assert!(arr_u8.get(15).unwrap());

        let mut arr_u16 = BitArray::<u16, 2>::new();
        arr_u16.set(20);
        assert!(arr_u16.get(20).unwrap());

        let mut arr_u32 = BitArray::<u32, 2>::new();
        arr_u32.set(40);
        assert!(arr_u32.get(40).unwrap());

        let mut arr_u64 = BitArray::<u64, 2>::new();
        arr_u64.set(100);
        assert!(arr_u64.get(100).unwrap());

        let mut arr_u128 = BitArray::<u128, 2>::new();
        arr_u128.set(200);
        assert!(arr_u128.get(200).unwrap());
    }
}
