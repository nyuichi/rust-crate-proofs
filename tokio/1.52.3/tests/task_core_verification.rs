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
