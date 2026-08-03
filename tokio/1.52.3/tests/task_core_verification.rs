#![cfg(feature = "rt")]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

struct WakeThenReady {
    polls: Arc<AtomicUsize>,
}

impl Future for WakeThenReady {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let poll = self.polls.fetch_add(1, Ordering::SeqCst);
        if poll == 0 {
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(42)
        }
    }
}

fn run_normal_path(builder: &mut tokio::runtime::Builder) {
    let runtime = builder.build().unwrap();
    let polls = Arc::new(AtomicUsize::new(0));
    let output = runtime.block_on(runtime.spawn(WakeThenReady {
        polls: polls.clone(),
    }));
    assert_eq!(output.unwrap(), 42);
    assert_eq!(polls.load(Ordering::SeqCst), 2);
}

#[test]
fn current_thread_spawn_wake_reschedule_and_complete() {
    run_normal_path(&mut tokio::runtime::Builder::new_current_thread());
}

#[cfg(feature = "rt-multi-thread")]
#[test]
fn multi_thread_spawn_wake_reschedule_and_complete() {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(2);
    run_normal_path(&mut builder);
}

struct PendingDrop {
    drops: Arc<AtomicUsize>,
}

impl Future for PendingDrop {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

impl Drop for PendingDrop {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn abort_handle_cancels_and_join_transfers_error_once() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let handle = runtime.spawn(PendingDrop {
        drops: drops.clone(),
    });
    let abort = handle.abort_handle();
    let cloned = abort.clone();
    assert_eq!(abort.id(), handle.id());
    assert_eq!(cloned.id(), handle.id());
    abort.abort();
    cloned.abort();

    let error = runtime.block_on(handle).unwrap_err();
    assert!(error.is_cancelled());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop(abort);
    drop(cloned);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

struct OutputDrop(Arc<AtomicUsize>);

impl Drop for OutputDrop {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn dropping_completed_join_handle_drops_output_once() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    runtime.block_on(async {
        let handle = tokio::spawn({
            let drops = drops.clone();
            async move { OutputDrop(drops) }
        });
        while !handle.is_finished() {
            tokio::task::yield_now().await;
        }
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(handle);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}
