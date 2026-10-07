//! Source-sliced translation diagnostic for the original Bytes shared branch.
//! The proof getter is a trusted true/true abstraction; no API theorem is proved.
#![allow(unexpected_cfgs, dead_code)]

#[cfg(creusot)]
use creusot_std::prelude::*;
use core::sync::atomic::{AtomicPtr, AtomicUsize};

#[path = "../../../../../src/provenance_specs.rs"]
mod provenance_specs;

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

/// Source-sliced shared-branch allocation/record body from production
/// `Bytes::from(Vec<u8>)`. The retained alignment assertion uses the existing
/// address-only helper, and the vtable field uses the true/true getter.
#[cfg_attr(creusot, ensures(result.ptr == ptr && result.len == len))]
pub fn from_vec_shared_branch(ptr: *mut u8, len: usize, cap: usize) -> Bytes {
    let shared = Box::new(Shared {
        buf: ptr,
        cap,
        ref_cnt: AtomicUsize::new(1),
    });

    let shared = Box::into_raw(shared);
    // The pointer should be aligned, so this assert should
    // always succeed.
    debug_assert!(
        0 == (crate::provenance_specs::pointer_addr(shared) & KIND_MASK),
        "internal: Box<Shared> should have an aligned pointer",
    );
    Bytes {
        ptr,
        len,
        data: AtomicPtr::new(shared as _),
        vtable: shared_vtable(),
    }
}
