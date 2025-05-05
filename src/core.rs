//! Core trait and type declarations for the whole project.

/// Hasher for the specific data-type.
///
/// Differs from [`core::hash::Hasher`] in the way that it is specific for a certain type and is not
/// supposed to be universal.
///
/// This allows the implementations to be leaner in terms of memory footprint
/// (in case if they need to store any state) and have less performance overhead by tailoring the
/// implementation to each target type and also avoiding the additional layer of indirection that
/// the pair [`core::hash::Hash`] and [`core::hash::Hasher`] have.
pub trait Hasher<T>
where
    Self: Default,
    T: Eq,
{
    /// State of the hasher instance.
    ///
    /// Usually contains such information as seed-values and number of buckets. But it's up to
    /// the implementation to decide what to store in it.
    type State: Clone + Default;

    /// Create a new hasher with a given `seed` and `num_buckets` number of buckets.
    fn from_seed(seed: u64, num_buckets: u32) -> Self;

    /// Create a new hasher from the given `state`.
    fn from_state(state: Self::State) -> Self;

    /// Get the state of the hasher.
    fn state(&self) -> &Self::State;

    /// Get the number of buckets (maximum value of the hash value).
    fn num_buckets(&self) -> u32;

    /// Hash the given `value`.
    ///
    /// # Notes
    ///
    /// - Currently only `u32` is supported due to lack of need for larger hash values.
    fn hash(&self, value: &T) -> u32;
}

pub trait ConstHasher<T>
where
    T: Eq,
{
    type HasherType: Hasher<T>;
}

// TODO: I'm not sure about the design choice of including `Hasher` as a generic parameter.
//       It prevents designing Maps that rely on some specific "internal" hasher or hashers that
//       require other inputs than just seed for initialization - for example count of keys
//       or the keys themselves.
//       Possible solutions:
//         - Move this parameter to the implementations completely.
//         - Split `Hasher` into builder and hasher traits so that implementations could provide
//           default builder for the generic parameter when needed.

/// An immutable hash map.
pub trait HashMap<K: Eq, V, H: Hasher<K>> {
    /// Get the value associated with the given `key`.
    fn get(&self, key: &K) -> Option<&V>;

    /// Get the number of elements in the map.
    fn len(&self) -> usize;

    /// Check if the map is empty.
    fn is_empty(&self) -> bool;

    /// Get the load factor of the map.
    fn load_factor(&self) -> f64;

    /// Get the number of collisions in the map.
    fn num_collisions(&self) -> usize;
}
