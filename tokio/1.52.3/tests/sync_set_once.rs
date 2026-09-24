#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::task::{Context, Poll, Wake, Waker};
use tokio::sync::SetOnce;

macro_rules! assert_not_impl {
    ($ty:ty: $trait:path) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<A> {
                fn marker() {}
            }
            struct Invalid;
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            impl<T: ?Sized + $trait> AmbiguousIfImpl<Invalid> for T {}
            let _ = <$ty as AmbiguousIfImpl<_>>::marker;
        };
    };
}

assert_not_impl!(SetOnce<std::rc::Rc<()>>: Send);
assert_not_impl!(SetOnce<std::rc::Rc<()>>: Sync);
assert_not_impl!(SetOnce<std::cell::Cell<u8>>: Sync);

#[derive(Clone)]
struct DropCounter {
    drops: Arc<AtomicU32>,
}

impl DropCounter {
    fn new() -> Self {
        DropCounter {
            drops: Arc::new(AtomicU32::new(0)),
        }
    }

    fn assert_num_drops(&self, value: u32) {
        assert_eq!(value, self.drops.load(Ordering::Relaxed));
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}

struct PanicDrop {
    drops: Arc<AtomicU32>,
}

struct PanicClone {
    clones: Arc<AtomicU32>,
    id: u64,
}

impl Clone for PanicClone {
    fn clone(&self) -> Self {
        self.clones.fetch_add(1, Ordering::Relaxed);
        panic!("payload clone panic");
    }
}

struct PanicEq {
    comparisons: Arc<AtomicU32>,
    id: u64,
}

impl PartialEq for PanicEq {
    fn eq(&self, _other: &Self) -> bool {
        self.comparisons.fetch_add(1, Ordering::Relaxed);
        panic!("payload equality panic");
    }
}

impl Drop for PanicDrop {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::Relaxed);
        panic!("payload destructor panic");
    }
}

#[test]
fn drop_cell() {
    let fooer = DropCounter::new();
    let fooer_cl = fooer.clone();

    {
        let once_cell = SetOnce::new();
        let prev = once_cell.set(fooer_cl);
        assert!(prev.is_ok())
    }

    fooer.assert_num_drops(1);
}

#[test]
fn drop_cell_new_with() {
    let fooer = DropCounter::new();

    {
        let once_cell = SetOnce::new_with(Some(fooer.clone()));
        assert!(once_cell.initialized());
    }

    fooer.assert_num_drops(1);
}

#[test]
fn drop_into_inner() {
    let fooer = DropCounter::new();

    let once_cell = SetOnce::new();
    assert!(once_cell.set(fooer.clone()).is_ok());
    let val = once_cell.into_inner();
    fooer.assert_num_drops(0);
    drop(val);
    fooer.assert_num_drops(1);
}

#[test]
fn drop_into_inner_new_with() {
    let fooer = DropCounter::new();

    let once_cell = SetOnce::new_with(Some(fooer.clone()));
    let val = once_cell.into_inner();
    fooer.assert_num_drops(0);
    drop(val);
    fooer.assert_num_drops(1);
}

