//! Body-proved callers of the explicitly trusted sequential native atomic bridge.
#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
use sequential_counter::SequentialCounter;

#[requires(initial@ + add@ <= usize::MAX@)]
#[requires(sub@ <= initial@ + add@)]
#[ensures(result@ == initial@ + add@ - sub@)]
pub fn scalar_transitions(initial: usize, add: usize, sub: usize) -> usize {
    let (counter, mut own) = SequentialCounter::new(initial);
    let before_add = counter.fetch_add_relaxed(add, own.borrow_mut());
    proof_assert!(before_add == initial);
    let before_sub = counter.fetch_sub_release(sub, own.borrow_mut());
    proof_assert!(before_sub@ == initial@ + add@);
    counter.load_acquire(own.borrow())
}

/// Mirrors two registered handles: two native Release decrements, then Acquire.
#[ensures(result@ == 0)]
pub fn two_handle_countdown() -> usize {
    let (counter, mut own) = SequentialCounter::new(1);
    let cloned = counter.fetch_add_relaxed(1, own.borrow_mut());
    assert!(cloned == 1);
    let first = counter.fetch_sub_release(1, own.borrow_mut());
    assert!(first == 2);
    let final_release = counter.fetch_sub_release(1, own.borrow_mut());
    assert!(final_release == 1);
    counter.load_acquire(own.borrow())
}

#[cfg(feature = "negative_wrong_identity")]
pub fn reject_wrong_identity() {
    let (first, _first_own) = SequentialCounter::new(1);
    let (_second, mut second_own) = SequentialCounter::new(1);
    // Equal scalar values confer no authority over the other atomic object.
    first.fetch_sub_release(1, second_own.borrow_mut());
}

#[cfg(feature = "negative_underflow")]
pub fn reject_underflow() {
    let (counter, mut own) = SequentialCounter::new(0);
    counter.fetch_sub_release(1, own.borrow_mut());
}
