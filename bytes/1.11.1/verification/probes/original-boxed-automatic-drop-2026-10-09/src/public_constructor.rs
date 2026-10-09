//! All-domain actual Bytes constructors with a proof-only representation sum.
//!
//! This module intentionally has no Clone/Drop invariant. Its proof state
//! describes only the selected source constructors and the direct byte read.

use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicPtr, AtomicUsize};
use creusot_std::{
    ghost::{
        GhostShared,
        perm::Perm,
        lifetime_logic::{EndBorrow, FullBorrow, Lifetime, LifetimeToken},
    },
    logic::{Id, Int, Seq},
    prelude::*,
    std::sync::{
        atomic::AtomicUsize as ModelAtomic,
        view::SyncView,
    },
};
use crate::{tag_specs, physical_projection, erased_call, boxed_alignment, field_event, lifecycle, pointer_event, provenance_specs, raw_vec,
    read_projection};

include!("../../../../src/bytes/shared_record.rs");
include!(concat!(env!("OUT_DIR"), "/public_records.rs"));

impl field_event::AtomicField for Shared {
    #[cfg_attr(creusot, ensures(field_event::atomic_model(result) == self.field_model()))]
    fn atomic_field(&self) -> &AtomicUsize { &self.ref_cnt }

    #[cfg(creusot)]
    #[logic(open(crate))]
    fn field_model(&self) -> ModelAtomic { field_event::atomic_model(&self.ref_cnt) }
}

/// Sum of the three native representations reached by the constructors in
/// this experiment. It is stored only in `Ghost<...>` on the extracted Bytes.
pub(crate) enum OriginalBytesProof {
    Shared(OriginalSharedProof),
    PromotableRaw(PromotableRawProof),
    Static(StaticBytesProof),
}

pub(crate) struct PromotableRawProof {
    base: raw_vec::BoundPtr,
    capacity: usize,
    expected: Snapshot<Seq<u8>>,
    capabilities: Ghost<(raw_vec::Recovery, raw_vec::PhysicalRegion)>,
    data_binding: Ghost<pointer_event::ReadOnlyPointer>,
}

pub(crate) struct StaticBytesProof {
    source: &'static [u8],
    source_pointer: *const u8,
    source_len: usize,
    source_permission: Ghost<&'static Perm<*const [u8]>>,
    data_binding: Ghost<pointer_event::ReadOnlyPointer>,
}

impl PromotableRawProof {
    #[logic(prophetic)]
    fn valid_for(
        self,
        ptr: *const u8,
        len: usize,
        data: creusot_std::std::sync::atomic::AtomicPtr<()>,
        vtable: &'static Vtable,
    ) -> bool {
        let p = self;
        pearlite! {
            p.base.invariant() &&
            p.capabilities.inner_logic().0.invariant() &&
            p.capabilities.inner_logic().1.invariant() &&
            p.base@ == Some((p.capabilities.inner_logic().0.namespace(), p.capacity@, 0int)) &&
            p.capacity@ == p.capabilities.inner_logic().0.capacity() &&
            p.capacity@ == p.capabilities.inner_logic().1.capacity() &&
            p.capacity@ == len@ &&
            (*p.expected).len() == len@ &&
            len@ > 0 &&
            ptr == p.base.raw_pointer() as *const u8 &&
            p.base.current_address() == ptr.addr_logic()@ &&
            p.capabilities.inner_logic().0.namespace() == p.capabilities.inner_logic().1.namespace() &&
            p.capabilities.inner_logic().1.resource_id() == p.capabilities.inner_logic().0.namespace() &&
            p.capabilities.inner_logic().1.lo() == 0 &&
            p.capabilities.inner_logic().1.hi() == p.capacity@ &&
            (forall<i: Int> 0 <= i && i < len@ ==>
                p.capabilities.inner_logic().1.slot(i) == Some(Some((*p.expected)[i]))) &&
            p.data_binding.inner_logic().model() == data &&
            p.data_binding.inner_logic().value().addr_logic() & 1usize == 1usize &&
            (
                (p.base.raw_pointer().addr_logic() & 1usize == 0usize &&
                    vtable == promotable_even_table() &&
                    p.data_binding.inner_logic().value() == tag_specs::tagged_data(p.base.raw_pointer())) ||
                (p.base.raw_pointer().addr_logic() & 1usize != 0usize &&
                    vtable == promotable_odd_table() &&
                    p.data_binding.inner_logic().value() == p.base.raw_pointer() as *mut ())
            )
        }
    }
}

