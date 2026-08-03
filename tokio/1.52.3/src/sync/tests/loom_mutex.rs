use crate::sync::Mutex;
use loom::future::block_on;
use loom::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use loom::sync::Arc;
use loom::thread;
use std::future::{poll_fn, Future};
use std::task::Poll;

#[test]
fn borrowed_guard_excludes_and_releases() {
    loom::model(|| {
        let mutex = Arc::new(Mutex::new(()));
        let active = Arc::new(AtomicUsize::new(0));

        let spawned = {
            let mutex = mutex.clone();
            let active = active.clone();
            thread::spawn(move || {
                let _guard = block_on(mutex.lock());
                assert_eq!(active.fetch_add(1, SeqCst), 0);
                thread::yield_now();
                assert_eq!(active.fetch_sub(1, SeqCst), 1);
            })
        };

        let _guard = block_on(mutex.lock());
        assert_eq!(active.fetch_add(1, SeqCst), 0);
        thread::yield_now();
        assert_eq!(active.fetch_sub(1, SeqCst), 1);
        drop(_guard);
        spawned.join().unwrap();
        assert_eq!(active.load(SeqCst), 0);
    });
}

#[test]
fn cancelled_waiter_does_not_consume_permit() {
    loom::model(|| {
        let mutex = Mutex::new(0usize);
        block_on(async {
            let held = mutex.lock().await;
            let mut waiter = Some(mutex.lock());
            poll_fn(|cx| {
                let waiter = waiter.take().unwrap();
                pin!(waiter);
                assert!(waiter.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            })
            .await;
            drop(held);
            assert!(mutex.try_lock().is_ok());
        });
    });
}
