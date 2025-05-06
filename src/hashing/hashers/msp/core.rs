use crate::core::Hasher;
use std::fmt::{Debug, Formatter};

/// Hasher based on multiply-shift and polynomial hashing.
#[derive(Clone)]
pub struct MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
{
    pub(super) state: <MSPHasher<T> as Hasher<T>>::State,
}

impl<T> Default for MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
{
    fn default() -> Self {
        <Self as Hasher<T>>::from_state(<Self as Hasher<T>>::State::default())
    }
}

impl<T> Debug for MSPHasher<T>
where
    MSPHasher<T>: Hasher<T>,
    <MSPHasher<T> as Hasher<T>>::State: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MSPHasher")
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct ConstMSPHasher<T, H>
where
    H: Hasher<T>,
{
    pub(super) state: H::State,
}
