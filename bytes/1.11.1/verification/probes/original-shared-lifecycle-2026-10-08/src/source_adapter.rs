//! Source-correspondent leaf for the original `bytes::Shared` allocation.
//!
//! `source_map.py` checks the exact production Shared fields, selected
//! `From<Vec<u8>>` branch, clone increment, and explicit cleanup bodies. Under
//! the default non-loom/non-extra-platforms configuration, the production
//! `loom::sync::atomic::AtomicUsize` alias is `core::sync::atomic::AtomicUsize`,
//! which is the field type used here. Public vtable dispatch and automatic Drop
//! are outside this leaf.

use alloc::boxed::Box;
use core::sync::atomic::{AtomicPtr, AtomicUsize};
use creusot_std::{ghost::perm::Perm, prelude::*};

use crate::{boxed_alignment, field_event, pointer_event, raw_vec};

/// Exact production field order and field types from bytes.rs::Shared.
pub(crate) struct Shared {
    pub(crate) buf: *mut u8,
    pub(crate) cap: usize,
    pub(crate) ref_cnt: AtomicUsize,
}

/// The actual `Bytes` field types and order from bytes.rs. The vtable value is
/// supplied by the caller because this leaf proves the concrete handle fields
/// and their pointer relation, while dynamic callback selection remains open.
/// Keeping `Vtable` opaque here prevents this adapter from suggesting that it
/// has verified the callback bodies or `SHARED_VTABLE` identity.
pub(crate) struct Bytes {
    pub(crate) ptr: *const u8,
    pub(crate) len: usize,
    pub(crate) data: AtomicPtr<()>,
    pub(crate) vtable: &'static Vtable,
}

pub(crate) struct Vtable {
    _opaque: (),
}

/// The constructor result keeps the B1 recovery and physical region, the real
/// typed Box permission for this Shared allocation, and the permission from
/// the same generic atomic constructor that initialized `Shared.ref_cnt`.
pub(crate) struct OriginalSharedHandle {
    pub(crate) bytes: Bytes,
    pub(crate) shared: *mut Shared,
    pub(crate) shared_owner: Ghost<Box<Perm<*const Shared>>>,
    pub(crate) count_permission: Ghost<Perm<creusot_std::std::sync::atomic::AtomicUsize>>,
    pub(crate) data_binding: Ghost<pointer_event::ReadOnlyPointer>,
    pub(crate) capabilities: Ghost<(raw_vec::Recovery, raw_vec::PhysicalRegion)>,
}

#[requires(input@.len() < creusot_std::std::vec::capacity_model(input))]
#[ensures(result.bytes.len@ == input@.len())]
#[ensures(result.bytes.ptr == result.shared_owner.val().buf as *const u8)]
#[ensures(result.shared_owner.val().cap@ == creusot_std::std::vec::capacity_model(input))]
#[ensures(*result.shared_owner.ward() == result.shared as *const Shared)]
#[ensures(*result.count_permission.ward() == field_event::atomic_model(&result.shared_owner.val().ref_cnt))]
#[ensures(result.data_binding.inner_logic().model() == pointer_event::pointer_model(&result.bytes.data))]
#[ensures(result.data_binding.inner_logic().value().addr_logic() == result.shared.addr_logic())]
#[ensures(result.bytes.vtable == vtable)]
#[ensures(result.capabilities.inner_logic().0.capacity() ==
    result.shared_owner.val().cap@)]
#[ensures(result.capabilities.inner_logic().1.capacity() ==
    result.shared_owner.val().cap@)]
#[ensures(result.capabilities.inner_logic().1.slot(result.bytes.len@) == Some(None))]
pub(crate) fn from_vec_spare_capacity(
    input: alloc::vec::Vec<u8>,
    vtable: &'static Vtable,
    mut current: Ghost<creusot_std::std::sync::view::SyncView>,
) -> OriginalSharedHandle {
    // This is the selected len < cap arm of bytes.rs::From<Vec<u8>>: B1 detaches
    // the actual Vec allocation, and the actual core atomic returned by the
    // generic field constructor is moved directly into the production-shaped
    // Shared record.
    let (raw, len, capabilities) = raw_vec::detach_vec(input);
    let (base, cap) = raw.into_bound_ptr_at_zero();
    let ptr = base.as_ptr();
    let (ref_cnt, count_permission) = field_event::new(1, current.borrow_mut());
    let boxed = Box::new(Shared {
        buf: ptr,
        cap,
        ref_cnt,
    });
    let (shared, shared_owner) = boxed_alignment::into_raw_aligned(boxed);
    let shared_addr = crate::provenance_specs::pointer_addr(shared);
    boxed_alignment::aligned_address_has_clear_low_bit(
        shared_addr,
        core::mem::align_of::<Shared>(),
    );
    debug_assert_eq!(shared_addr & 1, 0);
    let (data, data_permission) = pointer_event::new_pointer(shared.cast::<()>(), current.borrow_mut());
    let bytes = Bytes {
        ptr,
        len,
        data,
        vtable,
    };
    let data_binding = pointer_event::bind_read_only(&bytes.data, shared.cast::<()>(), data_permission);
    OriginalSharedHandle {
        bytes,
        shared,
        shared_owner,
        count_permission,
        data_binding,
        capabilities,
    }
}
