use o1_core::Hasher;
use std::fmt::{Debug, Formatter};

/// Hasher based on multiply-shift and polynomial hashing.
///
/// Contains both runtime and compile-time (const) implementations.
#[derive(Clone)]
pub struct MSPHasher<T: Eq>
where
    MSPHasher<T>: Hasher<T>,
{
    pub(super) state: <MSPHasher<T> as Hasher<T>>::State,
}

// Implement Copy for MSPHasher if its State is Copy
impl<T: Eq + Clone> Copy for MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Copy,
{
}

impl<T: Eq> Default for MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
{
    fn default() -> Self {
        <Self as Hasher<T>>::from_state(<Self as Hasher<T>>::State::default())
    }
}

impl<T> Debug for MSPHasher<T>
where
    T: Eq,
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MSPHasher")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Eq> MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Copy,
{
    /// Clone the hasher in a const context.
    pub const fn clone_const(&self) -> Self {
        Self { state: self.state }
    }
}
