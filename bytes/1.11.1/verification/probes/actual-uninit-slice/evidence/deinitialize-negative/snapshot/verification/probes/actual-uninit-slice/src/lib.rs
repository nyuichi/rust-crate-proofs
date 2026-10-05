//! Exact-source probe for transparent `UninitSlice` casts and initialized writes.
#![allow(unexpected_cfgs)]

#[allow(unused_imports)]
use creusot_std::prelude::*;

pub mod buf {
    mod uninit_slice {
        include!(concat!(env!("OUT_DIR"), "/actual_uninit_slice.rs"));
    }

    pub use uninit_slice::UninitSlice;
}

pub use buf::UninitSlice;

/// Convert an initialized slice through the exact `From` implementation,
/// overwrite one slot, and verify the writeback to the original borrow.
#[cfg(creusot)]
#[requires(index@ < input@.len())]
#[ensures((^input)@[index@] == byte)]
#[ensures(forall<i> 0 <= i && i < input@.len() && i != index@ ==> (^input)@[i] == input@[i])]
pub fn write_through_initialized_slice(input: &mut [u8], index: usize, byte: u8) {
    let destination: &mut UninitSlice = input.into();
    destination.write_byte(index, byte);
}

/// Copy source bytes through the exact `From` implementation and body-proved
/// helper; every output slot must correspond to the source sequence.
#[cfg(creusot)]
#[requires(input@.len() == source@.len())]
#[ensures((^input)@ == source@)]
pub fn copy_through_initialized_slice(input: &mut [u8], source: &[u8]) {
    let destination: &mut UninitSlice = input.into();
    destination.copy_from_slice(source);
}

/// The exact source `From<&mut [MaybeUninit<u8>]>` path initializes all slots.
#[cfg(creusot)]
#[requires(input@.len() == source@.len())]
#[ensures(forall<i> 0 <= i && i < source@.len() ==> (^input)@[i]@ == Some(source@[i]))]
pub fn copy_through_uninitialized_slice(
    input: &mut [core::mem::MaybeUninit<u8>],
    source: &[u8],
) {
    let destination: &mut UninitSlice = input.into();
    destination.copy_from_slice(source);
}

/// Deliberately violate the unsafe projection's preservation obligation.
/// This negative control must fail its precondition or resulting postcondition.
#[cfg(all(creusot, feature = "bad_deinitialize"))]
#[requires(input@.len() == 1)]
#[requires(input@[0]@ != None)]
#[ensures((^input)@[0]@ == None)]
pub fn bad_deinitialize_initialized_byte(input: &mut [core::mem::MaybeUninit<u8>]) {
    let destination: &mut UninitSlice = input.into();
    (unsafe { destination.as_uninit_slice_mut() })[0] =
        core::mem::MaybeUninit::uninit();
}

/// This feature deliberately contradicts the exact copy model.
#[cfg(all(creusot, feature = "wrong_copy_model"))]
#[requires(input@.len() == source@.len())]
#[requires(source@.len() > 0)]
#[requires(source@[0]@ != 0)]
#[ensures((^input)@[0]@ == Some(0u8))]
pub fn wrong_copy_model(input: &mut [core::mem::MaybeUninit<u8>], source: &[u8]) {
    let destination: &mut UninitSlice = input.into();
    destination.copy_from_slice(source);
}

#[cfg(not(creusot))]
pub fn native_write_and_copy(input: &mut [u8], source: &[u8], index: usize, byte: u8) {
    let destination: &mut UninitSlice = input.into();
    destination.write_byte(index, byte);
    destination.copy_from_slice(source);
}
