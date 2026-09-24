#![cfg_attr(not(feature = "sync"), allow(dead_code, unreachable_pub))]

//! A multi-producer, single-consumer queue for sending values between
//! asynchronous tasks.
//!
//! This module provides two variants of the channel: bounded and unbounded. The
//! bounded variant has a limit on the number of messages that the channel can
//! store, and if this limit is reached, trying to send another message will
//! wait until a message is received from the channel. An unbounded channel has
//! an infinite capacity, so the `send` method will always complete immediately.
//! This makes the [`UnboundedSender`] usable from both synchronous and
//! asynchronous code.
//!
//! Similar to the `mpsc` channels provided by `std`, the channel constructor
//! functions provide separate send and receive handles, [`Sender`] and
//! [`Receiver`] for the bounded channel, [`UnboundedSender`] and
//! [`UnboundedReceiver`] for the unbounded channel. If there is no message to read,
//! the current task will be notified when a new value is sent. [`Sender`] and
//! [`UnboundedSender`] allow sending values into the channel. If the bounded
//! channel is at capacity, the send is rejected and the task will be notified
//! when additional capacity is available. In other words, the channel provides
//! backpressure.
//!
//! This channel is also suitable for the single-producer single-consumer
//! use-case. (Unless you only need to send one message, in which case you
//! should use the [oneshot] channel.)
//!
//! # Disconnection
//!
//! When all [`Sender`] handles have been dropped, it is no longer
//! possible to send values into the channel. This is considered the termination
//! event of the stream. Once all senders have been dropped and any remaining
//! buffered values have been received, `Receiver::recv` returns `None`
//! (and `Receiver::poll_recv` returns `Poll::Ready(None)`).
//!
//! If the [`Receiver`] handle is dropped, then messages can no longer
//! be read out of the channel. In this case, all further attempts to send will
//! result in an error. Additionally, all unread messages will be drained from the
//! channel and dropped.
//!
//! # Clean Shutdown
//!
//! When the [`Receiver`] is dropped, it is possible for unprocessed messages to
//! remain in the channel. Instead, it is usually desirable to perform a "clean"
//! shutdown. To do this, the receiver first calls `close`, which will prevent
//! any further messages to be sent into the channel. Then, the receiver
//! consumes the channel to completion, at which point the receiver can be
//! dropped.
//!
//! # Communicating between sync and async code
//!
//! When you want to communicate between synchronous and asynchronous code, there
//! are two situations to consider:
//!
//! **Bounded channel**: If you need a bounded channel, you should use a bounded
//! Tokio `mpsc` channel for both directions of communication. Instead of calling
//! the async [`send`][bounded-send] or [`recv`][bounded-recv] methods, in
//! synchronous code you will need to use the [`blocking_send`][blocking-send] or
//! [`blocking_recv`][blocking-recv] methods.
//!
//! **Unbounded channel**: You should use the kind of channel that matches where
//! the receiver is. So for sending a message _from async to sync_, you should
//! use [the standard library unbounded channel][std-unbounded] or
//! [crossbeam][crossbeam-unbounded].  Similarly, for sending a message _from sync
//! to async_, you should use an unbounded Tokio `mpsc` channel.
//!
//! Please be aware that the above remarks were written with the `mpsc` channel
//! in mind, but they can also be generalized to other kinds of channels. In
//! general, any channel method that isn't marked async can be called anywhere,
//! including outside of the runtime. For example, sending a message on a
//! [oneshot] channel from outside the runtime is perfectly fine.
//!
//! # Multiple runtimes
//!
//! The `mpsc` channel is runtime agnostic. You can freely move it between
//! different instances of the Tokio runtime or even use it from non-Tokio
//! runtimes.
//!
//! When used in a Tokio runtime, it participates in
//! [cooperative scheduling](crate::task::coop#cooperative-scheduling) to avoid
//! starvation. This feature does not apply when used from non-Tokio runtimes.
//!
//! As an exception, methods ending in `_timeout` are not runtime agnostic
//! because they require access to the Tokio timer. See the documentation of
//! each `*_timeout` method for more information on its use.
//!
//! # Allocation behavior
//!
//! <div class="warning">The implementation details described in this section may change in future
//! Tokio releases.</div>
//!
//! The mpsc channel stores elements in blocks. Blocks are organized in a linked list. Sending
//! pushes new elements onto the block at the front of the list, and receiving pops them off the
//! one at the back. A block can hold 32 messages on a 64-bit target and 16 messages on a 32-bit
//! target. This number is independent of channel and message size. Each block also stores 4
//! pointer-sized values for bookkeeping (so on a 64-bit machine, each message has 1 byte of
//! overhead).
//!
//! When all values in a block have been received, it becomes empty. It will then be freed, unless
//! the channel's first block (where newly-sent elements are being stored) has no next block. In
//! that case, the empty block is reused as the next block.
//!
//! [`Sender`]: crate::sync::mpsc::Sender
//! [`Receiver`]: crate::sync::mpsc::Receiver
//! [bounded-send]: crate::sync::mpsc::Sender::send()
//! [bounded-recv]: crate::sync::mpsc::Receiver::recv()
//! [blocking-send]: crate::sync::mpsc::Sender::blocking_send()
//! [blocking-recv]: crate::sync::mpsc::Receiver::blocking_recv()
//! [`UnboundedSender`]: crate::sync::mpsc::UnboundedSender
//! [`UnboundedReceiver`]: crate::sync::mpsc::UnboundedReceiver
//! [oneshot]: crate::sync::oneshot
//! [`Handle::block_on`]: crate::runtime::Handle::block_on()
//! [std-unbounded]: std::sync::mpsc::channel
//! [crossbeam-unbounded]: https://docs.rs/crossbeam/*/crossbeam/channel/fn.unbounded.html
//! [`send_timeout`]: crate::sync::mpsc::Sender::send_timeout

