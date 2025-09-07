use o1_core::Hasher;
use std::fmt::{Debug, Formatter};

/// Hasher based on XXH3 algorithm.
///
/// Contains both runtime and compile-time (const) implementations.
#[derive(Clone)]
pub struct XXH3Hasher<T: Eq>
where
    XXH3Hasher<T>: Hasher<T>,
{
    pub(super) state: <XXH3Hasher<T> as Hasher<T>>::State,
}

impl<T: Eq + Clone> Copy for XXH3Hasher<T>
where
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Copy,
{
}

impl<T: Eq> Default for XXH3Hasher<T>
where
    XXH3Hasher<T>: Hasher<T>,
{
    fn default() -> Self {
        <Self as Hasher<T>>::from_state(<Self as Hasher<T>>::State::default())
    }
}

impl<T> Debug for XXH3Hasher<T>
where
    T: Eq,
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XXH3Hasher")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Eq> XXH3Hasher<T>
where
    XXH3Hasher<T>: Hasher<T>,
    <XXH3Hasher<T> as Hasher<T>>::State: Copy,
{
    /// Clone the hasher in a const context.
    pub const fn clone_const(&self) -> Self {
        Self { state: self.state }
    }
}
