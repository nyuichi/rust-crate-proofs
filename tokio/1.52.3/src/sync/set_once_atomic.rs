use crate::loom::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// The atomic publication operations used by `SetOnce`.
///
/// Keeping the orderings behind operation-specific methods makes the
/// Release/Acquire publication edge explicit and prevents unrelated call sites
/// from silently selecting a weaker ordering.
pub(crate) struct SetOnceFlag {
    value: AtomicBool,
}

impl SetOnceFlag {
    #[cfg(not(all(loom, test)))]
    pub(crate) const fn new(value: bool) -> Self {
        Self {
            value: AtomicBool::new(value),
        }
    }

    #[cfg(all(loom, test))]
    pub(crate) fn new(value: bool) -> Self {
        Self {
            value: AtomicBool::new(value),
        }
    }

    #[inline]
    pub(crate) fn load_acquire(&self) -> bool {
        self.value.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn store_release(&self, value: bool) {
        self.value.store(value, Ordering::Release);
    }

    #[inline]
    pub(crate) fn load_relaxed(&self) -> bool {
        self.value.load(Ordering::Relaxed)
    }

    #[inline]
    pub(crate) fn store_relaxed(&self, value: bool) {
        self.value.store(value, Ordering::Relaxed);
    }
}