#[test]
fn panicking_payload_destructor_runs_once() {
    let drops = Arc::new(AtomicU32::new(0));
    let cell = SetOnce::from(PanicDrop {
        drops: Arc::clone(&drops),
    });

    let result = catch_unwind(AssertUnwindSafe(|| drop(cell)));
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn into_inner_transfers_panicking_destructor() {
    let drops = Arc::new(AtomicU32::new(0));
    let cell = SetOnce::from(PanicDrop {
        drops: Arc::clone(&drops),
    });
    let value = cell.into_inner().unwrap();

    assert_eq!(drops.load(Ordering::Relaxed), 0);
    let result = catch_unwind(AssertUnwindSafe(|| drop(value)));
    assert!(result.is_err());
    assert_eq!(drops.load(Ordering::Relaxed), 1);
}

#[test]
fn clone_panic_preserves_source() {
    let clones = Arc::new(AtomicU32::new(0));
    let cell = SetOnce::from(PanicClone {
        clones: Arc::clone(&clones),
        id: 17,
    });

    let result = catch_unwind(AssertUnwindSafe(|| cell.clone()));
    assert!(result.is_err());
    assert_eq!(clones.load(Ordering::Relaxed), 1);
    assert_eq!(cell.get().map(|value| value.id), Some(17));
}

#[test]
fn equality_panic_preserves_both_cells() {
    let comparisons = Arc::new(AtomicU32::new(0));
    let left = SetOnce::from(PanicEq {
        comparisons: Arc::clone(&comparisons),
        id: 23,
    });
    let right = SetOnce::from(PanicEq {
        comparisons: Arc::clone(&comparisons),
        id: 29,
    });

    let result = catch_unwind(AssertUnwindSafe(|| left == right));
    assert!(result.is_err());
    assert_eq!(comparisons.load(Ordering::Relaxed), 1);
    assert_eq!(left.get().map(|value| value.id), Some(23));
    assert_eq!(right.get().map(|value| value.id), Some(29));
}

#[test]
fn from() {
    let cell = SetOnce::from(2);
    assert_eq!(*cell.get().unwrap(), 2);
}

#[test]
fn set_and_get() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    ONCE.set(5).unwrap();
    let value = ONCE.get().unwrap();
    assert_eq!(*value, 5);
}

#[test]
fn const_constructors_match_runtime_state() {
    const EMPTY: SetOnce<u64> = SetOnce::const_new();
    const FULL: SetOnce<u64> = SetOnce::const_new_with(73);

    let runtime_empty = SetOnce::<u64>::new();
    let runtime_full = SetOnce::new_with(Some(73));

    assert_eq!(EMPTY.get(), runtime_empty.get());
    assert_eq!(FULL.get(), runtime_full.get());
    assert_eq!(EMPTY.initialized(), runtime_empty.initialized());
    assert_eq!(FULL.initialized(), runtime_full.initialized());
}

#[tokio::test]
async fn set_and_wait() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    tokio::spawn(async { ONCE.set(5) });

    let value = ONCE.wait().await;
    assert_eq!(*value, 5);
}

#[test]
#[cfg_attr(target_family = "wasm", ignore)]
fn set_and_wait_multiple_threads() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    let res1 = std::thread::spawn(|| ONCE.set(4));

    let res2 = std::thread::spawn(|| ONCE.set(3));

    let result_first = res1.join().unwrap().is_err();
    let result_two = res2.join().unwrap().is_err();

    assert!(result_first != result_two);
}

#[tokio::test]
#[cfg_attr(target_family = "wasm", ignore)]
async fn set_and_wait_threads() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    let thread = std::thread::spawn(|| {
        ONCE.set(4).unwrap();
    });

    let value = ONCE.wait().await;
    thread.join().unwrap();
    assert_eq!(*value, 4);
}

#[test]
fn get_uninit() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();
    let uninit = ONCE.get();
    assert!(uninit.is_none());
}

#[test]
fn set_twice() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    let first = ONCE.set(5);
    assert_eq!(first, Ok(()));
    let second = ONCE.set(6);
    assert!(second.is_err());
}

#[test]
fn is_none_initializing() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    assert_eq!(ONCE.get(), None);

    ONCE.set(20).unwrap();

    assert!(ONCE.set(10).is_err());
}

#[tokio::test]
async fn is_some_initializing() {
    static ONCE: SetOnce<u32> = SetOnce::const_new();

    tokio::spawn(async { ONCE.set(20) });

    assert_eq!(*ONCE.wait().await, 20);
}

#[test]
fn into_inner_int_empty_setonce() {
    let once = SetOnce::<u32>::new();

    let val = once.into_inner();

    assert!(val.is_none());
}

#[test]
fn auto_trait_bounds() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<SetOnce<std::cell::Cell<u8>>>();
    assert_send::<SetOnce<u64>>();
    assert_sync::<SetOnce<u64>>();
}

