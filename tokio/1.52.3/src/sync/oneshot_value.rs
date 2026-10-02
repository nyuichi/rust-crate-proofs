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

#[cfg(test)]
mod verification_tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn oneshot_value_layout_matches_unsafe_cell_option() {
        assert_eq!(
            size_of::<OneshotValue<u64>>(),
            size_of::<UnsafeCell<Option<u64>>>()
        );
        assert_eq!(
            align_of::<OneshotValue<u64>>(),
            align_of::<UnsafeCell<Option<u64>>>()
        );
    }

    #[test]
    fn oneshot_value_store_take_roundtrip() {
        let slot = OneshotValue::empty();

        unsafe {
            assert!(!slot.has_value());
            slot.store(41_u64);
            assert!(slot.has_value());
            assert_eq!(slot.take(), Some(41));
            assert!(!slot.has_value());
            assert_eq!(slot.take(), None);
        }
    }
}
