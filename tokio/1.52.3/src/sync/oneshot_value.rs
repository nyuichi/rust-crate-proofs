use crate::loom::cell::UnsafeCell;

/// Operation-specific wrapper for oneshot's `UnsafeCell<Option<T>>` slot.
/// State-bit ownership is checked by the caller; raw callback access does not
/// escape this module.
#[repr(transparent)]
pub(super) struct OneshotValue<T> {
    inner: UnsafeCell<Option<T>>,
}

impl<T> OneshotValue<T> {
    pub(super) fn empty() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    /// # Safety
    ///
    /// VALUE_SENT must be clear and the unique Sender must own slot access.
    pub(super) unsafe fn store(&self, value: T) {
        self.inner.with_mut(|ptr| unsafe { *ptr = Some(value) });
    }

    /// # Safety
    ///
    /// The state bits must grant the caller exclusive slot access.
    pub(super) unsafe fn take(&self) -> Option<T> {
        self.inner.with_mut(|ptr| unsafe { (*ptr).take() })
    }

    /// # Safety
    ///
    /// VALUE_SENT must have been observed with Acquire, making the Receiver
    /// the unique side allowed to inspect the slot.
    pub(super) unsafe fn has_value(&self) -> bool {
        self.inner.with(|ptr| unsafe { (*ptr).is_some() })
    }
}
