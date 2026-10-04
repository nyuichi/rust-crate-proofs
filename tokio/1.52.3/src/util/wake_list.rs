use core::mem::MaybeUninit;
use core::ptr;
use std::task::Waker;

const NUM_WAKERS: usize = 32;

/// A list of wakers to be woken.
///
/// # Invariants
///
/// The first `curr` elements of `inner` are initialized.
pub(crate) struct WakeList {
    inner: [MaybeUninit<Waker>; NUM_WAKERS],
    curr: usize,
}

impl WakeList {
    pub(crate) fn new() -> Self {
        const UNINIT_WAKER: MaybeUninit<Waker> = MaybeUninit::uninit();

        Self {
            inner: [UNINIT_WAKER; NUM_WAKERS],
            curr: 0,
        }
    }

    #[inline]
    pub(crate) fn can_push(&self) -> bool {
        self.curr < NUM_WAKERS
    }

    pub(crate) fn push(&mut self, val: Waker) {
        debug_assert!(self.can_push());

        self.inner[self.curr] = MaybeUninit::new(val);
        self.curr += 1;
    }

    pub(crate) fn wake_all(&mut self) {
        struct DropGuard {
            start: *mut Waker,
            end: *mut Waker,
        }

        impl Drop for DropGuard {
            fn drop(&mut self) {
                // SAFETY: Both pointers are part of the same object, with `start <= end`.
                let len = unsafe { self.end.offset_from(self.start) } as usize;
                let slice = ptr::slice_from_raw_parts_mut(self.start, len);
                // SAFETY: All elements in `start..len` are initialized, so we can drop them.
                unsafe { ptr::drop_in_place(slice) };
            }
        }

        debug_assert!(self.curr <= NUM_WAKERS);

        let mut guard = {
            let start = self.inner.as_mut_ptr().cast::<Waker>();
            // SAFETY: The resulting pointer is in bounds or one after the length of the same object.
            let end = unsafe { start.add(self.curr) };
            // Transfer ownership of the wakers in `inner` to `DropGuard`.
            self.curr = 0;
            DropGuard { start, end }
        };
        while !ptr::eq(guard.start, guard.end) {
            // SAFETY: `start` is always initialized if `start != end`.
            let waker = unsafe { ptr::read(guard.start) };
            // SAFETY: The resulting pointer is in bounds or one after the length of the same object.
            guard.start = unsafe { guard.start.add(1) };
            // If this panics, then `guard` will clean up the remaining wakers.
            waker.wake();
        }
    }
}

impl Drop for WakeList {
    fn drop(&mut self) {
        let slice =
            ptr::slice_from_raw_parts_mut(self.inner.as_mut_ptr().cast::<Waker>(), self.curr);
        // SAFETY: The first `curr` elements are initialized, so we can drop them.
        unsafe { ptr::drop_in_place(slice) };
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::task::Wake;

    struct Probe {
        wakes: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
        panic_on_wake: bool,
    }

    impl Wake for Probe {
        fn wake(self: Arc<Self>) {
            self.wakes.fetch_add(1, Ordering::SeqCst);
            if self.panic_on_wake {
                panic!("wake probe");
            }
        }
    }

    impl Drop for Probe {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn probe(wakes: &Arc<AtomicUsize>, drops: &Arc<AtomicUsize>, panic_on_wake: bool) -> Waker {
        Waker::from(Arc::new(Probe {
            wakes: wakes.clone(),
            drops: drops.clone(),
            panic_on_wake,
        }))
    }

    #[test]
    fn capacity_and_drop_cover_exact_initialized_prefix() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        {
            let mut list = WakeList::new();
            for _ in 0..NUM_WAKERS {
                assert!(list.can_push());
                list.push(probe(&wakes, &drops, false));
            }
            assert!(!list.can_push());
        }
        assert_eq!(wakes.load(Ordering::SeqCst), 0);
        assert_eq!(drops.load(Ordering::SeqCst), NUM_WAKERS);
    }

    #[test]
    fn wake_panic_transfers_then_cleans_the_remaining_suffix() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let mut list = WakeList::new();
        for index in 0..NUM_WAKERS {
            list.push(probe(&wakes, &drops, index == 3));
        }

        let result = catch_unwind(AssertUnwindSafe(|| list.wake_all()));
        assert!(result.is_err());
        assert_eq!(wakes.load(Ordering::SeqCst), 4);
        assert_eq!(drops.load(Ordering::SeqCst), NUM_WAKERS);
        assert!(list.can_push());

        // `curr` was cleared before the first arbitrary wake call, so dropping
        // the list cannot touch the transferred wakers a second time.
        drop(list);
        assert_eq!(drops.load(Ordering::SeqCst), NUM_WAKERS);
    }
}