impl StaticBytesProof {
    #[logic(prophetic)]
    fn valid_for(
        self,
        ptr: *const u8,
        len: usize,
        data: creusot_std::std::sync::atomic::AtomicPtr<()>,
    ) -> bool {
        let p = self;
        pearlite! {
            p.source_pointer == ptr &&
            p.source_len == len &&
            p.source_permission.inner_logic().val() == p.source &&
            *p.source_permission.inner_logic().ward() as *const u8 == ptr &&
            len@ == p.source_permission.inner_logic().val()@.len() &&
            p.source@ == p.source_permission.inner_logic().val()@ &&
            p.data_binding.inner_logic().model() == data &&
            p.data_binding.inner_logic().value() == static_null_data()
        }
    }
}

#[logic(opaque)]
fn shared_table() -> &'static Vtable { dead }
#[logic(opaque)]
fn promotable_even_table() -> &'static Vtable { dead }
#[logic(opaque)]
fn promotable_odd_table() -> &'static Vtable { dead }
#[logic(opaque)]
fn static_table() -> &'static Vtable { dead }
#[logic(opaque)]
fn static_null_data() -> *mut () { dead }

// Closed native-table reifications. `extract_constructor.py` checks the exact
// static declaration and constructor branch for each identity. These narrow
// generic erasure contracts do not assert any bytes ownership or dispatch law.
#[trusted]
#[ensures(result == shared_table())]
fn shared_table_reification() -> &'static Vtable {
    #[cfg(creusot)]
    { unreachable!("closed Shared Vtable reification") }
    #[cfg(not(creusot))]
    { original_shared_table_native() }
}

#[trusted]
#[ensures(result == promotable_even_table())]
fn promotable_even_table_reification() -> &'static Vtable {
    #[cfg(creusot)]
    { unreachable!("closed even promotable Vtable reification") }
    #[cfg(not(creusot))]
    { &PROMOTABLE_EVEN_VTABLE }
}

#[trusted]
#[ensures(result == promotable_odd_table())]
fn promotable_odd_table_reification() -> &'static Vtable {
    #[cfg(creusot)]
    { unreachable!("closed odd promotable Vtable reification") }
    #[cfg(not(creusot))]
    { &PROMOTABLE_ODD_VTABLE }
}

#[trusted]
#[ensures(result == static_table())]
fn static_table_reification() -> &'static Vtable {
    #[cfg(creusot)]
    { unreachable!("closed static Vtable reification") }
    #[cfg(not(creusot))]
    { &STATIC_VTABLE }
}

/// Reify Rust's actual null data pointer as an opaque pointer model. This is
/// a generic pointer boundary; it carries no Bytes-specific law.
#[trusted]
#[ensures(result == static_null_data())]
fn static_null_pointer() -> *mut () {
    #[cfg(creusot)]
    { unreachable!("closed null-pointer reification") }
    #[cfg(not(creusot))]
    { core::ptr::null_mut() }
}


/// Generic pointer-tag representation boundary. The native source map retains
/// production `ptr_map` (`addr | KIND_VEC`); this contract covers only the
/// address-preserving tag operation, not allocation ownership or Bytes laws.
#[trusted]
#[requires(ptr.addr_logic() & 1usize == 0usize)]
#[ensures(result == tag_specs::tagged_data(ptr))]
#[ensures(result.addr_logic() == (ptr.addr_logic() | 1usize))]
fn promotable_tag_pointer(ptr: *mut u8) -> *mut () {
    #[cfg(creusot)]
    { unreachable!("generic pointer tag interpretation") }
    #[cfg(not(creusot))]
    { ptr_map(ptr, |address| address | 1).cast() }
}

impl Bytes {
    #[logic(prophetic)]
    pub(crate) fn original_bytes_valid(self) -> bool {
        match self.original_bytes.inner_logic() {
            OriginalBytesProof::Shared(proof) => {
                proof.valid_for(self.ptr, self.len, pointer_event::pointer_model(&self.data)) &&
                self.vtable == shared_table()
            }
            OriginalBytesProof::PromotableRaw(proof) => {
                proof.valid_for(self.ptr, self.len, pointer_event::pointer_model(&self.data), self.vtable)
            }
            OriginalBytesProof::Static(proof) => {
                proof.valid_for(self.ptr, self.len, pointer_event::pointer_model(&self.data)) &&
                self.vtable == static_table()
            }
        }
    }

