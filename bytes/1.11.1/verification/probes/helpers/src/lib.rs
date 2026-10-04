#![allow(unexpected_cfgs)]
use creusot_std::prelude::*;
#[path = "../../../../src/arithmetic.rs"]
mod arithmetic;
use arithmetic::{min_u64_usize, saturating_sub_usize_u64};

#[path = "../../../../src/std_specs.rs"]
mod std_specs;

#[ensures(result@ == if b@ <= a@ { a@ - b@ } else { 0 })]
pub fn subtract(a: usize, b: u64) -> usize {
    saturating_sub_usize_u64(a, b)
}
#[ensures(result@ == if a@ < b@ { a@ } else { b@ })]
pub fn minimum(a: u64, b: usize) -> usize {
    min_u64_usize(a, b)
}

pub fn positive_cases() {
    assert!(subtract(0, 1) == 0);
    assert!(subtract(5, 3) == 2);
    assert!(subtract(5, 8) == 0);
    assert!(subtract(usize::MAX, 0) == usize::MAX);
    assert!(minimum(0, usize::MAX) == 0);
    assert!(minimum(u64::MAX, 1) == 1);
}

#[cfg(feature = "wrong_postcondition")]
#[ensures(result@ == 1)]
pub fn wrong_postcondition() -> usize { subtract(0, 1) }

#[cfg(feature = "reachable_false")]
pub fn reachable_false() {
    let value = subtract(0, 1);
    assert!(value == 0);
    assert!(false);
}
