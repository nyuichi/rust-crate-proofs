//! Unique subregion permissions for real `Box<[u8]>` allocations.
//!
//! This probe uses the standard `Perm::split_at_mut` contract and only the
//! permission returned by `Perm::from_box`. It establishes byte mutation and
//! recovery for a single allocation. It makes no claim about Bytes/BytesMut
//! aliasing, reference counts, or drop behavior.
#![allow(unexpected_cfgs)]

use creusot_std::{
    ghost::perm::Perm,
    prelude::*,
    std::ptr::PtrAddExt,
};

/// Mutate the last byte of each nonempty region of one uniquely owned box.
///
/// The regions are split at an arbitrary interior boundary. `add_live` derives
/// the right raw pointer from the live left region, and `slice_from_raw_parts_mut`
/// supplies the two slice metadata lengths expected by `Perm::as_mut`.
#[requires(input@.len() >= 2)]
#[requires(0 < split@ && split@ < input@.len())]
#[requires(left_value@ != right_value@)]
#[ensures(result@.len() == input@.len())]
#[ensures(result@[split@ - 1] == left_value)]
#[ensures(result@[split@] == right_value)]
#[ensures(forall<i> 0 <= i && i < input@.len() && i != split@ - 1 && i != split@ ==> result@[i] == input@[i])]
pub fn write_disjoint_regions(
    input: Box<[u8]>,
    split: usize,
    left_value: u8,
    right_value: u8,
) -> Box<[u8]> {
    let len = input.len();
    let (base, mut ownership) = Perm::from_box(input);
    let base_bytes = base as *mut u8;

    {
        let (left_ownership, right_ownership) = ghost! {
            (**ownership).split_at_mut(*Int::new(split as i128))
        }
        .split();

        // The live witness spans the prefix and includes its one-past pointer.
        let left_live = ghost! { left_ownership.live() };
        let right_bytes = unsafe { base_bytes.add_live(split, left_live) };

        let left_raw = core::ptr::slice_from_raw_parts_mut(base_bytes, split);
        let right_raw = core::ptr::slice_from_raw_parts_mut(right_bytes, len - split);

        let left = unsafe { Perm::as_mut(left_raw, left_ownership) };
        let right = unsafe { Perm::as_mut(right_raw, right_ownership) };

        left[split - 1] = left_value;
        right[0] = right_value;
    }

    unsafe { Perm::to_box(base, ownership) }
}

#[cfg(feature = "wrong_byte")]
#[requires(input@.len() >= 2)]
#[requires(0 < split@ && split@ < input@.len())]
#[requires(left_value@ != right_value@)]
#[ensures(result@[split@ - 1] == left_value)]
#[ensures(result@[split@] == right_value)]
#[ensures(result@[split@ - 1]@ == left_value@ + 1)]
pub fn wrong_byte(
    input: Box<[u8]>,
    split: usize,
    left_value: u8,
    right_value: u8,
) -> Box<[u8]> {
    write_disjoint_regions(input, split, left_value, right_value)
}

#[cfg(feature = "wrong_overlap")]
#[requires(input@.len() >= 2)]
#[requires(0 < split@ && split@ < input@.len())]
#[requires(left_value@ != right_value@)]
#[ensures(result@[split@ - 1] == left_value)]
#[ensures(result@[split@] == right_value)]
#[ensures(result@[split@ - 1] == right_value)]
pub fn wrong_overlap(
    input: Box<[u8]>,
    split: usize,
    left_value: u8,
    right_value: u8,
) -> Box<[u8]> {
    write_disjoint_regions(input, split, left_value, right_value)
}
