//! Second, translation-only vtable-leaf diagnostic.
//!
//! This case changes only the getter boundary across cfgs: proof cfg has a
//! true/true trusted foreign getter and contains no SHARED_VTABLE static; native
//! cfg retains the local table initializer and getter returning its address.
//! The probe still covers a source-sliced Bytes record field only.
#![allow(unexpected_cfgs, dead_code)]

#[cfg(creusot)]
use creusot_std::prelude::*;
use core::sync::atomic::AtomicPtr;

pub struct Bytes {
    ptr: *const u8,
    len: usize,
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

// Native-only stand-ins retain the production callback signatures and exact
// table initializer, but do not model callback behavior.
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

// In proof cfg this declaration has no local body or static initializer. Its
// only contract is true/true; it supplies no table identity or field facts.
#[cfg(creusot)]
#[trusted]
#[requires(true)]
#[ensures(true)]
extern "Rust" fn shared_vtable() -> &'static Vtable {
    loop {}
}

// Native cfg keeps the runtime implementation and real static reference.
#[cfg(not(creusot))]
fn shared_vtable() -> &'static Vtable {
    &SHARED_VTABLE
}

/// Source-sliced field materialization from the shared branch of production
/// `impl From<Vec<u8>> for Bytes`; the allocation setup is represented by args.
pub fn from_vec_record(ptr: *const u8, len: usize, shared: *mut ()) -> Bytes {
    Bytes {
        ptr,
        len,
        data: AtomicPtr::new(shared),
        vtable: shared_vtable(),
    }
}
