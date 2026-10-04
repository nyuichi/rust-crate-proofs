//! Creusot 0.13 probe for pointer tagging and metadata addresses.
#![allow(unexpected_cfgs)]

use creusot_std::prelude::*;
#[cfg(feature = "negative_forged_dereference")]
use creusot_std::ghost::perm::Perm;

#[path = "../../../../src/provenance_specs.rs"]
mod provenance_specs;

const KIND_MASK: usize = 1;
const KIND_VEC: usize = 1;

/// Tag an allocation-derived pointer without discarding its runtime provenance.
#[requires(ptr.addr_logic() & KIND_MASK == 0usize)]
#[ensures(result.addr_logic() == ptr.addr_logic() | KIND_VEC)]
pub fn tag_vec_ptr(ptr: *mut u8) -> *mut u8 {
    let addr = provenance_specs::pointer_addr(ptr) | KIND_VEC;
    provenance_specs::pointer_with_address(ptr, addr)
}

/// Clear the metadata bit by offsetting from the tagged pointer.
#[requires(ptr.addr_logic() & KIND_MASK == KIND_VEC)]
#[ensures(result.addr_logic() == ptr.addr_logic() & !KIND_MASK)]
pub fn untag_vec_ptr(ptr: *mut u8) -> *mut u8 {
    let addr = provenance_specs::pointer_addr(ptr) & !KIND_MASK;
    provenance_specs::pointer_with_address(ptr, addr)
}

/// Setting and clearing the kind bit restores the original numerical address.
#[requires(ptr.addr_logic() & KIND_MASK == 0usize)]
#[ensures(result.addr_logic() == ptr.addr_logic())]
#[bitwise_proof]
pub fn tag_untag_address_roundtrip(ptr: *mut u8) -> *mut u8 {
    untag_vec_ptr(tag_vec_ptr(ptr))
}

/// Null-derived metadata pointers retain the requested integer address.
#[ensures(result == addr)]
pub fn metadata_pointer_has_address(addr: usize) -> usize {
    provenance_specs::pointer_addr(provenance_specs::metadata_pointer(addr))
}

/// Try to read through a null-derived pointer whose address aliases a live
/// reference. The probe must fail at `Perm::as_ref`'s ownership precondition:
/// matching a numeric address must not create allocation permission.
#[cfg(feature = "negative_forged_dereference")]
#[ensures(result == *value)]
pub fn forged_metadata_reborrow(value: &u8) -> u8 {
    let (real_ptr, ownership) = Perm::from_ref(value);
    let fake_ptr = provenance_specs::metadata_pointer(
        provenance_specs::pointer_addr(real_ptr),
    );
    unsafe { *Perm::as_ref(fake_ptr.cast_const(), ownership) }
}
