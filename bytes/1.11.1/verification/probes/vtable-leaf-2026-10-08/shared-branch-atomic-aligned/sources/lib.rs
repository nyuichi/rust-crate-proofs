//! Diagnostic for source-sliced Bytes shared-branch construction.
//! This is an interface experiment, not a replacement executable constructor.
#![allow(unexpected_cfgs, dead_code)]

extern crate alloc;

use alloc::boxed::Box;
use creusot_std::{ghost::perm::Perm, prelude::*};
use core::sync::atomic::{AtomicPtr, AtomicUsize};

#[path = "../../../../../src/provenance_specs.rs"]
mod provenance_specs;
#[path = "../../../../../src/ownership_proof/boxed_alignment.rs"]
mod boxed_alignment;

pub struct Bytes {
    ptr: *const u8,
    len: usize,
    // inlined "trait object"
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
}

pub struct BytesMut;

pub(crate) struct Vtable {
    /// fn(data, ptr, len)
    pub clone: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> Bytes,
    /// fn(data, ptr, len)
    ///
    /// `into_*` consumes the `Bytes`, returning the respective value.
    pub into_vec: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> Vec<u8>,
    pub into_mut: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> BytesMut,
    /// fn(data)
    pub is_unique: unsafe fn(&AtomicPtr<()>) -> bool,
    /// fn(data, ptr, len)
    pub drop: unsafe fn(&mut AtomicPtr<()>, *const u8, usize),
}

// This is the exact production Shared field shape. Its production Drop body is
// outside this source-sliced allocation-and-record construction diagnostic.
struct Shared {
    // Holds arguments to dealloc upon Drop, but otherwise doesn't use them
    buf: *mut u8,
    cap: usize,
    ref_cnt: AtomicUsize,
}

// Native-only callback stubs retain the production signatures so the table
// initializer compiles natively; they model no callback behavior.
#[cfg(not(creusot))]
unsafe fn shared_clone(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Bytes {
    loop {}
}
#[cfg(not(creusot))]
unsafe fn shared_to_vec(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Vec<u8> {
    loop {}
}
#[cfg(not(creusot))]
unsafe fn shared_to_mut(_: &AtomicPtr<()>, _: *const u8, _: usize) -> BytesMut {
    loop {}
}
#[cfg(not(creusot))]
unsafe fn shared_is_unique(_: &AtomicPtr<()>) -> bool {
    loop {}
}
#[cfg(not(creusot))]
unsafe fn shared_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) {
    loop {}
}

#[cfg(not(creusot))]
static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};

// The proof abstraction supplies only a valid static reference at the type
// level; true/true says nothing about which table or what its entries contain.
#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn shared_vtable() -> &'static Vtable {
    unimplemented!()
}

#[cfg(not(creusot))]
fn shared_vtable() -> &'static Vtable {
    &SHARED_VTABLE
}

const KIND_MASK: usize = 0b1;

// These proof-only normal-return bridges intentionally say nothing about the
// values stored in the native atomics. Native cfg executes the exact core
// constructors.
#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn atomic_usize_new(value: usize) -> AtomicUsize {
    unimplemented!()
}

#[cfg(not(creusot))]
fn atomic_usize_new(value: usize) -> AtomicUsize {
    AtomicUsize::new(value)
}

#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn atomic_ptr_new<T>(value: *mut T) -> AtomicPtr<T> {
    unimplemented!()
}

#[cfg(not(creusot))]
fn atomic_ptr_new<T>(value: *mut T) -> AtomicPtr<T> {
    AtomicPtr::new(value)
}

/// Diagnostic output keeps the allocation permission available to the
/// interface experiment. It is not a production Bytes API.
type SharedBranchOutput = (
    Bytes,
    *mut Shared,
    Ghost<Box<Perm<*const Shared>>>,
);

/// Source-sliced shared-branch allocation/record body from production
/// `Bytes::from(Vec<u8>)`. Only the generic aligned Box permission helper,
/// address-only numeric helper, and true/true vtable getter cross interfaces.
#[cfg_attr(creusot, ensures(result.0.ptr == ptr && result.0.len == len))]
#[cfg_attr(creusot, ensures(*result.2.ward() == result.1 as *const Shared))]
#[cfg_attr(creusot, ensures(result.2.val().buf == ptr))]
#[cfg_attr(creusot, ensures(result.2.val().cap == cap))]
fn from_vec_shared_branch(ptr: *mut u8, len: usize, cap: usize) -> SharedBranchOutput {
    let boxed = Box::new(Shared {
        buf: ptr,
        cap,
        ref_cnt: atomic_usize_new(1),
    });

    let (shared, shared_owner) = boxed_alignment::into_raw_aligned(boxed);
    // The pointer should be aligned, so this assert should
    // always succeed.
    let shared_addr = crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(
        shared_addr,
        core::mem::align_of::<Shared>(),
    );
    debug_assert!(
        0 == (shared_addr & KIND_MASK),
        "internal: Box<Shared> should have an aligned pointer",
    );
    let bytes = Bytes {
        ptr,
        len,
        data: atomic_ptr_new(shared as _),
        vtable: shared_vtable(),
    };
    (bytes, shared, shared_owner)
}
