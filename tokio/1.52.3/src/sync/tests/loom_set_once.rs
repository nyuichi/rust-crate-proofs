use crate::sync::{Notify, SetOnce};

use loom::future::block_on;
use loom::sync::atomic::AtomicU32;
use loom::thread;
use std::future::Future;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::{Context, Poll};

#[derive(Clone)]
struct DropCounter {
    pub drops: Arc<AtomicU32>,
}

impl DropCounter {
    pub fn new() -> Self {
        Self {
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

#[test]
fn set_once_drop_test() {
    loom::model(|| {
        let set_once = Arc::new(SetOnce::new());
        let set_once_clone = Arc::clone(&set_once);

        let drop_counter = DropCounter::new();
        let counter_cl = drop_counter.clone();

        let thread = thread::spawn(move || set_once_clone.set(counter_cl).is_ok());

        let foo = drop_counter.clone();

        let set = set_once.set(foo).is_ok();
        let res = thread.join().unwrap();

        drop(set_once);

        drop_counter.assert_num_drops(2);
        assert!(res != set);
    });
}

#[test]
fn set_once_wait_test() {
    loom::model(|| {
        let tx = Arc::new(SetOnce::new());
        let rx_one = tx.clone();
        let rx_two = tx.clone();

        let thread = thread::spawn(move || {
            assert!(rx_one.set(2).is_ok());
        });

        block_on(async {
            assert_eq!(*rx_two.wait().await, 2);
        });

        thread.join().unwrap();
    });
}

#[test]
fn set_once_wakes_all_registered_waiters_test() {
    loom::model(|| {
        let cell = SetOnce::new();
        let mut first = Box::pin(cell.wait());
        let mut second = Box::pin(cell.wait());
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        assert!(matches!(first.as_mut().poll(&mut cx), Poll::Pending));
        assert!(matches!(second.as_mut().poll(&mut cx), Poll::Pending));
        cell.set(19).unwrap();

        assert!(matches!(first.as_mut().poll(&mut cx), Poll::Ready(&19)));
        assert!(matches!(second.as_mut().poll(&mut cx), Poll::Ready(&19)));
        drop(first);
        drop(second);
        assert_eq!(cell.get(), Some(&19));
    });
}

#[test]
fn set_once_notify_generation_before_registration_test() {
    loom::model(|| {
        let notify = Notify::new();
        let mut notified = Box::pin(notify.notified());

        // The future captured the old notify_waiters generation but has not
        // registered in the intrusive list yet.
        notify.notify_waiters();

        // Its first poll must observe the changed generation and complete.
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(notified.as_mut().poll(&mut cx), Poll::Ready(())));
    });
}

#[test]
fn set_once_get_publication_test() {
    loom::model(|| {
        let cell = Arc::new(SetOnce::new());
        let writer = Arc::clone(&cell);

        let thread = thread::spawn(move || {
            writer.set((0x1234_u32, 0x5678_u32)).unwrap();
        });

        // Observe through `get`, rather than through `join`, so the Release
        // store and Acquire load in SetOnce are the publication edge checked
        // by loom.
        loop {
            if let Some(value) = cell.get() {
                assert_eq!(*value, (0x1234, 0x5678));
                break;
            }
            thread::yield_now();
        }

        thread.join().unwrap();
    });
}

#[test]
fn set_once_three_writers_test() {
    loom::model(|| {
        let cell = Arc::new(SetOnce::new());
        let first = Arc::clone(&cell);
        let second = Arc::clone(&cell);

        let first_thread = thread::spawn(move || first.set(10).is_ok());
        let second_thread = thread::spawn(move || second.set(20).is_ok());
        let third = cell.set(30).is_ok();

        let first = first_thread.join().unwrap();
        let second = second_thread.join().unwrap();
        assert_eq!(first as usize + second as usize + third as usize, 1);

        let value = *cell.get().unwrap();
        assert!(value == 10 || value == 20 || value == 30);
        assert_eq!(value == 10, first);
        assert_eq!(value == 20, second);
        assert_eq!(value == 30, third);
    });
}

#[test]
fn set_once_cancel_wait_then_rewait_test() {
    loom::model(|| {
        let cell = SetOnce::new();

        // Poll once so `Notified` registers a waiter, then cancel by dropping
        // the future. Its cancellation path must unlink safely and must not
        // alter SetOnce's publication state.
        let mut cancelled = Box::pin(cell.wait());
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(cancelled.as_mut().poll(&mut cx), Poll::Pending));
        drop(cancelled);

        assert!(cell.get().is_none());
        cell.set(77).unwrap();
        assert_eq!(*block_on(cell.wait()), 77);
    });
}
