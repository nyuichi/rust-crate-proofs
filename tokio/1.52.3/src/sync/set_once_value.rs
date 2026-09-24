use crate::loom::cell::UnsafeCell;

use std::mem::MaybeUninit;
use std::ptr;

/// Operation-specific physical cell used by `SetOnce`.
///
/// Keeping the loom callback API behind these four operations makes the proof
/// adapter's contract exact: uninitialized/initialized construction, one
/// exclusive write, a publication-justified shared read, and an owned take.
#[repr(transparent)]
pub(super) struct SetOnceValue<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> SetOnceValue<T> {
    #[cfg(not(all(loom, test)))]
    pub(super) const fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[cfg(all(loom, test))]
    pub(super) fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[cfg(not(all(loom, test)))]
    pub(super) const fn initialized(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    #[cfg(all(loom, test))]
    pub(super) fn initialized(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::new(value)),
        }
    }

    /// # Safety
    ///
    /// The slot must be uninitialized and all writers must be serialized.
    pub(super) unsafe fn write(&self, value: T) {
        self.inner
            .with_mut(|ptr| unsafe { (*ptr).as_mut_ptr().write(value) });
    }

    /// # Safety
    ///
    /// The slot must be initialized and its publication must have been
    /// observed with an Acquire operation.
    pub(super) unsafe fn get(&self) -> &T {
        unsafe { &*self.inner.with(|ptr| (*ptr).as_ptr()) }
    }

    /// # Safety
    ///
    /// The slot must be initialized and the caller must have exclusive
    /// ownership of the complete SetOnce representation.
    pub(super) unsafe fn take(&mut self) -> T {
        unsafe { self.inner.with_mut(|ptr| ptr::read(ptr).assume_init()) }
    }
}
