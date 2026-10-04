#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/chain_ops.rs"]
mod chain_ops;

#[ensures(result@ == if a@ + b@ <= usize::MAX@ { a@ + b@ } else { usize::MAX@ })]
pub fn sum_exact(a: usize, b: usize) -> usize {
    chain_ops::saturating_sum(a, b)
}

#[ensures(result.0@ == if a_remaining@ <= count@ { a_remaining@ } else { count@ })]
#[ensures(result.1@ == if a_remaining@ < count@ { count@ - a_remaining@ } else { 0 })]
#[ensures(result.0@ + result.1@ == count@)]
pub fn split_exact(a_remaining: usize, count: usize) -> (usize, usize) {
    chain_ops::split_count(a_remaining, count)
}

pub fn boundary_cases() {
    assert!(chain_ops::saturating_sum(2, 3) == 5);
    assert!(chain_ops::saturating_sum(usize::MAX, 0) == usize::MAX);
    assert!(chain_ops::saturating_sum(usize::MAX, 1) == usize::MAX);
    assert!(chain_ops::split_count(0, 0) == (0, 0));
    assert!(chain_ops::split_count(3, 2) == (2, 0));
    assert!(chain_ops::split_count(2, 2) == (2, 0));
    assert!(chain_ops::split_count(1, 2) == (1, 1));
    assert!(chain_ops::split_count(0, usize::MAX) == (0, usize::MAX));
}

#[cfg(feature = "wrong_overflow")]
#[requires(a@ == usize::MAX@)]
#[requires(b@ == 1)]
#[ensures(result@ == 0)]
pub fn wrong_overflow(a: usize, b: usize) -> usize {
    chain_ops::saturating_sum(a, b)
}

#[cfg(feature = "wrong_split")]
#[requires(a_remaining@ == 1)]
#[requires(count@ == 2)]
#[ensures(result.0@ == 2)]
pub fn wrong_split(a_remaining: usize, count: usize) -> (usize, usize) {
    chain_ops::split_count(a_remaining, count)
}
