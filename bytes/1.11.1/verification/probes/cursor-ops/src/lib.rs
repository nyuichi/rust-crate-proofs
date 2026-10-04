//! Proof probe for the arithmetic and slice helpers used by Cursor's Buf impl.
//! The byte sequence is always the actual input slice; no length model is trusted.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/arithmetic.rs"]
mod arithmetic;

#[path = "../../../../src/std_specs.rs"]
mod std_specs;

#[path = "../../../../src/cursor_ops.rs"]
mod cursor_ops;
use cursor_ops::{
    cursor_chunk, cursor_chunk_start, cursor_position_after_advance, cursor_remaining,
};

#[requires(cnt@ <= (if pos@ <= len@ { len@ - pos@ } else { 0 }))]
#[ensures(result@ == pos@ + cnt@)]
#[ensures(pos@ + cnt@ <= u64::MAX@)]
pub fn advance_position(len: usize, pos: u64, cnt: usize) -> u64 {
    cursor_position_after_advance(len, pos, cnt)
}

#[ensures(result@ == if pos@ <= len@ { len@ - pos@ } else { 0 })]
pub fn remaining(len: usize, pos: u64) -> usize {
    cursor_remaining(len, pos)
}

#[ensures(result@ == slice@.subsequence(
    if pos@ < slice@.len() { pos@ } else { slice@.len() },
    slice@.len()
))]
pub fn chunk<'a>(slice: &'a [u8], pos: u64) -> &'a [u8] {
    cursor_chunk(slice, pos)
}

/// Exercises EOF, beyond-EOF zero advances, and exact suffix selection.
pub fn positive_cases() {
    assert!(remaining(10, 2) == 8);
    assert!(remaining(10, 10) == 0);
    assert!(remaining(10, 20) == 0);

    assert!(cursor_chunk_start(0, 4) == 0);
    assert!(cursor_chunk_start(2, 4) == 2);
    assert!(cursor_chunk_start(4, 4) == 4);
    assert!(cursor_chunk_start(u64::MAX, 4) == 4);
    let bytes = [97u8, 98, 99, 100];
    let middle = chunk(&bytes, 2);
    let at_end = chunk(&bytes, 4);
    let beyond_end = chunk(&bytes, 20);
    proof_assert!(middle@ == bytes@.subsequence(2, 4));
    proof_assert!(at_end@ == bytes@.subsequence(4, 4));
    proof_assert!(beyond_end@ == bytes@.subsequence(4, 4));

    assert!(advance_position(10, 2, 8) == 10);
    assert!(advance_position(10, 10, 0) == 10);
    assert!(advance_position(10, 20, 0) == 20);
}

/// Negative control: the helper's actual position is one less than this claim.
#[cfg(feature = "wrong_position")]
#[requires(cnt@ <= (if pos@ <= len@ { len@ - pos@ } else { 0 }))]
#[ensures(result@ == pos@ + cnt@ + 1)]
pub fn wrong_position(len: usize, pos: u64, cnt: usize) -> u64 {
    cursor_position_after_advance(len, pos, cnt)
}

/// Negative control: raw u64 addition can overflow without Cursor's bound.
#[cfg(feature = "overflow_example")]
#[requires(pos@ == u64::MAX@)]
#[requires(cnt@ == 1)]
#[ensures(result@ == pos@ + cnt@)]
pub fn unguarded_overflow(pos: u64, cnt: u64) -> u64 {
    pos + cnt
}