#[test]
fn trait_views_preserve_state_and_value() {
    let empty = SetOnce::<String>::default();
    let empty_clone = empty.clone();
    assert_eq!(empty, empty_clone);
    assert_eq!(format!("{empty:?}"), "SetOnce { value: None }");

    let initialized = SetOnce::from(String::from("published"));
    let initialized_clone = initialized.clone();
    assert_eq!(initialized, initialized_clone);
    assert_eq!(
        format!("{initialized:?}"),
        "SetOnce { value: Some(\"published\") }"
    );
}

#[test]
fn set_once_error_traits() {
    use std::error::Error;

    let error = tokio::sync::SetOnceError(7_u64);
    assert_eq!(error.to_string(), "SetOnceError");
    assert!(error.source().is_none());
    assert_eq!(error, tokio::sync::SetOnceError(7));
    assert_eq!(format!("{error:?}"), "SetOnceError(7)");
}

struct PanicWake;

impl Wake for PanicWake {
    fn wake(self: Arc<Self>) {
        panic!("wake requested");
    }
}

struct CountingWake {
    wakes: Arc<AtomicU32>,
}

impl Wake for CountingWake {
    fn wake(self: Arc<Self>) {
        self.wakes.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn publication_survives_panicking_waiter_wake() {
    let cell = SetOnce::new();
    let mut waiter = Box::pin(cell.wait());
    let waker = Waker::from(Arc::new(PanicWake));
    let mut cx = Context::from_waker(&waker);

    assert!(matches!(Pin::new(&mut waiter).poll(&mut cx), Poll::Pending));

    let set_result = catch_unwind(AssertUnwindSafe(|| cell.set(41_u64)));
    assert!(set_result.is_err());

    // Publication precedes notification. Even though waking unwound through
    // `set`, the value remains initialized and later writers are rejected.
    drop(waiter);
    assert_eq!(cell.get(), Some(&41));
    assert_eq!(cell.set(42), Err(tokio::sync::SetOnceError(42)));
}

#[test]
fn panicking_waker_leaves_remaining_waiters_unlinked_and_ready() {
    let cell = SetOnce::new();
    let before_count = Arc::new(AtomicU32::new(0));
    let after_count = Arc::new(AtomicU32::new(0));

    let mut before = Box::pin(cell.wait());
    let mut panics = Box::pin(cell.wait());
    let mut after = Box::pin(cell.wait());

    let before_waker = Waker::from(Arc::new(CountingWake {
        wakes: Arc::clone(&before_count),
    }));
    let panic_waker = Waker::from(Arc::new(PanicWake));
    let after_waker = Waker::from(Arc::new(CountingWake {
        wakes: Arc::clone(&after_count),
    }));
    let mut before_cx = Context::from_waker(&before_waker);
    let mut panic_cx = Context::from_waker(&panic_waker);
    let mut after_cx = Context::from_waker(&after_waker);

    assert!(matches!(
        before.as_mut().poll(&mut before_cx),
        Poll::Pending
    ));
    assert!(matches!(panics.as_mut().poll(&mut panic_cx), Poll::Pending));
    assert!(matches!(after.as_mut().poll(&mut after_cx), Poll::Pending));

    let result = catch_unwind(AssertUnwindSafe(|| cell.set(47_u64)));
    assert!(result.is_err());
    assert_eq!(before_count.load(Ordering::Relaxed), 1);
    assert_eq!(after_count.load(Ordering::Relaxed), 0);

    let noop = futures::task::noop_waker();
    let mut noop_cx = Context::from_waker(&noop);
    assert!(matches!(
        before.as_mut().poll(&mut noop_cx),
        Poll::Ready(&47)
    ));
    assert!(matches!(
        panics.as_mut().poll(&mut noop_cx),
        Poll::Ready(&47)
    ));
    assert!(matches!(
        after.as_mut().poll(&mut noop_cx),
        Poll::Ready(&47)
    ));
    assert_eq!(cell.get(), Some(&47));
}