pub(super) mod block;

mod bounded;
pub use self::bounded::{
    channel, OwnedPermit, Permit, PermitIterator, Receiver, Sender, WeakSender,
};

mod chan;

pub(super) mod list;

mod unbounded;
pub use self::unbounded::{
    unbounded_channel, UnboundedReceiver, UnboundedSender, WeakUnboundedSender,
};

pub mod error;

/// The number of values a block can contain.
///
/// This value must be a power of 2. It also must be smaller than the number of
/// bits in `usize`.
#[cfg(all(target_pointer_width = "64", not(loom)))]
pub(crate) const BLOCK_CAP: usize = 32;

#[cfg(all(not(target_pointer_width = "64"), not(loom)))]
pub(crate) const BLOCK_CAP: usize = 16;

#[cfg(loom)]
pub(crate) const BLOCK_CAP: usize = 2;

#[cfg(all(test, not(loom)))]
mod verification_tests {
    use super::{channel, error::TryRecvError, unbounded_channel};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::task::{Context, Poll, Waker};

    #[test]
    fn mpsc_poll_recv_pending_then_value_restores_capacity() {
        let (tx, mut rx) = channel(1);
        let mut cx = Context::from_waker(Waker::noop());

        assert_eq!(rx.poll_recv(&mut cx), Poll::Pending);
        tx.try_send(17_u64).unwrap();
        assert_eq!(tx.capacity(), 0);
        assert_eq!(rx.poll_recv(&mut cx), Poll::Ready(Some(17)));
        assert_eq!(tx.capacity(), 1);
    }

