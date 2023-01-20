use std::{
    fmt::{Debug, Pointer},
    ops::Deref,
};

/// A raw pointer to a pinned structure that is `!Unpin`.
pub struct RawPtr<T: ?Sized> {
    inner: *const T,
}

impl<T: ?Sized> RawPtr<T> {
    /// Creates a new raw pointer to `T`.
    ///
    /// While this method is safe per se, using the produced raw pointer is safe only if `ptr`
    /// references a pinned structure that is `!Unpin`.
    pub fn new(ptr: *const T) -> Self {
        Self { inner: ptr }
    }

    /// Returns whether the underlying raw pointer is `NULL`.
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

impl<T: ?Sized> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

impl<T: ?Sized> Copy for RawPtr<T> {}

impl<T: ?Sized> Deref for RawPtr<T> {
    type Target = *const T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> PartialEq for RawPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Sized> RawPtr<T> {
    pub fn null() -> Self {
        Self {
            inner: std::ptr::null(),
        }
    }

    pub fn ptr_eq(&self, ptr: *const T) -> bool {
        std::ptr::eq(self.inner, ptr)
    }
}

impl<T: ?Sized> Debug for RawPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pointer::fmt(&self.inner, f)
    }
}

// SAFETY: Only safe if T is pinned. In the current code, we only use this type on pinned variables.
unsafe impl<T: ?Sized> Send for RawPtr<T> {}

// SAFETY: Only safe if T is pinned. In the current code, we only use this type on pinned variables.
unsafe impl<T: ?Sized> Sync for RawPtr<T> {}
