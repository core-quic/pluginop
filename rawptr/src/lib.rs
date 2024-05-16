//! An internal crate to offer a safe interface over raw pointers
//! to **pinned** types.

use std::{
    fmt::{Debug, Pointer},
    io::Cursor,
    ops::{Deref, DerefMut},
};

/// A raw pointer to a pinned structure that is `!Unpin`.
pub struct RawPtr<T: ?Sized> {
    inner: *const T,
}

#[allow(dead_code)]
impl<T: ?Sized> RawPtr<T> {
    /// Creates a new raw pointer to `T`.
    ///
    /// SAFETY: While this method is safe per se, the whole safety of all the methods of the
    /// `RawPtr` wrapper relies on `ptr` referencing a pinned structure that is `!Unpin`.
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
        *self
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
        std::ptr::eq(self.inner, other.inner)
    }
}

#[allow(dead_code)]
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

/// A raw mutable pointer to a pinned structure that is `!Unpin`.
pub struct RawMutPtr<T: ?Sized> {
    inner: *mut T,
}

impl<T: ?Sized> RawMutPtr<T> {
    /// Creates a new raw mutable pointer to `T`.
    ///
    /// SAFETY: While this method is safe per se, the whole safety of all the methods of the
    /// `RawMutPtr` wrapper relies on `ptr` referencing a pinned structure that is `!Unpin`.
    pub fn new(ptr: *mut T) -> Self {
        Self { inner: ptr }
    }

    /// Returns whether the underlying raw pointer is `NULL`.
    pub fn is_null(&self) -> bool {
        self.inner.is_null()
    }
}

impl<T: ?Sized> Clone for RawMutPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for RawMutPtr<T> {}

impl<T: ?Sized> Deref for RawMutPtr<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: ?Sized> DerefMut for RawMutPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: ?Sized> PartialEq for RawMutPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inner, other.inner)
    }
}

impl<T: Sized> RawMutPtr<T> {
    pub fn null() -> Self {
        Self {
            inner: std::ptr::null_mut(),
        }
    }

    pub fn ptr_eq(&self, ptr: *mut T) -> bool {
        std::ptr::eq(self.inner, ptr)
    }
}

impl<T: ?Sized> Debug for RawMutPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Pointer::fmt(&self.inner, f)
    }
}

// SAFETY: Only safe if T is pinned AND if the underlying reference is not moved out. In the
// current code, we only use this type on pinned variables.
unsafe impl<T: ?Sized> Send for RawMutPtr<T> {}

// SAFETY: Only safe if T is pinned AND if the underlying reference is not moved out. In the
// current code, we only use this type on pinned variables.
unsafe impl<T: ?Sized> Sync for RawMutPtr<T> {}

use bytes::{Bytes, BytesMut};

/// A (safe) wrapper over a raw pointer to [`Cursor<Bytes>`]
#[derive(Debug)]
pub struct CursorBytesPtr(RawMutPtr<Cursor<Bytes>>);

impl Deref for CursorBytesPtr {
    type Target = Cursor<Bytes>;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Valid only if `Cursor` is pinned and in single-thread.
        unsafe { &**self.0 }
    }
}

impl DerefMut for CursorBytesPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Valid only if `Cursor` is pinned and in single-thread.
        unsafe { &mut **self.0 }
    }
}

impl From<&mut Cursor<Bytes>> for CursorBytesPtr {
    fn from(value: &mut Cursor<Bytes>) -> Self {
        // Don't forget the memory here since we have only a reference.
        CursorBytesPtr(RawMutPtr::new(value as *const _ as *mut _))
    }
}

/// A (safe) wrapper over a raw pointer to [`BytesMut`].
#[derive(Debug)]
pub struct BytesMutPtr(RawMutPtr<BytesMut>);

impl Deref for BytesMutPtr {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Valid only if `BytesMut` is pinned and in single-thread.
        unsafe { &**self.0 }
    }
}

impl DerefMut for BytesMutPtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Valid only if `BytesMut` is pinned and in single-thread.
        unsafe { &mut **self.0 }
    }
}

impl From<&mut BytesMut> for BytesMutPtr {
    fn from(value: &mut BytesMut) -> Self {
        // Don't forget the memory here since we have only a reference.
        BytesMutPtr(RawMutPtr::new(value as *const _ as *mut _))
    }
}
