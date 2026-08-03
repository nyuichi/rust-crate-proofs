use std::cell::RefCell;
use std::task::Waker;

pub(crate) struct Defer {
    deferred: RefCell<Vec<Waker>>,
}

impl Defer {
    pub(crate) fn new() -> Defer {
        Defer {
            deferred: RefCell::default(),
        }
    }

    pub(crate) fn defer(&self, waker: &Waker) {
        let mut deferred = self.deferred.borrow_mut();

        // If the same task adds itself a bunch of times, then only add it once.
        if let Some(last) = deferred.last() {
            if last.will_wake(waker) {
                return;
            }
        }

        deferred.push(waker.clone());
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.deferred.borrow().is_empty()
    }

    pub(crate) fn wake(&self) {
        while let Some(waker) = self.deferred.borrow_mut().pop() {
            waker.wake();
        }
    }

    #[cfg(feature = "taskdump")]
    pub(crate) fn take_deferred(&self) -> Vec<Waker> {
        let mut deferred = self.deferred.borrow_mut();
        std::mem::take(&mut *deferred)
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Arc;
    use std::task::{RawWaker, RawWakerVTable, Wake};

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    struct PanicWake;

    impl Wake for PanicWake {
        fn wake(self: Arc<Self>) {
            panic!("deferred wake panic");
        }
    }

    unsafe fn clone_panics(_: *const ()) -> RawWaker {
        panic!("waker clone panic");
    }

    unsafe fn raw_noop(_: *const ()) {}

    static PANIC_CLONE_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone_panics, raw_noop, raw_noop, raw_noop);

    #[test]
    fn defer_clone_panic_preserves_queue() {
        let defer = Defer::new();
        let waker =
            unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &PANIC_CLONE_VTABLE)) };

        let result = catch_unwind(AssertUnwindSafe(|| defer.defer(&waker)));
        assert!(result.is_err());
        assert!(defer.is_empty());
    }

    #[test]
    fn defer_waker_panic_pops_before_call() {
        let defer = Defer::new();
        let survivor = Waker::from(Arc::new(NoopWake));
        let panicker = Waker::from(Arc::new(PanicWake));
        defer.defer(&survivor);
        defer.defer(&panicker);

        let result = catch_unwind(AssertUnwindSafe(|| defer.wake()));
        assert!(result.is_err());
        let queued = defer.deferred.borrow();
        assert_eq!(queued.len(), 1);
        assert!(queued[0].will_wake(&survivor));
    }

    #[test]
    fn defer_adjacent_duplicate_once_and_drop_releases_clone() {
        let owner = Arc::new(NoopWake);
        let waker = Waker::from(owner.clone());
        let defer = Defer::new();
        let before = Arc::strong_count(&owner);

        defer.defer(&waker);
        assert_eq!(defer.deferred.borrow().len(), 1);
        assert_eq!(Arc::strong_count(&owner), before + 1);
        defer.defer(&waker);
        assert_eq!(defer.deferred.borrow().len(), 1);
        assert_eq!(Arc::strong_count(&owner), before + 1);

        drop(defer);
        assert_eq!(Arc::strong_count(&owner), before);
    }
}
