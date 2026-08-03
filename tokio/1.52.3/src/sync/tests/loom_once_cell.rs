use crate::sync::OnceCell;
use loom::future::block_on;
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::sync::Arc;
use loom::thread;
use std::future::{pending, Future};
use std::task::{Context, Poll};

#[test]
fn concurrent_initializers_publish_exactly_once() {
    loom::model(|| {
        let cell = Arc::new(OnceCell::new());
        let calls = Arc::new(AtomicUsize::new(0));

        let first = {
            let cell = cell.clone();
            let calls = calls.clone();
            thread::spawn(move || {
                block_on(async {
                    *cell
                        .get_or_init(|| async {
                            calls.fetch_add(1, Ordering::Relaxed);
                            17_u32
                        })
                        .await
                })
            })
        };
        let second = {
            let cell = cell.clone();
            let calls = calls.clone();
            thread::spawn(move || {
                block_on(async {
                    *cell
                        .get_or_init(|| async {
                            calls.fetch_add(1, Ordering::Relaxed);
                            29_u32
                        })
                        .await
                })
            })
        };

        let first = first.join().unwrap();
        let second = second.join().unwrap();
        assert_eq!(first, second);
        assert!(first == 17 || first == 29);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(cell.get(), Some(&first));
    });
}

#[test]
fn cancelled_initializer_releases_permit() {
    loom::model(|| {
        let cell = OnceCell::new();
        let mut init = Box::pin(cell.get_or_init(|| pending::<u32>()));
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(init.as_mut().poll(&mut cx), Poll::Pending));
        drop(init);

        assert_eq!(cell.set(41), Ok(()));
        assert_eq!(cell.get(), Some(&41));
    });
}

#[test]
fn failed_try_initializer_reopens_cell() {
    loom::model(|| {
        let cell = OnceCell::new();
        let result = block_on(cell.get_or_try_init(|| async { Err::<u32, u8>(3) }));
        assert_eq!(result, Err(3));
        assert_eq!(cell.set(43), Ok(()));
        assert_eq!(cell.get(), Some(&43));
    });
}
