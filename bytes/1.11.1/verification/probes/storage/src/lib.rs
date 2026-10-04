//! Standard-pointer foundation. This is not a proof of Bytes or its vtable/drop.
#![allow(unexpected_cfgs)]
use creusot_std::{ghost::perm::Perm, prelude::*};

/// An arbitrary initialized boxed slice passes through its real raw allocation.
/// Ownership comes from the input Box; no permission is fabricated by this probe.
#[requires(index@ < input@.len())]
#[ensures(result.0 == input@[index@])]
#[ensures(result.1@ == input@)]
pub fn read_and_recover(input: Box<[u8]>, index: usize) -> (u8, Box<[u8]>) {
    let (pointer, permission) = Perm::from_box(input);
    let bytes = unsafe { Perm::as_ref(pointer, ghost! { &**permission }) };
    let value = bytes[index];
    let recovered = unsafe { Perm::to_box(pointer, permission) };
    (value, recovered)
}

/// The split borrows preserve the values and whole-allocation ownership.
/// Independent BytesMut owners and partially initialized capacity are not modeled here.
#[requires(at@ <= input@.len())]
#[ensures(result@ == input@)]
pub fn split_borrows_and_recover(input: Box<[u8]>, at: usize) -> Box<[u8]> {
    let (pointer, permission) = Perm::from_box(input);
    let split_at = snapshot! { at@ }.into_ghost();
    ghost! {
        let (left, right) = permission.split_at(*split_at);
        proof_assert!(left.val()@.len() == at@);
        proof_assert!(right.val()@.len() == permission.val()@.len() - at@);
    };
    unsafe { Perm::to_box(pointer, permission) }
}

#[cfg(feature = "wrong_byte")]
#[requires(index@ < input@.len())]
#[ensures(result@ == (input@[index@])@ + 1)]
pub fn wrong_byte(input: Box<[u8]>, index: usize) -> u8 {
    read_and_recover(input, index).0
}

#[cfg(feature = "missing_ownership")]
pub unsafe fn recover_without_ownership(pointer: *mut [u8]) -> Box<[u8]> {
    // This call must generate an unprovable false precondition, not succeed.
    Box::from_raw(pointer)
}