    #[logic]
    pub(crate) fn original_bytes_content(self) -> Seq<u8> {
        match self.original_bytes.inner_logic() {
            OriginalBytesProof::Shared(proof) => {
                proof.invariant.inner_logic().val().public().3.3
            }
            OriginalBytesProof::PromotableRaw(proof) => *proof.expected,
            OriginalBytesProof::Static(proof) => pearlite! { proof.source@ },
        }
    }
}

// In this extracted crate the record is constructed only by the three
// included constructor branches. This invariant supplies AsRef's typed-self
// refinement; it is not used to verify Clone, Drop, or unextracted APIs.
impl creusot_std::invariant::Invariant for Bytes {
    #[logic(prophetic)]
    fn invariant(self) -> bool { self.original_bytes_valid() }
}

#[ensures(result.original_bytes_valid())]
#[ensures(result.original_bytes_content() == input@)]
fn original_bytes_from_vec(input: Vec<u8>) -> Bytes {
    let len = input.len();
    let capacity = input.capacity();
    if len < capacity {
        original_bytes_shared_from_vec(input)
    } else {
        proof_assert!(len == capacity);
        original_bytes_from_box(input.into_boxed_slice())
    }
}

#[ensures(result.original_bytes_valid())]
#[ensures(result.original_bytes_content() == input@)]
#[ensures(result.boxed_only())]
fn original_bytes_from_box(input: Box<[u8]>) -> Bytes {
    if input.len() == 0 {
        return Bytes::new();
    }

    let expected = snapshot!(input@);
    let (raw, len, capabilities) = raw_vec::detach_boxed_slice(input);
    let (base, capacity) = raw.into_bound_ptr_at_zero();
    proof_assert!(capacity == len);
    let ptr = base.as_ptr();
    let ptr_address = provenance_specs::pointer_addr(ptr);
    ghost! {tag_specs::classify_low_bit(ptr_address);tag_specs::tagged_low_bit(ptr_address);};
    let mut current = ghost! { SyncView::new().into_inner() };

    let (data_value, vtable) = if ptr_address & 1 == 0 {
        (promotable_tag_pointer(ptr), promotable_even_table_reification())
    } else {
        (ptr.cast::<()>(), promotable_odd_table_reification())
    };
    let (data, permission) = pointer_event::new_pointer(data_value, current.borrow_mut());
    let data_binding = pointer_event::bind_read_only(&data, data_value, permission);

    Bytes {
        ptr,
        len,
        data,
        vtable,
        original_bytes: ghost! {
            OriginalBytesProof::PromotableRaw(PromotableRawProof {
                base,
                capacity,
                expected,
                capabilities,
                data_binding,
            })
        },
    }
}

#[ensures(result.original_bytes_valid())]
#[ensures(result.original_bytes_content() == bytes@)]
#[ensures(result.static_repr())]
fn original_bytes_from_static(bytes: &'static [u8]) -> Bytes {
    use creusot_std::std::slice::SliceExt as _;
    let (source_pointer, source_permission) = bytes.as_ptr_perm();
    let source_len = bytes.len();
    let mut current = ghost! { SyncView::new().into_inner() };
    let null = static_null_pointer();
    let (data, permission) = pointer_event::new_pointer(null, current.borrow_mut());
    let data_binding = pointer_event::bind_read_only(&data, null, permission);
    Bytes {
        ptr: source_pointer,
        len: source_len,
        data,
        vtable: static_table_reification(),
        original_bytes: ghost! {
            OriginalBytesProof::Static(StaticBytesProof {
                source: bytes,
                source_pointer,
                source_len,
                source_permission,
                data_binding,
            })
        },
    }
}

