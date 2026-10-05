//! Restricted proof gate for actual Shared control allocation and explicit release.
#![allow(unexpected_cfgs)]
#![recursion_limit = "512"]
extern crate alloc;
#[cfg(not(creusot))]
#[path = "../../../../src/allocation_ops.rs"]
mod allocation_ops;
#[path = "../../../../src/storage_ops.rs"]
mod storage_ops;
#[path = "../../../../src/capacity_ops.rs"]
mod capacity_ops;
#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../src/ownership_proof/owned_region.rs"]
mod owned_region;
#[path = "../../../../src/ownership_proof/raw_vec.rs"]
mod raw_vec;
#[path = "../../../../src/ownership_proof/sequential_counter.rs"]
mod sequential_counter;
#[path = "../../../../src/ownership_proof/shared_protocol.rs"]
mod shared_protocol;
#[path = "../../../../src/ownership_proof/bound_ptr.rs"]
mod bound_ptr;
#[path = "../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

pub(crate) mod ownership_proof {
    pub(crate) use crate::{owned_region, raw_vec, sequential_counter, shared_protocol, bound_ptr, boxed_alignment};
}
mod actual { include!(concat!(env!("OUT_DIR"), "/actual_split.rs")); }
use creusot_std::prelude::*;
#[requires(split@ <= input@.len())]
pub fn left_then_right(input: Vec<u8>, split: usize) {
    actual::BytesMut::proof_split_then_release(input, split, false);
}
#[requires(split@ <= input@.len())]
pub fn right_then_left(input: Vec<u8>, split: usize) {
    actual::BytesMut::proof_split_then_release(input, split, true);
}

#[requires(0 < split@ && split@ < input@.len())]
pub fn mutate_both(input: Vec<u8>, split: usize, left: u8, right: u8, right_first: bool) {
    actual::BytesMut::proof_mutate_split_then_release(input, split, left, right, right_first);
}
#[requires(split@ <= input@.len())]
pub fn access_both(input: Vec<u8>, split: usize, right_first: bool) {
    actual::BytesMut::proof_access_split_then_release(input, split, right_first);
}

/// Physical-boundary negative: a B1 spare slot has no initialized byte value.
#[cfg(feature = "negative_unknown_access")]
pub fn reject_unknown_bound_access() {
    let input: Vec<u8> = Vec::with_capacity(4);
    let (bound, len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    assert!(len == 0);
    let mut region = ghost! { capabilities.into_inner().1 };
    if capacity > 0 {
        proof_assert!(region.slot(0int) == Some(None));
        let _forbidden = unsafe { raw_vec::borrow_bound_mut(&bound, 1, region.borrow_mut()) };
    }
}

/// Native-capacity bounded actual split_off, mutable views and both release orders.
pub fn split_off_both(input: Vec<u8>, at: usize, right_first: bool) {
    actual::BytesMut::proof_split_off_then_release(input, at, right_first);
}

/// Shrinking length retains all owned slots for subsequent explicit retirement.
pub fn shrink_split_off(input: Vec<u8>, at: usize, keep: usize, right_first: bool) {
    actual::BytesMut::proof_shrink_split_off_then_release(input, at, keep, right_first);
}

/// Advance interior views while retaining full regions for explicit retirement.
pub fn advance_split_off(input: Vec<u8>, at: usize, left_count: usize, right_count: usize, right_first: bool) {
    actual::BytesMut::proof_advance_split_off_then_release(input, at, left_count, right_count, right_first);
}

/// Initialize and publish one spare byte per side where capacity permits.
pub fn initialize_split_spare(input: Vec<u8>, at: usize, right_first: bool) {
    actual::BytesMut::proof_initialize_split_spare(input, at, right_first);
}
/// Re-uninitialize a truncated byte, rewrite it, then publish it again.
#[requires(input@.len() >= 1)]
pub fn reinitialize_split_prefix(input: Vec<u8>, right_first: bool) {
    actual::BytesMut::proof_reinitialize_split_prefix(input, right_first);
}

/// Shared reads remain valid while a disjoint registered handle is mutated.
pub fn readonly_split(input: Vec<u8>, at: usize, right_first: bool) {
    actual::BytesMut::proof_readonly_split(input, at, right_first);
}

/// Read-only physical access still requires an initialized byte, not just capacity.
#[cfg(feature = "negative_unknown_read")]
pub fn reject_unknown_read() {
    let input: Vec<u8> = Vec::with_capacity(4);
    let (bound, len, capacity, capabilities) = bound_ptr::detach_bound_vec(input);
    assert!(len == 0);
    let region = ghost! { capabilities.into_inner().1 };
    if capacity > 0 {
        proof_assert!(region.slot(0int) == Some(None));
        let _forbidden = unsafe { raw_vec::borrow_bound(&bound, 1, region.borrow()) };
    }
}

/// Unpromoted unique ownership supports read/write/spare publication and cleanup.
pub fn unique_access(input: Vec<u8>, keep: usize, value: u8) {
    actual::BytesMut::proof_unique_access(input, keep, value);
}

/// Capacity-bounded actual resize and append on unpromoted unique ownership.
pub fn noalloc_unique(input: Vec<u8>, new_len: usize, value: u8, extend: &[u8]) {
    actual::BytesMut::proof_noalloc_unique(input, new_len, value, extend);
}
/// Resize and append to the retained right split_to handle, then release both.
pub fn noalloc_split(input: Vec<u8>, split: usize, new_len: usize, value: u8, extend: &[u8], left_first: bool) {
    actual::BytesMut::proof_noalloc_split(input, split, new_len, value, extend, left_first);
}

/// Repeated unique view advancement retains and releases the original allocation.
pub fn unique_advance(input: Vec<u8>, first: usize, second: usize, value: u8) {
    actual::BytesMut::proof_unique_advance(input, first, second, value);
}
