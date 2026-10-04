#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;

#[path = "../../../../src/slice_ops.rs"]
mod slice_ops;

/// Checks an exact suffix after a valid advance.
#[requires(count@ <= input@.len())]
#[ensures((^input)@ == input@[count@..])]
pub fn advance_exact(input: &mut &[u8], count: usize) {
    slice_ops::advance_slice(input, count);
}

/// Checks that copying consumes and writes the exact source prefix.
#[requires(dst@.len() <= input@.len())]
#[ensures((^dst)@ == input@[0..dst@.len()])]
#[ensures((^input)@ == input@[dst@.len()..])]
pub fn copy_exact(input: &mut &[u8], dst: &mut [u8]) {
    slice_ops::copy_to_slice(input, dst);
}

#[cfg(feature = "wrong_content")]
#[requires(input@.len() == 1)]
#[requires(input@[0]@ == 1)]
#[requires(dst@.len() == 1)]
#[ensures((^dst)@[0]@ == 2)]
pub fn wrong_content(input: &mut &[u8], dst: &mut [u8]) {
    slice_ops::copy_to_slice(input, dst);
}

#[cfg(feature = "wrong_advance")]
#[requires(input@.len() == 2)]
#[ensures((^input)@ == input@)]
pub fn wrong_advance(input: &mut &[u8]) {
    slice_ops::advance_slice(input, 1);
}

#[cfg(feature = "out_of_bounds")]
#[requires(input@.len() == 0)]
pub fn out_of_bounds_advance(input: &mut &[u8]) {
    slice_ops::advance_slice(input, 1);
}