#[requires(value.original_bytes_valid())]
#[ensures(result@ == value.original_bytes_content())]
fn original_bytes_as_slice(value: &Bytes) -> &[u8] {
    let expected = snapshot!(value.original_bytes_content());
    let lease = ghost! {
        match &*value.original_bytes {
            OriginalBytesProof::Shared(proof) => {
                let full: &FullBorrow<raw_vec::PhysicalRegion> = (*proof.physical).to_ref();
                let region = full.borrow(&proof.ticket.token);
                read_projection::ReadLease::Physical {
                    bound: &proof.bound,
                    region,
                }
            }
            OriginalBytesProof::PromotableRaw(proof) => {
                let capabilities = proof.capabilities.borrow();
                let capabilities: &(raw_vec::Recovery, raw_vec::PhysicalRegion) = *capabilities;
                read_projection::ReadLease::Physical {
                    bound: &proof.base,
                    region: &capabilities.1,
                }
            }
            OriginalBytesProof::Static(proof) => {
                let permission = proof.source_permission.borrow();
                let permission: &&'static Perm<*const [u8]> = *permission;
                read_projection::ReadLease::Slice(*permission)
            }
        }
    };
    unsafe { read_projection::borrow_any(value.ptr, value.len, lease, expected) }
}

// Original Shared source component and strong len<capacity contract. This is
// generated deterministically from the accepted Shared gate by the extractor.
include!(concat!(env!("OUT_DIR"), "/shared_constructor_component.rs"));

// Actual original constructors and read method, extracted from production.
include!(concat!(env!("OUT_DIR"), "/public_traits.rs"));

/// Representative all-domain call chain: the actual extracted `From<Vec>`
/// and `AsRef<[u8]>` implementations, followed by a slice copy. This does not
/// clone `Bytes` or rely on a Clone invariant for it.
#[ensures(result@ == input@)]
pub(crate) fn arbitrary_vec_from_then_read(input: Vec<u8>) -> Vec<u8> {
    let bytes = Bytes::from(input);
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}

/// All-domain counterpart for the extracted actual `From<Box<[u8]>>`,
/// including both native pointer-alignment table branches and empty fallback.
#[ensures(result@ == input@)]
pub(crate) fn arbitrary_box_from_then_read(input: Box<[u8]>) -> Vec<u8> {
    let bytes = Bytes::from(input);
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}

/// Direct empty/static route through the actual `Bytes::new` source body.
#[ensures(result@.len() == 0)]
pub(crate) fn empty_new_then_read() -> Vec<u8> {
    let bytes = Bytes::new();
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}

/// Concrete branch-shaped drivers: zero-length with spare capacity exercises
/// the Shared arm, and nonempty exact-capacity input exercises Vec-to-Box.
/// These are path examples only, not allocation-performance claims.
#[requires(input@.len() == 0 && creusot_std::std::vec::capacity_model(input) > 0)]
#[ensures(result@.len() == 0)]
pub(crate) fn zero_len_with_spare_capacity_from_then_read(input: Vec<u8>) -> Vec<u8> {
    let bytes = Bytes::from(input);
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}

#[requires(input@.len() > 0 && creusot_std::std::vec::capacity_model(input) == input@.len())]
#[ensures(result@ == input@)]
pub(crate) fn nonempty_full_capacity_from_then_read(input: Vec<u8>) -> Vec<u8> {
    let bytes = Bytes::from(input);
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}

/// Deliberate negative control: a raw Box constructor with the wrong native
/// table must not satisfy the representation-sum invariant.
#[cfg(feature = "negative_wrong_selected_vtable")]
#[requires(input@.len() > 0)]
#[ensures(result.original_bytes_valid())]
pub(crate) fn reject_wrong_selected_vtable(input: Box<[u8]>) -> Bytes {
    let mut bytes = original_bytes_from_box(input);
    bytes.vtable = shared_table_reification();
    bytes
}

/// Deliberate negative control: changing the actual extent after construction
/// must prevent an exact AsRef read from refining the original Box contents.
#[cfg(feature = "negative_wrong_constructor_content")]
#[requires(input@.len() > 0)]
#[ensures(result@ == input@)]
pub(crate) fn reject_wrong_constructor_extent(input: Box<[u8]>) -> Vec<u8> {
    let mut bytes = original_bytes_from_box(input);
    bytes.len = 0;
    <Bytes as AsRef<[u8]>>::as_ref(&bytes).to_vec()
}


include!("boxed_drop.rs");
include!("../generated/boxed_client.rs");
