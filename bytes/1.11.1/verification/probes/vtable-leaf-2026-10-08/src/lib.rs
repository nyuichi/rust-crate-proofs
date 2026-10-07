//! Diagnostic translation probe for `Bytes::from(Vec<u8>)` vtable materialization.
//!
//! This source retains the production `Bytes`/`Vtable` field shapes and the exact
//! `Vtable` definition. The callback bodies and `BytesMut` are deliberately
//! minimal stand-ins: this probe tests type and record construction translation,
//! not the actual methods, callback semantics, ownership, or a Bytes theorem.
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

// The exact production field order/names and callback types are preserved. The
// callbacks are intentionally divergent stubs so this probe does not introduce
// callback behavior or recursively construct a Bytes value.
unsafe fn shared_clone(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Bytes {
    loop {}
}
unsafe fn shared_to_vec(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Vec<u8> {
    loop {}
}
unsafe fn shared_to_mut(_: &AtomicPtr<()>, _: *const u8, _: usize) -> BytesMut {
    loop {}
}
unsafe fn shared_is_unique(_: &AtomicPtr<()>) -> bool {
    loop {}
}
unsafe fn shared_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) {
    loop {}
}

static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};

#[cfg(feature = "helper")]
#[trusted]
#[requires(true)]
#[ensures(true)]
fn shared_vtable() -> &'static Vtable {
    &SHARED_VTABLE
}

/// Source-sliced field materialization from the shared branch of production
/// `impl From<Vec<u8>> for Bytes`, with the allocation setup abstracted into
/// parameters so this translation covers only the layout and record body.
#[cfg(feature = "baseline")]
pub fn from_vec_record(ptr: *const u8, len: usize, shared: *mut ()) -> Bytes {
    Bytes {
        ptr,
        len,
        data: AtomicPtr::new(shared),
        vtable: &SHARED_VTABLE,
    }
}

/// Same field materialization as `from_vec_record`, with only the vtable
/// reference routed through the true/true trusted leaf.
#[cfg(feature = "helper")]
pub fn from_vec_record(ptr: *const u8, len: usize, shared: *mut ()) -> Bytes {
    Bytes {
        ptr,
        len,
        data: AtomicPtr::new(shared),
        vtable: shared_vtable(),
    }
}
