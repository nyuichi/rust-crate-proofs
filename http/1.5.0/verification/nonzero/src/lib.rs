#![cfg_attr(creusot, feature(nonzero_internals))]
#![allow(dead_code, unexpected_cfgs)]

extern crate creusot_std;

use core::num::NonZeroU16;
#[allow(unused_imports)]
use creusot_std::prelude::{ensures, requires};

/// Checks construction and extraction against the shared `NonZeroU16` model.
#[ensures(match result {
    Some(output) => value@ != 0 && output@ == value@,
    None => value@ == 0,
})]
pub fn checked_nonzero_round_trip(value: u16) -> Option<u16> {
    match NonZeroU16::new(value) {
        Some(code) => Some(code.get()),
        None => None,
    }
}

/// Checks that equality on standard-library `NonZeroU16` follows its integer model.
#[requires(left@ != 0 && right@ != 0)]
#[ensures(result == (left@ == right@))]
pub fn nonzero_equality_matches_u16(left: u16, right: u16) -> bool {
    let left = unsafe { NonZeroU16::new_unchecked(left) };
    let right = unsafe { NonZeroU16::new_unchecked(right) };
    left == right
}

/// Checks that ordering on standard-library `NonZeroU16` follows its integer model.
#[requires(left@ != 0 && right@ != 0)]
#[ensures(result == (left@ < right@))]
pub fn nonzero_order_matches_u16(left: u16, right: u16) -> bool {
    let left = unsafe { NonZeroU16::new_unchecked(left) };
    let right = unsafe { NonZeroU16::new_unchecked(right) };
    left < right
}
