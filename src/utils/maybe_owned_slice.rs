use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

/// A smart pointer that holds onto either an owned heap-allocated slice or a borrowed slice.
pub enum MaybeOwnedSliceMut<'a, T> {
    Borrowed(&'a mut [T]),
    Owned(Box<[T]>),
}

impl<T> Deref for MaybeOwnedSliceMut<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        match self {
            MaybeOwnedSliceMut::Borrowed(ref slice) => slice,
            MaybeOwnedSliceMut::Owned(boxed) => boxed,
        }
    }
}

impl<T> DerefMut for MaybeOwnedSliceMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeOwnedSliceMut::Borrowed(ref mut slice) => slice,
            MaybeOwnedSliceMut::Owned(ref mut boxed) => boxed,
        }
    }
}

impl<T> Debug for MaybeOwnedSliceMut<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeOwnedSliceMut::Borrowed(slice) => f.debug_tuple("Borrowed").field(slice).finish(),
            MaybeOwnedSliceMut::Owned(boxed) => f.debug_tuple("Owned").field(boxed).finish(),
        }
    }
}

impl<'a, T> MaybeOwnedSliceMut<'a, T> {
    pub fn from_vec(v: Vec<T>) -> Self {
        MaybeOwnedSliceMut::Owned(v.into_boxed_slice())
    }
    pub fn owned_into_vec(self) -> Vec<T> {
        match self {
            MaybeOwnedSliceMut::Owned(boxed) => boxed.into_vec(),
            MaybeOwnedSliceMut::Borrowed(_) => {
                panic!("Cannot convert borrowed slice to Vec. Use `as_slice()` method instead.",)
            }
        }
    }
    pub fn from_box(v: Box<[T]>) -> Self {
        MaybeOwnedSliceMut::Owned(v)
    }
    pub const fn from_slice(s: &'a mut [T]) -> Self {
        MaybeOwnedSliceMut::Borrowed(s)
    }
    pub const fn as_slice(&self) -> &[T] {
        match self {
            MaybeOwnedSliceMut::Borrowed(slice) => slice,
            MaybeOwnedSliceMut::Owned(boxed) => boxed,
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            MaybeOwnedSliceMut::Borrowed(ref mut slice) => slice,
            MaybeOwnedSliceMut::Owned(ref mut boxed) => &mut *boxed,
        }
    }
    pub const fn is_owned(&self) -> bool {
        matches!(self, MaybeOwnedSliceMut::Owned(_))
    }
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, MaybeOwnedSliceMut::Borrowed(_))
    }
}

impl<T> From<Vec<T>> for MaybeOwnedSliceMut<'_, T> {
    fn from(vec: Vec<T>) -> Self {
        MaybeOwnedSliceMut::from_vec(vec)
    }
}

impl<T> From<Box<[T]>> for MaybeOwnedSliceMut<'_, T> {
    fn from(boxed: Box<[T]>) -> Self {
        MaybeOwnedSliceMut::from_box(boxed)
    }
}

impl<'a, T> From<&'a mut [T]> for MaybeOwnedSliceMut<'a, T> {
    fn from(slice: &'a mut [T]) -> Self {
        MaybeOwnedSliceMut::from_slice(slice)
    }
}

impl<T: Clone> Clone for MaybeOwnedSliceMut<'_, T> {
    fn clone(&self) -> Self {
        match self {
            MaybeOwnedSliceMut::Borrowed(slice) => {
                MaybeOwnedSliceMut::Owned(slice.to_vec().into_boxed_slice())
            }
            MaybeOwnedSliceMut::Owned(boxed) => MaybeOwnedSliceMut::Owned(boxed.clone()),
        }
    }
}

impl<'b, T: PartialEq> PartialEq<MaybeOwnedSliceMut<'b, T>> for MaybeOwnedSliceMut<'_, T> {
    fn eq(&self, other: &MaybeOwnedSliceMut<'b, T>) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq> Eq for MaybeOwnedSliceMut<'_, T> {}

unsafe impl<T: Send> Send for MaybeOwnedSliceMut<'_, T> {}

unsafe impl<T: Sync> Sync for MaybeOwnedSliceMut<'_, T> {}

impl<T: Unpin> Unpin for MaybeOwnedSliceMut<'_, T> {}

impl<T> AsRef<[T]> for MaybeOwnedSliceMut<'_, T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for MaybeOwnedSliceMut<'_, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
