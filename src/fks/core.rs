//! Declares core types for [`FKSMap`].
use crate::core::Hasher;
use crate::utils::maybe_owned_slice::MaybeOwnedSliceMut;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// Static hash-table based on the FKS scheme.
///
/// # Guarantees
///
/// - O(1) time complexity for lookups.
/// - At most 2 memory reads for a key lookup.
///
/// # Examples
///
/// ```rust
/// use o1::core::HashMap;
/// use o1::hashing::hashers::MSPHasher;
/// use o1::fks::FKSMap;
///
/// let book_reviews = FKSMap::<&str, &str, MSPHasher<&str>>::new(
///     [
///         ("Adventures of Huckleberry Finn", "My favorite book."),
///         ("Grimms' Fairy Tales", "Masterpiece."),
///         ("Pride and Prejudice", "Very enjoyable."),
///         ("The Adventures of Sherlock Holmes", "Eye lyked it alot."),
///     ].into(),
///     42,
///     0.75,
/// ).unwrap();
///
/// // Check for a specific one.
/// if book_reviews.get(&"Les Misérables").is_none() {
///     println!("We've got {} reviews, but Les Misérables ain't one.",
///              book_reviews.len());
/// }
/// ```
pub struct FKSMap<'a, K: Eq, V, H: Hasher<K>> {
    #[doc(hidden)]
    pub l1_hasher: H,
    #[doc(hidden)]
    pub buckets: MaybeOwnedSliceMut<'a, Bucket<K, H>>,
    #[doc(hidden)]
    pub slots: MaybeOwnedSliceMut<'a, MaybeUninit<(K, V)>>,
}

impl<K, V, H> Debug for FKSMap<'_, K, V, H>
where
    K: Eq + Debug,
    V: Debug,
    H: Hasher<K> + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FKSMap")
            .field("l1_hasher", &self.l1_hasher)
            .field("buckets", &self.buckets)
            .field("slots", &self.slots)
            .finish()
    }
}

/// A bucket of the hash table.
///
/// Each bucket is associated with an output of the L1 hash function and a number of slots that
/// contain the collided keys.
#[derive(Debug)]
pub struct Bucket<K: Eq, H: Hasher<K>> {
    /// The offset of the first slot in the bucket.
    pub offset: usize,
    /// A bit-mask of the occupied slots in the bucket.
    pub slots: u8,
    /// A number of slots in the bucket.
    pub num_slots: u8,
    /// L2 hasher that contains parameters for the L2 hash function.
    pub hasher: H,
    pub key_type: PhantomData<K>,
}

impl<K: Eq, H: Hasher<K>> Bucket<K, H> {
    #[inline]
    pub fn num_slots(&self) -> usize {
        self.num_slots as usize
    }
}

impl<K: Eq, H: Hasher<K>> Default for Bucket<K, H> {
    fn default() -> Self {
        Self {
            offset: 0,
            slots: 0,
            num_slots: 0,
            hasher: H::default(),
            key_type: PhantomData,
        }
    }
}