    #[test]
    fn mpsc_poll_recv_held_permit_send_then_value_then_closed() {
        let (tx, mut rx) = channel(1);
        let permit = tx.try_reserve().unwrap();
        let mut cx = Context::from_waker(Waker::noop());

        rx.close();
        assert_eq!(rx.poll_recv(&mut cx), Poll::Pending);
        permit.send(23_u64);
        assert_eq!(rx.poll_recv(&mut cx), Poll::Ready(Some(23)));
        assert_eq!(rx.poll_recv(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn mpsc_poll_recv_held_permit_drop_then_closed() {
        let (tx, mut rx) = channel::<u64>(1);
        let permit = tx.try_reserve().unwrap();
        let mut cx = Context::from_waker(Waker::noop());

        rx.close();
        assert_eq!(rx.poll_recv(&mut cx), Poll::Pending);
        drop(permit);
        assert_eq!(tx.capacity(), 1);
        assert_eq!(rx.poll_recv(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn mpsc_poll_recv_many_exact_batch_and_limit_zero() {
        let (tx, mut rx) = channel(3);
        let mut cx = Context::from_waker(Waker::noop());
        let mut buffer = vec![5_u64];

        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 0), Poll::Ready(0));
        assert_eq!(buffer, vec![5]);

        tx.try_send(7).unwrap();
        tx.try_send(11).unwrap();
        assert_eq!(tx.capacity(), 1);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 3), Poll::Ready(2));
        assert_eq!(buffer, vec![5, 7, 11]);
        assert_eq!(tx.capacity(), 3);
    }

    #[test]
    fn mpsc_poll_recv_many_preallocated_bounded_branches() {
        let (tx, mut rx) = channel(4);
        let mut cx = Context::from_waker(Waker::noop());
        let mut buffer = Vec::with_capacity(6);
        buffer.push(5_u64);
        let allocation_capacity = buffer.capacity();

        tx.try_send(7).unwrap();
        tx.try_send(11).unwrap();
        tx.try_send(13).unwrap();
        let held = tx.try_reserve().unwrap();

        assert!(buffer.capacity() - buffer.len() >= 2);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 2), Poll::Ready(2));
        assert_eq!(buffer, vec![5, 7, 11]);
        assert_eq!(buffer.capacity(), allocation_capacity);
        assert_eq!(tx.capacity(), 2);

        rx.close();
        assert!(buffer.capacity() - buffer.len() >= 1);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 1), Poll::Ready(1));
        assert_eq!(buffer, vec![5, 7, 11, 13]);
        assert_eq!(buffer.capacity(), allocation_capacity);
        assert!(buffer.capacity() - buffer.len() >= 1);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 1), Poll::Pending);

        drop(held);
        assert!(buffer.capacity() - buffer.len() >= 1);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 1), Poll::Ready(0));
        assert_eq!(buffer.capacity(), allocation_capacity);
    }

    #[test]
    fn mpsc_poll_recv_many_preallocated_unbounded_branches() {
        let (tx, mut rx) = unbounded_channel();
        let mut cx = Context::from_waker(Waker::noop());
        let mut buffer = Vec::with_capacity(6);
        buffer.push(5_u64);
        let allocation_capacity = buffer.capacity();

        tx.send(7).unwrap();
        tx.send(11).unwrap();
        tx.send(13).unwrap();
        drop(tx);

        assert!(buffer.capacity() - buffer.len() >= 2);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 2), Poll::Ready(2));
        assert_eq!(buffer, vec![5, 7, 11]);
        assert_eq!(buffer.capacity(), allocation_capacity);
        assert!(buffer.capacity() - buffer.len() >= 2);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 2), Poll::Ready(1));
        assert_eq!(buffer, vec![5, 7, 11, 13]);
        assert_eq!(buffer.capacity(), allocation_capacity);
        assert!(buffer.capacity() - buffer.len() >= 1);
        assert_eq!(rx.poll_recv_many(&mut cx, &mut buffer, 1), Poll::Ready(0));
        assert_eq!(buffer.capacity(), allocation_capacity);
    }

    #[test]
    fn mpsc_poll_recv_many_push_panic_restores_bounded_capacity() {
        let (tx, mut rx) = channel(2);
        let mut cx = Context::from_waker(Waker::noop());
        let mut nearly_full_zst = vec![(); usize::MAX - 1];

        tx.try_send(()).unwrap();
        tx.try_send(()).unwrap();
        assert_eq!(tx.capacity(), 0);

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = rx.poll_recv_many(&mut cx, &mut nearly_full_zst, 2);
        }));
        assert!(panic.is_err());
        assert_eq!(nearly_full_zst.len(), usize::MAX);
        assert!(rx.is_empty());
        assert_eq!(tx.capacity(), 2);
        tx.try_send(()).unwrap();
        tx.try_send(()).unwrap();
        assert_eq!(rx.try_recv(), Ok(()));
        assert_eq!(rx.try_recv(), Ok(()));
        assert_eq!(tx.capacity(), 2);

        rx.close();
        let mut fresh = Vec::new();
        assert_eq!(rx.poll_recv_many(&mut cx, &mut fresh, 1), Poll::Ready(0));
    }

    #[test]
    fn mpsc_poll_recv_many_push_panic_clears_unbounded_count() {
        let (tx, mut rx) = unbounded_channel();
        let mut cx = Context::from_waker(Waker::noop());
        let mut nearly_full_zst = vec![(); usize::MAX - 1];

        tx.send(()).unwrap();
        tx.send(()).unwrap();

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = rx.poll_recv_many(&mut cx, &mut nearly_full_zst, 2);
        }));
        assert!(panic.is_err());
        assert_eq!(nearly_full_zst.len(), usize::MAX);
        assert!(rx.is_empty());

        rx.close();
        let mut fresh = Vec::new();
        assert_eq!(rx.poll_recv_many(&mut cx, &mut fresh, 1), Poll::Ready(0));
    }

    #[test]
    fn mpsc_poll_recv_unbounded_pending_then_value() {
        let (tx, mut rx) = unbounded_channel();
        let mut cx = Context::from_waker(Waker::noop());

        assert_eq!(rx.poll_recv(&mut cx), Poll::Pending);
        tx.send(29_u64).unwrap();
        assert_eq!(rx.poll_recv(&mut cx), Poll::Ready(Some(29)));
    }

    #[test]
    fn mpsc_endpoint_counts_bounded_lifecycle_and_terminal_upgrade() {
        let (tx, mut rx) = channel::<u64>(1);
        let tx2 = tx.clone();
        let weak = tx.downgrade();
        let weak2 = weak.clone();

        assert_eq!(tx.strong_count(), 2);
        assert_eq!(rx.sender_strong_count(), 2);
        assert_eq!(tx.weak_count(), 2);
        assert_eq!(rx.sender_weak_count(), 2);

        drop(tx2);
        drop(weak2);
        assert_eq!(weak.strong_count(), 1);
        assert_eq!(weak.weak_count(), 1);

        drop(tx);
        assert_eq!(weak.strong_count(), 0);
        assert!(weak.upgrade().is_none());
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));

        drop(weak);
        assert_eq!(rx.sender_weak_count(), 0);
    }

    #[test]
    fn mpsc_endpoint_counts_unbounded_lifecycle_and_terminal_upgrade() {
        let (tx, mut rx) = unbounded_channel::<u64>();
        let tx2 = tx.clone();
        let weak = tx.downgrade();
        let weak2 = weak.clone();

        assert_eq!(tx.strong_count(), 2);
        assert_eq!(rx.sender_strong_count(), 2);
        assert_eq!(tx.weak_count(), 2);
        assert_eq!(rx.sender_weak_count(), 2);

        drop(tx2);
        drop(weak2);
        drop(tx);
        assert_eq!(weak.strong_count(), 0);
        assert_eq!(weak.weak_count(), 1);
        assert!(weak.upgrade().is_none());
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));

        drop(weak);
        assert_eq!(rx.sender_weak_count(), 0);
    }

    #[test]
    fn mpsc_endpoint_counts_owned_permit_keeps_upgrade_open() {
        let (tx, mut rx) = channel::<u64>(1);
        let weak = tx.downgrade();
        let permit = tx.try_reserve_owned().unwrap();

        assert_eq!(weak.strong_count(), 1);
        let upgraded = weak.upgrade().expect("owned permit retains its sender");
        assert_eq!(upgraded.strong_count(), 2);
        drop(upgraded);

        let tx = permit.release();
        assert_eq!(tx.strong_count(), 1);
        let permit = tx.try_reserve_owned().unwrap();
        let tx = permit.send(41);
        assert_eq!(tx.strong_count(), 1);
        assert_eq!(rx.try_recv(), Ok(41));

        let permit = tx.try_reserve_owned().unwrap();
        assert_eq!(weak.strong_count(), 1);
        drop(permit);
        assert_eq!(weak.strong_count(), 0);
        assert!(weak.upgrade().is_none());
        drop(rx);
    }

    #[test]
    fn mpsc_try_recv_public_branches_bounded() {
        let (tx, mut rx) = channel(1);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));

        tx.try_send(53_u64).unwrap();
        assert_eq!(tx.capacity(), 0);
        assert_eq!(rx.try_recv(), Ok(53));
        assert_eq!(tx.capacity(), 1);

        let permit = tx.try_reserve().unwrap();
        rx.close();
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
        drop(permit);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
    }

    #[test]
    fn mpsc_try_recv_public_branches_unbounded() {
        let (tx, mut rx) = unbounded_channel();
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
        tx.send(59_u64).unwrap();
        assert_eq!(rx.try_recv(), Ok(59));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
        drop(tx);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
    }
}
