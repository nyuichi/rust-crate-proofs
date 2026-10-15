#[cfg(any(
    all(creusot, bytes_original_freeze_gate),
    all(creusot, bytes_original_unique_gate),
))]
use creusot_std::{prelude::*, ghost::perm::Perm};
#[cfg(all(creusot, bytes_original_unique_gate))]
use creusot_std::logic::ra::excl::Excl;
#[cfg(all(creusot, bytes_original_unique_gate))]
use creusot_std::logic::Id;
#[cfg(any(all(creusot, bytes_original_freeze_gate), all(creusot, bytes_original_unique_gate)))]
use crate::ownership_proof::raw_vec::{self, BoundPtr, PhysicalRegion, RawAllocation, Recovery, slot_known};
#[cfg(any(all(creusot, bytes_original_freeze_gate), all(creusot, bytes_original_unique_gate)))]
use crate::ownership_proof::boxed_alignment;
#[cfg(all(creusot, bytes_original_shared_gate))]
use creusot_std::prelude::*;
use core::mem::{self, ManuallyDrop};
use core::ops::{Deref, RangeBounds};
use core::ptr::NonNull;
use core::{cmp, fmt, hash, ptr, slice};

use alloc::{
    alloc::{dealloc, Layout},
    borrow::Borrow,
    boxed::Box,
    string::String,
    vec::Vec,
};

use crate::buf::IntoIter;
#[allow(unused)]
use crate::loom::sync::atomic::AtomicMut;
use crate::loom::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::ref_count_ops::RefCountCell;
use crate::{Buf, BytesMut};

pub(crate) type BytesDataCell = AtomicPtr<()>;

// Creusot-only exclusions below cover unrelated vtable/lifecycle, pointer-range, and generic
// adapter bodies. Their native APIs remain present; no Bytes-specific trusted contract replaces
// any excluded implementation.

include!("bytes/bytes_record.rs");
include!("bytes/vtable_record.rs");

#[cfg(all(creusot, bytes_original_freeze_gate))]
struct OriginalFrozenProof {
    base: BoundPtr,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
    shared: *mut Shared,
    shared_owner: Ghost<Box<Perm<*const Shared>>>,
}
#[cfg(all(creusot, bytes_original_freeze_gate))]
impl Bytes {
    #[logic(open(self), prophetic)]
    pub(crate) fn original_frozen_valid(self) -> bool {
        pearlite! {
            match self.original_frozen {
                None => false,
                Some(proof) => {
                    let caps = proof.capabilities.inner_logic();
                    proof.base.invariant() && caps.0.invariant() && caps.1.invariant() &&
                    proof.base@ == Some((caps.0.namespace(), caps.0.capacity(), 0int)) &&
                    self.ptr == proof.base.raw_pointer() as *const u8 &&
                    caps.1.capacity() == caps.0.capacity() &&
                    caps.1.namespace() == caps.0.namespace() &&
                    caps.1.resource_id() == caps.0.namespace() &&
                    caps.1.lo() == 0 && caps.1.hi() == caps.0.capacity() &&
                    self.len@ < caps.0.capacity() &&
                    *proof.shared_owner.ward() == proof.shared as *const Shared &&
                    proof.shared_owner.val().buf == proof.base.raw_pointer() &&
                    proof.shared_owner.val().cap@ == caps.0.capacity() &&
                    (forall<i:Int> 0 <= i && i < self.len@ ==> slot_known(caps.1.slot(i)))
                }
            }
        }
    }
    #[logic(open(self))]
    pub(crate) fn original_frozen_bytes(self) -> Seq<u8> {
        pearlite! { Seq::create(self.len@, |i:Int|
            self.original_frozen.unwrap_logic().capabilities.inner_logic().1.slot(i).unwrap_logic().unwrap_logic()) }
    }
    #[logic(open(self))]
    pub(crate) fn original_frozen_pointer(self) -> *const u8 { self.ptr }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
struct OriginalBoxedProof {
    base: BoundPtr,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
    contents: Snapshot<Seq<u8>>,
    even_pointer: bool,
    vtable: &'static Vtable,
    data_cell_permission: Ghost<creusot_std::ghost::perm::Perm<
        crate::loom::sync::atomic::NativePtr<()>,
    >>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
struct OriginalSpareSharedProof {
    raw: RawAllocation,
    base: BoundPtr,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
    contents: Snapshot<Seq<u8>>,
    shared: *mut Shared,
    shared_owner: Ghost<Box<Perm<*const Shared>>>,
    counter_owner: Ghost<(
        creusot_std::ghost::perm::Perm<crate::ref_count_ops::NativeCounter>,
        usize,
    )>,
    vtable: &'static Vtable,
    data_cell_permission: Ghost<creusot_std::ghost::perm::Perm<
        crate::loom::sync::atomic::NativePtr<()>,
    >>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl OriginalSpareSharedProof {
    /// The release carrier keeps the actual header Box, counter permission,
    /// and full B1 allocation tied to the same native Shared pointer.
    #[logic(open(self), prophetic)]
    fn release_valid(self, shared: *mut Shared, count: usize) -> bool {
        pearlite! {
            let caps = self.capabilities.inner_logic();
            let counter = self.counter_owner.inner_logic();
            self.shared == shared &&
            self.raw.invariant() &&
            self.base.invariant() && caps.0.invariant() && caps.1.invariant() &&
            self.raw.raw_pointer() == self.base.raw_pointer() &&
            self.raw.capacity() == caps.0.capacity() &&
            self.raw.namespace() == caps.0.namespace() &&
            self.base@ == Some((caps.0.namespace(), caps.0.capacity(), 0int)) &&
            caps.1.capacity() == caps.0.capacity() &&
            caps.1.namespace() == caps.0.namespace() &&
            caps.1.resource_id() == caps.0.namespace() &&
            caps.1.lo() == 0 && caps.1.hi() == caps.0.capacity() &&
            0 < caps.0.capacity() &&
            self.shared_owner.inner_logic().invariant() &&
            *self.shared_owner.ward() == shared as *const Shared &&
            self.shared_owner.val().buf == self.base.raw_pointer() &&
            self.shared_owner.val().cap@ == caps.0.capacity() &&
            self.shared_owner.val().ref_cnt.invariant() &&
            !self.shared_owner.val().ref_cnt.has_event_invariant() &&
            counter.0.invariant() &&
            *counter.0.ward() == self.shared_owner.val().ref_cnt.counter_model() &&
            counter.0.val() == count && counter.1 == count
        }
    }

    /// Exact count-one handoff to the original Vec-backed mutable layout.
    #[logic(open(self), prophetic)]
    fn take_mut_valid(self, shared: *mut Shared, ptr: *const u8, len: usize) -> bool {
        pearlite! {
            self.release_valid(shared, 1usize) &&
            ptr == self.base.raw_pointer() as *const u8 &&
            len@ == (*self.contents).len() &&
            len@ <= self.raw.capacity() &&
            (forall<i:Int> 0 <= i && i < len@ ==>
                self.capabilities.inner_logic().1.slot(i) == Some(Some((*self.contents)[i])))
        }
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
struct BoxedReleaseProof {
    base: BoundPtr,
    view: BoundPtr,
    view_offset: usize,
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Bytes {
    /// The Bytes view produced by consuming a nonempty boxed slice is backed
    /// by the exact B1 allocation and full initialized region returned while
    /// consuming that Box.
    #[logic(open(self), prophetic)]
    pub(crate) fn original_boxed_valid(self) -> bool {
        pearlite! {
            match self.original_boxed {
                None => false,
                Some(proof) => {
                    let caps = proof.capabilities.inner_logic();
                    proof.base.invariant() && caps.0.invariant() && caps.1.invariant() &&
                    proof.base@ == Some((caps.0.namespace(), caps.0.capacity(), 0int)) &&
                    self.ptr == proof.base.raw_pointer() as *const u8 &&
                    self.len@ == caps.0.capacity() &&
                    caps.1.capacity() == caps.0.capacity() &&
                    caps.1.namespace() == caps.0.namespace() &&
                    caps.1.resource_id() == caps.0.namespace() &&
                    caps.1.lo() == 0 && caps.1.hi() == caps.0.capacity() &&
                    self.readonly_packet == None &&
                    self.readonly_cell_permission.inner_logic() == None &&
                    self.original_spare_shared == None &&
                    proof.even_pointer == (self.ptr.addr_logic()@ % 2 == 0) &&
                    self.vtable == proof.vtable &&
                    proof.data_cell_permission.inner_logic().ward().raw_cell() == &self.data &&
                    proof.data_cell_permission.inner_logic().val().addr_logic()@ == (
                        if proof.even_pointer {
                            self.ptr.addr_logic()@ + 1
                        } else {
                            self.ptr.addr_logic()@
                        }) &&
                    proof.data_cell_permission.inner_logic().val() as *mut u8 == (
                        if proof.even_pointer {
                            proof.base.raw_pointer().offset_logic(1)
                        } else {
                            proof.base.raw_pointer()
                        }) &&
                    (*proof.contents).len() == self.len@ &&
                    forall<i:Int> 0 <= i && i < self.len@ ==>
                        slot_known(caps.1.slot(i)) &&
                        caps.1.slot(i) == Some(Some((*proof.contents)[i]))
                }
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn original_boxed_bytes(self) -> Seq<u8> {
        pearlite! {
            *self.original_boxed.unwrap_logic().contents
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_boxed_vtable(self) -> &'static Vtable {
        self.original_boxed.unwrap_logic().vtable
    }

    /// The spare-capacity Vec constructor preserves the actual Vec allocation,
    /// Shared header Box, and pointer-cell and counter permissions.
    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_valid(self) -> bool {
        pearlite! {
            self.original_spare_shared_allocation_valid() &&
            self.original_spare_shared_header_valid() &&
            self.original_spare_shared_counter_valid() &&
            self.original_spare_shared_cell_valid()
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_allocation_valid(self) -> bool {
        pearlite! {
            match self.original_spare_shared {
                None => false,
                Some(proof) => {
                    let caps = proof.capabilities.inner_logic();
                    proof.raw.invariant() &&
                    proof.base.invariant() &&
                    caps.0.invariant() && caps.1.invariant() &&
                    proof.raw.raw_pointer() == proof.base.raw_pointer() &&
                    proof.raw.capacity() == caps.0.capacity() &&
                    proof.raw.namespace() == caps.0.namespace() &&
                    proof.base@ == Some((caps.0.namespace(), caps.0.capacity(), 0int)) &&
                    self.ptr == proof.base.raw_pointer() as *const u8 &&
                    self.len@ == (*proof.contents).len() &&
                    self.len@ < caps.0.capacity() &&
                    caps.1.capacity() == caps.0.capacity() &&
                    caps.1.namespace() == caps.0.namespace() &&
                    caps.1.resource_id() == caps.0.namespace() &&
                    caps.1.lo() == 0 && caps.1.hi() == caps.0.capacity() &&
                    (forall<i:Int> 0 <= i && i < self.len@ ==>
                        caps.1.slot(i) == Some(Some((*proof.contents)[i]))) &&
                    (forall<i:Int> self.len@ <= i && i < caps.0.capacity() ==>
                        caps.1.slot(i) == Some(None))
                }
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_header_valid(self) -> bool {
        pearlite! {
            self.original_spare_shared_tag_clear() &&
            self.original_spare_shared_header_state_valid()
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_tag_clear(self) -> bool {
        pearlite! {
            match self.original_spare_shared {
                None => false,
                Some(proof) => (proof.shared.addr_logic() & KIND_MASK) == 0usize
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_header_state_valid(self) -> bool {
        pearlite! {
            match self.original_spare_shared {
                None => false,
                Some(proof) => {
                    let caps = proof.capabilities.inner_logic();
                    proof.shared_owner.inner_logic().invariant() &&
                    *proof.shared_owner.ward() == proof.shared as *const Shared &&
                    proof.shared_owner.val().buf == proof.base.raw_pointer() &&
                    proof.shared_owner.val().cap@ == caps.0.capacity() &&
                    proof.shared_owner.val().ref_cnt.invariant() &&
                    !proof.shared_owner.val().ref_cnt.has_event_invariant()
                }
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_counter_valid(self) -> bool {
        pearlite! {
            match self.original_spare_shared {
                None => false,
                Some(proof) => {
                    let counter = proof.counter_owner.inner_logic();
                    counter.0.invariant() &&
                    *counter.0.ward() == proof.shared_owner.val().ref_cnt.counter_model() &&
                    counter.0.val() == 1usize && counter.1 == 1usize
                }
            }
        }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_cell_valid(self) -> bool {
        pearlite! {
            match self.original_spare_shared {
                None => false,
                Some(proof) => {
                    self.readonly_packet == None &&
                    self.readonly_cell_permission.inner_logic() == None &&
                    self.original_boxed == None &&
                    self.vtable == proof.vtable &&
                    proof.data_cell_permission.inner_logic().invariant() &&
                    proof.data_cell_permission.inner_logic().ward().raw_cell() == &self.data &&
                    proof.data_cell_permission.inner_logic().val() == proof.shared as *mut ()
                }
            }
        }
    }

    #[logic(open(self))]
    pub(crate) fn original_spare_shared_bytes(self) -> Seq<u8> {
        pearlite! { *self.original_spare_shared.unwrap_logic().contents }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_spare_shared_capacity(self) -> Int {
        pearlite! {
            self.original_spare_shared.unwrap_logic().capabilities.inner_logic().0.capacity()
        }
    }

    /// An original Bytes view backed by either the boxed-slice or the
    /// spare-capacity Vec carrier owns an actual B1 region for its visible
    /// bytes. The representation modes are mutually exclusive.
    #[logic(open(self), prophetic)]
    pub(crate) fn original_borrowable_valid(self) -> bool {
        self.original_boxed_valid() || self.original_spare_shared_valid()
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn original_borrowable_bytes(self) -> Seq<u8> {
        pearlite! {
            match self.original_boxed {
                Some(proof) => *proof.contents,
                None => *self.original_spare_shared.unwrap_logic().contents
            }
        }
    }
}

#[cfg(all(creusot, bytes_original_unique_gate))]
impl Bytes {
    /// The readonly packet carried by a BytesMut ARC freeze agrees with this
    /// handle's concrete view. This records the owner transition and pointer
    /// view; vtable-driven read and release behavior remain separate proof
    /// obligations.
    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_packet_valid(self) -> bool {
        pearlite! {
            match (self.readonly_packet, self.readonly_cell_permission.inner_logic()) {
                (Some(packet), Some(permission)) =>
                    packet.invariant() &&
                    packet.cursor.raw_pointer() as *const u8 == self.ptr &&
                    packet.len@ == self.len@ &&
                    packet.cursor.current_address() == self.ptr.addr_logic()@ &&
                    self.original_boxed == None &&
                    self.original_spare_shared == None &&
                    permission.ward().raw_cell() == &self.data &&
                    permission.val() == packet.capsule_public().0,
                _ => false,
            }
        }
    }

    /// Exact proof projections for the private readonly packet carried by a
    /// BytesMut ARC freeze. These expose metadata only; the affine ticket and
    /// region remain in the packet field.
    #[logic(open(self), prophetic)]
    pub(crate) fn proof_view_pointer(self) -> *const u8 {
        self.ptr
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn proof_view_len(self) -> Int {
        pearlite! { self.len@ }
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn proof_vtable(self) -> &'static Vtable {
        self.vtable
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn proof_data_cell(self) -> BytesDataCell {
        self.data
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_packet_state(self) -> Option<crate::bytes_mut::shared_capsule::SharedReadonlyPacket> {
        self.readonly_packet
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_cell_permission_state(
        self,
    ) -> Option<creusot_std::ghost::perm::Perm<crate::loom::sync::atomic::NativePtr<()>>> {
        self.readonly_cell_permission.inner_logic()
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_packet_public(self) -> Option<(*mut (), crate::ref_count_ops::NativeCounter, Id, Id)> {
        pearlite! { match self.readonly_packet {
            Some(packet) => Some(packet.capsule_public()),
            None => None,
        }}
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_packet_key(self) -> Option<crate::ownership_proof::ready_registry::ReadyKey> {
        pearlite! { match self.readonly_packet {
            Some(packet) => Some(packet.lease_key()),
            None => None,
        }}
    }

    #[logic(open(self), prophetic)]
    pub(crate) fn readonly_packet_ticket_frame(self) -> Option<(
        Id,
        creusot_std::logic::Int,
        Id,
        crate::ownership_proof::ref_count_ledger::RefKind,
        crate::ownership_proof::ref_count_ledger::AllocationBinding,
        creusot_std::logic::Int,
        creusot_std::logic::Int,
    )> {
        pearlite! { match self.readonly_packet {
            Some(packet) => {
                let ticket = packet.ticket.inner_logic();
                Some((
                    ticket.ledger_id(),
                    ticket.owner_id(),
                    ticket.fragment_resource_id(),
                    ticket.class(),
                    ticket.allocation_binding(),
                    ticket.owned_lo(),
                    ticket.owned_hi(),
                ))
            },
            None => None,
        }}
    }
}

// Temporary ordinary translation leaves. No atomic value/state, refcount,
// vtable identity or callback law is assumed. Native branches below still use
// the exact original core/loom constructors and SHARED_VTABLE reference.
#[cfg(all(creusot, bytes_original_freeze_gate))]
#[trusted]
#[ensures(true)]
fn original_atomic_ptr_new<T>(_value: *mut T) -> AtomicPtr<T> { unimplemented!() }
#[cfg(all(creusot, bytes_original_freeze_gate))]
#[trusted]
#[ensures(true)]
fn original_shared_vtable() -> &'static Vtable { unimplemented!() }

impl Bytes {
    /// Releases this handle explicitly. The backing storage is reclaimed when
    /// its last owner releases it.
    ///
    /// This calls the same destructor as `Drop`, exactly once.
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn cleanup(self) {
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { original_shared_cleanup(self); }
        #[cfg(not(all(creusot, bytes_original_shared_gate)))]
        {
            let mut this = ManuallyDrop::new(self);
            let ptr = this.ptr;
            let len = this.len;
            let callback = this.vtable.drop;
            unsafe { callback(&mut this.data, ptr, len) }
        }
    }

    /// Explicitly release a handle produced by the BytesMut ARC freeze path.
    /// This consumes the readonly owner and delegates to the same shared
    /// native release body as the BytesMut owner path. Ordinary `Drop` and
    /// vtable callbacks are unchanged and remain outside this proof entry.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.readonly_packet_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(context.inner_logic().invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        context.inner_logic().token_namespaces().contains(crate::ref_count_ops::event_namespace())
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
        context.inner_logic().ready_entry_state(self.readonly_packet_key().unwrap_logic().slot()) == Some((
            self.readonly_packet_key().unwrap_logic().resource_id(),
            self.readonly_packet_key().unwrap_logic().slot(),
            self.readonly_packet_key().unwrap_logic().resource_id(),
            Some(Excl((self.readonly_packet_key().unwrap_logic().resource_id(), None)))
        )) && self.readonly_packet_key().unwrap_logic().slot() < context.inner_logic().ready_len()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures((^context.inner_logic()).invariant()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        (^context.inner_logic()).token_namespaces() == context.inner_logic().token_namespaces() &&
        (^context.inner_logic()).ready_len() == context.inner_logic().ready_len()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ > 0))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ > 1 ==>
        (^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(self.readonly_packet_key().unwrap_logic().slot()) == Some((
            self.readonly_packet_key().unwrap_logic().resource_id(),
            self.readonly_packet_key().unwrap_logic().slot(),
            self.readonly_packet_key().unwrap_logic().resource_id(),
            Some(Excl((self.readonly_packet_key().unwrap_logic().resource_id(), None)))
        ))
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == 1 ==>
        !(^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(self.readonly_packet_key().unwrap_logic().slot()) == None
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        forall<i: Int> 0 <= i && i < context.inner_logic().ready_len() &&
            i != self.readonly_packet_key().unwrap_logic().slot() ==>
            (^context.inner_logic()).ready_entry_state(i) == context.inner_logic().ready_entry_state(i)
    ))]
    ///
    /// # Safety
    /// The handle must be an ARC-backed readonly handle produced by the
    /// BytesMut freeze path. In the direct proof build, `context` must be its
    /// matching live operation context.
    pub(crate) unsafe fn dispose_shared(
        self,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        context: Ghost<&mut crate::ownership_proof::shared_context::BorrowedSharedContext<
            '_, crate::bytes_mut::shared_capsule::HeaderLeaseState,
        >>,
    ) -> usize {
        let mut this = self;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            let packet = this.readonly_packet.take().unwrap();
            let permission = mem::replace(&mut this.readonly_cell_permission, ghost!(None));
            let mut permission = ghost! { permission.into_inner().unwrap() };
            let ptr = crate::loom::sync::atomic::NativePtr::exclusive_read(
                &mut this.data,
                ghost!(&mut *permission),
            );
            core::mem::forget(this);
            crate::bytes_mut::release_readonly_shared(
                ptr, packet, context,
            )
        }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        {
            let ptr = this.data.with_mut(|ptr| *ptr);
            core::mem::forget(this);
            crate::bytes_mut::release_readonly_shared(ptr)
        }
    }

    /// Proof entry for the Readonly shared copy path. It transfers the actual
    /// packet and operation context to the same all-build nonunique source
    /// helper used by the native vtable callback. That helper lends the
    /// packet's physical interval for the B4-backed copy, restores it, and
    /// releases this owner through the common source release body. The unique
    /// Vec reconstruction branch remains a separate path.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[requires(self.readonly_packet_valid())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(
        crate::ref_count_ops::event_namespace()
    ))]
    #[requires(self.readonly_packet_key().unwrap_logic().slot() <
        context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(
        self.readonly_packet_key().unwrap_logic().slot()) == Some((
            self.readonly_packet_key().unwrap_logic().resource_id(),
            self.readonly_packet_key().unwrap_logic().slot(),
            self.readonly_packet_key().unwrap_logic().resource_id(),
            Some(Excl((
                self.readonly_packet_key().unwrap_logic().resource_id(), None
            )))
        )))]
    #[ensures(result.bytes@ == *result.witness.inner_logic().contents)]
    #[ensures(*result.witness.inner_logic().release_old_count > 0)]
    #[ensures(*result.witness.inner_logic().release_old_count > 1 ==>
        (^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(
            self.readonly_packet_key().unwrap_logic().slot()) == Some((
                self.readonly_packet_key().unwrap_logic().resource_id(),
                self.readonly_packet_key().unwrap_logic().slot(),
                self.readonly_packet_key().unwrap_logic().resource_id(),
                Some(Excl((self.readonly_packet_key().unwrap_logic().resource_id(), None)))
            )))]
    #[ensures(*result.witness.inner_logic().release_old_count == 1 ==>
        !(^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(
            self.readonly_packet_key().unwrap_logic().slot()) == None)]
    #[ensures((*result.witness.inner_logic().region).invariant())]
    #[ensures((*result.witness.inner_logic().region).resource_id() ==
        (*result.witness.inner_logic().region).namespace())]
    #[ensures(*result.witness.inner_logic().pointer == self.ptr)]
    #[ensures((*result.witness.inner_logic().contents).len() == self.len@)]
    #[ensures((*result.witness.inner_logic().region).namespace() ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().0)]
    #[ensures((*result.witness.inner_logic().region).capacity() ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().1)]
    #[ensures(*result.witness.inner_logic().start ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().2)]
    #[ensures((*result.witness.inner_logic().region).lo() <=
        *result.witness.inner_logic().start)]
    #[ensures(*result.witness.inner_logic().start +
        (*result.witness.inner_logic().contents).len() <=
            (*result.witness.inner_logic().region).hi())]
    #[ensures(forall<index: Int>
        0 <= index && index < (*result.witness.inner_logic().contents).len() ==>
            (*result.witness.inner_logic().region).slot(
                *result.witness.inner_logic().start + index) ==
                Some(Some((*result.witness.inner_logic().contents)[index])))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.readonly_packet_key().unwrap_logic().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    pub(crate) unsafe fn copy_shared_readonly_to_vec_with_context(
        self,
        context: Ghost<&mut crate::ownership_proof::shared_context::BorrowedSharedContext<
            '_, crate::bytes_mut::shared_capsule::HeaderLeaseState,
        >>,
    ) -> crate::bytes_mut::SharedReadonlyCopyResult {
        let mut this = self;
        let packet = this.readonly_packet.take().unwrap();
        let permission = mem::replace(&mut this.readonly_cell_permission, ghost!(None));
        let mut permission = ghost! { permission.into_inner().unwrap() };
        let shared = crate::loom::sync::atomic::NativePtr::exclusive_read(
            &mut this.data,
            ghost!(&mut *permission),
        );
        let ptr = this.ptr;
        let len = this.len;
        core::mem::forget(this);
        let copied = crate::bytes_mut::shared_v_to_vec_nonunique(
            shared, ptr, len,
            Some(crate::bytes_mut::SharedReadonlyCopyProof {
                packet,
                context,
            }),
        );
        copied
    }

    /// Convert a packet-backed Readonly view into a fresh mutable Vec owner
    /// through the same nonunique copy/release source operation used by the
    /// original `shared_v_to_mut` callback. The returned witness records the
    /// copied values and the exact conditional Context result of releasing
    /// this Readonly owner.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    #[requires(self.readonly_packet_valid())]
    #[requires(context.inner_logic().invariant())]
    #[requires(context.inner_logic().token_namespaces().contains(
        crate::ref_count_ops::event_namespace()
    ))]
    #[requires(self.readonly_packet_key().unwrap_logic().slot() <
        context.inner_logic().ready_len())]
    #[requires(context.inner_logic().ready_entry_state(
        self.readonly_packet_key().unwrap_logic().slot()) == Some((
            self.readonly_packet_key().unwrap_logic().resource_id(),
            self.readonly_packet_key().unwrap_logic().slot(),
            self.readonly_packet_key().unwrap_logic().resource_id(),
            Some(Excl((
                self.readonly_packet_key().unwrap_logic().resource_id(), None
            )))
        )))]
    #[ensures(result.buffer.unique_valid())]
    #[ensures(result.buffer.unique_vec_rebuildable())]
    #[ensures(result.buffer.unique_length() == self.len@)]
    #[ensures(result.buffer.unique_bytes() == *result.witness.inner_logic().contents)]
    #[ensures(result.buffer.unique_offset() == 0)]
    #[ensures(result.buffer.unique_pointer() ==
        creusot_std::std::vec::pointer_model(*result.output_vector))]
    #[ensures(result.buffer.unique_capacity() ==
        creusot_std::std::vec::capacity_model(*result.output_vector))]
    #[ensures(result.buffer.unique_allocation_capacity() ==
        creusot_std::std::vec::capacity_model(*result.output_vector))]
    #[ensures((*result.output_vector)@ == *result.witness.inner_logic().contents)]
    #[ensures(forall<index: Int>
        0 <= index && index < self.len@ ==>
            result.buffer.unique_slot(index) ==
                Some(Some((*result.witness.inner_logic().contents)[index])))]
    #[ensures(forall<index: Int>
        self.len@ <= index && index <
            creusot_std::std::vec::capacity_model(*result.output_vector) ==>
            result.buffer.unique_slot(index) == Some(None))]
    #[ensures(*result.witness.inner_logic().release_old_count > 0)]
    #[ensures(*result.witness.inner_logic().release_old_count > 1 ==>
        (^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(
            self.readonly_packet_key().unwrap_logic().slot()) == Some((
                self.readonly_packet_key().unwrap_logic().resource_id(),
                self.readonly_packet_key().unwrap_logic().slot(),
                self.readonly_packet_key().unwrap_logic().resource_id(),
                Some(Excl((self.readonly_packet_key().unwrap_logic().resource_id(), None)))
            )))]
    #[ensures(*result.witness.inner_logic().release_old_count == 1 ==>
        !(^context.inner_logic()).ready_has_key(self.readonly_packet_key().unwrap_logic()) &&
        (^context.inner_logic()).ready_entry_state(
            self.readonly_packet_key().unwrap_logic().slot()) == None)]
    #[ensures((*result.witness.inner_logic().region).invariant())]
    #[ensures((*result.witness.inner_logic().region).resource_id() ==
        (*result.witness.inner_logic().region).namespace())]
    #[ensures(*result.witness.inner_logic().pointer == self.ptr)]
    #[ensures((*result.witness.inner_logic().contents).len() == self.len@)]
    #[ensures((*result.witness.inner_logic().region).namespace() ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().0)]
    #[ensures((*result.witness.inner_logic().region).capacity() ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().1)]
    #[ensures(*result.witness.inner_logic().start ==
        self.readonly_packet_state().unwrap_logic().cursor@.unwrap_logic().2)]
    #[ensures((*result.witness.inner_logic().region).lo() <=
        *result.witness.inner_logic().start)]
    #[ensures(*result.witness.inner_logic().start +
        (*result.witness.inner_logic().contents).len() <=
            (*result.witness.inner_logic().region).hi())]
    #[ensures(forall<index: Int>
        0 <= index && index < (*result.witness.inner_logic().contents).len() ==>
            (*result.witness.inner_logic().region).slot(
                *result.witness.inner_logic().start + index) ==
                Some(Some((*result.witness.inner_logic().contents)[index])))]
    #[ensures((^context.inner_logic()).invariant())]
    #[ensures((^context.inner_logic()).token_namespaces() ==
        context.inner_logic().token_namespaces())]
    #[ensures((^context.inner_logic()).ready_len() == context.inner_logic().ready_len())]
    #[ensures(forall<index: Int>
        0 <= index && index < context.inner_logic().ready_len() &&
        index != self.readonly_packet_key().unwrap_logic().slot() ==>
            (^context.inner_logic()).ready_entry_state(index) ==
                context.inner_logic().ready_entry_state(index))]
    pub(crate) unsafe fn copy_shared_readonly_to_mut_with_context(
        self,
        context: Ghost<&mut crate::ownership_proof::shared_context::BorrowedSharedContext<
            '_, crate::bytes_mut::shared_capsule::HeaderLeaseState,
        >>,
    ) -> crate::bytes_mut::SharedReadonlyMutableResult {
        let mut this = self;
        let packet = this.readonly_packet.take().unwrap();
        let permission = mem::replace(&mut this.readonly_cell_permission, ghost!(None));
        let mut permission = ghost! { permission.into_inner().unwrap() };
        let shared = crate::loom::sync::atomic::NativePtr::exclusive_read(
            &mut this.data,
            ghost!(&mut *permission),
        );
        let ptr = this.ptr;
        let len = this.len;
        core::mem::forget(this);
        crate::bytes_mut::shared_v_to_mut_nonunique(
            shared, ptr, len,
            Some(crate::bytes_mut::SharedReadonlyCopyProof {
                packet,
                context,
            }),
        )
    }

    /// Explicitly release a nonempty, full-view boxed-slice handle through the
    /// same source cleanup function used by the vtable path.
    ///
    /// # Safety
    /// The handle must be the unpromoted full view produced directly from a
    /// nonempty boxed slice. It must not have been cloned, sliced, converted,
    /// or otherwise transferred before this call.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.original_boxed_valid()))]
    #[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
    pub(crate) unsafe fn dispose_boxed(self) {
        let mut this = self;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let boxed = this.original_boxed.take().unwrap();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let mut permission = ghost! { boxed.data_cell_permission.into_inner() };
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let tagged = crate::loom::sync::atomic::NativePtr::exclusive_read(
            &mut this.data,
            ghost!(&mut *permission),
        );
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let tagged = this.data.with_mut(|ptr| *ptr);
        let even_pointer = crate::provenance_specs::pointer_addr(this.ptr) % 2 == 0;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        proof_assert!(even_pointer == boxed.even_pointer);
        let native_buf = if even_pointer {
            untagged_vec_data_pointer(tagged.cast())
        } else {
            tagged.cast()
        };
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let base = boxed.base;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let capabilities = boxed.capabilities;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        proof_assert!(native_buf == base.raw_pointer());
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let view = base;
        let view_ptr = this.ptr;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        proof_assert!(view_ptr == view.raw_pointer() as *const u8);
        let len = this.len;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let release = BoxedReleaseProof {
            base,
            view,
            view_offset: 0,
            capabilities,
        };
        core::mem::forget(this);
        unsafe {
            free_boxed_slice(
                native_buf,
                view_ptr,
                len,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                Some(release),
            )
        }
    }

    /// Explicitly release the unpromoted count-one Shared handle created by
    /// the spare-capacity `From<Vec<u8>>` path.
    ///
    /// # Safety
    /// The handle must be the sole, full-view handle returned by that
    /// constructor and must not have been cloned, sliced, converted, or
    /// otherwise transferred before this call.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.original_spare_shared_valid()))]
    #[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
    pub(crate) unsafe fn dispose_spare_shared(self) {
        let mut this = self;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let mut proof = this.original_spare_shared.take().unwrap();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let mut permission = ghost! { proof.data_cell_permission.into_inner() };
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let shared = crate::loom::sync::atomic::NativePtr::exclusive_read(
            &mut this.data,
            ghost!(&mut *permission),
        ).cast::<Shared>();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        { proof.data_cell_permission = permission; }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let shared = this.data.with_mut(|pointer| *pointer).cast::<Shared>();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            proof_assert!(shared == proof.shared);
            proof_assert!(proof.release_valid(shared, 1usize));
        }
        core::mem::forget(this);
        #[cfg(all(creusot, bytes_original_unique_gate))]
        unsafe { release_shared(shared, Some(proof)) };
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        unsafe { release_shared(shared) };
    }

    /// Convert the unpromoted spare-capacity Shared representation back to
    /// the original unique Vec-backed BytesMut path.
    ///
    /// # Safety
    /// The handle must be the unchanged, sole handle returned by the spare-
    /// capacity `From<Vec<u8>>` constructor.
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.original_spare_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == self.len@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_offset() == 0))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.unique_capacity() == self.original_spare_shared_capacity()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.unique_allocation_capacity() == self.original_spare_shared_capacity()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.unique_pointer() == self.ptr as *mut u8
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.unique_bytes() == self.original_spare_shared_bytes()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
        result.unique_length() <= i && i < result.unique_capacity() ==>
            result.unique_slot(i) == Some(None)
    ))]
    #[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
    pub(crate) unsafe fn into_mut_spare_shared(self) -> BytesMut {
        let mut this = self;
        let ptr = this.ptr;
        let len = this.len;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let mut proof = this.original_spare_shared.take().unwrap();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let mut permission = ghost! { proof.data_cell_permission.into_inner() };
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let shared = crate::loom::sync::atomic::NativePtr::exclusive_read(
            &mut this.data,
            ghost!(&mut *permission),
        ).cast::<Shared>();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        { proof.data_cell_permission = permission; }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let shared = this.data.with_mut(|pointer| *pointer).cast::<Shared>();
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            proof_assert!(shared == proof.shared);
            proof_assert!(proof.take_mut_valid(shared, ptr, len));
        }
        mem::forget(this);
        #[cfg(all(creusot, bytes_original_unique_gate))]
        unsafe { shared_to_mut_impl(shared, ptr, len, Some(proof)) }
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        unsafe { shared_to_mut_impl(shared, ptr, len) }
    }

    /// Creates a new empty `Bytes`.
    ///
    /// This will not allocate and the returned `Bytes` handle will be empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let b = Bytes::new();
    /// assert_eq!(&b[..], b"");
    /// ```
    #[inline]
    #[cfg(not(any(
        all(loom, test),
        all(creusot, bytes_original_constructor_gate),
        all(creusot, bytes_original_unique_gate),
    )))]
    pub const fn new() -> Self {
        // Make it a named const to work around
        // "unsizing casts are not allowed in const fn"
        const EMPTY: &[u8] = &[];
        Bytes::from_static(EMPTY)
    }

    #[cfg(all(creusot, bytes_original_unique_gate, not(bytes_original_constructor_gate)))]
    pub fn new() -> Self {
        const EMPTY: &[u8] = &[];
        Bytes::from_static(EMPTY)
    }

    /// Creates a new empty `Bytes`.
    #[cfg(all(loom, test, not(all(creusot, bytes_original_constructor_gate))))]
    pub fn new() -> Self {
        const EMPTY: &[u8] = &[];
        Bytes::from_static(EMPTY)
    }

    #[cfg(all(creusot, bytes_original_constructor_gate))]
    #[ensures(result.original_bytes_valid())]
    #[ensures(result.original_bytes_content().len() == 0)]
    pub fn new() -> Self {
        const EMPTY: &[u8] = &[];
        Bytes::from_static(EMPTY)
    }

    /// Creates a new `Bytes` from a static slice.
    ///
    /// The returned `Bytes` will point directly to the static slice. There is
    /// no allocating or copying.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let b = Bytes::from_static(b"hello");
    /// assert_eq!(&b[..], b"hello");
    /// ```
    #[inline]
    #[cfg(not(any(
        all(loom, test),
        all(creusot, bytes_original_constructor_gate),
        all(creusot, bytes_original_unique_gate),
    )))]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: BytesDataCell::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    #[cfg(all(creusot, bytes_original_unique_gate, not(bytes_original_constructor_gate)))]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: BytesDataCell::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    /// Creates a new `Bytes` from a static slice.
    #[cfg(all(loom, test, not(all(creusot, bytes_original_constructor_gate))))]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: BytesDataCell::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    #[cfg(all(creusot, bytes_original_constructor_gate))]
    #[ensures(result.original_bytes_valid())]
    #[ensures(result.original_bytes_content() == bytes@)]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        original_bytes_from_static(bytes)
    }

    /// Creates a new `Bytes` with length zero and the given pointer as the address.
    #[cfg_attr(creusot, creusot::no_translate)]
    fn new_empty_with_ptr(ptr: *const u8) -> Self {
        debug_assert!(!ptr.is_null());

        // Detach this pointer's provenance from whichever allocation it came from, and reattach it
        // to the provenance of the fake ZST [u8;0] at the same address.
        let ptr = without_provenance(ptr as usize);

        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,

            #[cfg(all(creusot, bytes_original_freeze_gate))]

            original_frozen: None,
            ptr,
            len: 0,
            data: BytesDataCell::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    /// Create [Bytes] with a buffer whose lifetime is controlled
    /// via an explicit owner.
    ///
    /// A common use case is to zero-copy construct from mapped memory.
    ///
    /// ```
    /// # struct File;
    /// #
    /// # impl File {
    /// #     pub fn open(_: &str) -> Result<Self, ()> {
    /// #         Ok(Self)
    /// #     }
    /// # }
    /// #
    /// # mod memmap2 {
    /// #     pub struct Mmap;
    /// #
    /// #     impl Mmap {
    /// #         pub unsafe fn map(_file: &super::File) -> Result<Self, ()> {
    /// #             Ok(Self)
    /// #         }
    /// #     }
    /// #
    /// #     impl AsRef<[u8]> for Mmap {
    /// #         fn as_ref(&self) -> &[u8] {
    /// #             b"buf"
    /// #         }
    /// #     }
    /// # }
    /// use bytes::Bytes;
    /// use memmap2::Mmap;
    ///
    /// # fn main() -> Result<(), ()> {
    /// let file = File::open("upload_bundle.tar.gz")?;
    /// let mmap = unsafe { Mmap::map(&file) }?;
    /// let b = Bytes::from_owner(mmap);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The `owner` will be transferred to the constructed [Bytes] object, which
    /// will ensure it is dropped once all remaining clones of the constructed
    /// object are dropped. The owner will then be responsible for dropping the
    /// specified region of memory as part of its [Drop] implementation.
    ///
    /// Note that converting [Bytes] constructed from an owner into a [BytesMut]
    /// will always create a deep copy of the buffer into newly allocated memory.
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn from_owner<T>(owner: T) -> Self
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        // Safety & Miri:
        // The ownership of `owner` is first transferred to the `Owned` wrapper and `Bytes` object.
        // This ensures that the owner is pinned in memory, allowing us to call `.as_ref()` safely
        // since the lifetime of the owner is controlled by the lifetime of the new `Bytes` object,
        // and the lifetime of the resulting borrowed `&[u8]` matches that of the owner.
        // Note that this remains safe so long as we only call `.as_ref()` once.
        //
        // There are some additional special considerations here:
        //   * We rely on Bytes's Drop impl to clean up memory should `.as_ref()` panic.
        //   * Setting the `ptr` and `len` on the bytes object last (after moving the owner to
        //     Bytes) allows Miri checks to pass since it avoids obtaining the `&[u8]` slice
        //     from a stack-owned Box.
        // More details on this: https://github.com/tokio-rs/bytes/pull/742/#discussion_r1813375863
        //                  and: https://github.com/tokio-rs/bytes/pull/742/#discussion_r1813316032

        let owned = Box::into_raw(Box::new(Owned {
            ref_cnt: RefCountCell::new(1),
            owner,
        }));

        let mut ret = Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: NonNull::dangling().as_ptr(),
            len: 0,
            data: BytesDataCell::new(owned.cast()),
            vtable: &Owned::<T>::VTABLE,
        };

        let buf = unsafe { &*owned }.owner.as_ref();
        ret.ptr = buf.as_ptr();
        ret.len = buf.len();

        ret
    }

    /// Returns the number of bytes contained in this `Bytes`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let b = Bytes::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result == self.len))]
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the `Bytes` has a length of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let b = Bytes::new();
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if this is the only reference to the data and
    /// `Into<BytesMut>` would avoid cloning the underlying buffer.
    ///
    /// Always returns false if the data is backed by a [static slice](Bytes::from_static),
    /// or an [owner](Bytes::from_owner).
    ///
    /// The result of this method may be invalidated immediately if another
    /// thread clones this value while this is being called. Ensure you have
    /// unique access to this value (`&mut Bytes`) first if you need to be
    /// certain the result is valid (i.e. for safety reasons).
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let a = Bytes::from(vec![1, 2, 3]);
    /// assert!(a.is_unique());
    /// let b = a.clone();
    /// assert!(!a.is_unique());
    /// ```
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn is_unique(&self) -> bool {
        unsafe { (self.vtable.is_unique)(&self.data) }
    }

    /// Creates `Bytes` instance from slice, by copying it.
    pub fn copy_from_slice(data: &[u8]) -> Self {
        data.to_vec().into()
    }

    /// Returns a slice of self for the provided range.
    ///
    /// This will increment the reference count for the underlying memory and
    /// return a new `Bytes` handle set to the slice.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let a = Bytes::from(&b"hello world"[..]);
    /// let b = a.slice(2..5);
    ///
    /// assert_eq!(&b[..], b"llo");
    /// ```
    ///
    /// # Panics
    ///
    /// Requires that `begin <= end` and `end <= self.len()`, otherwise slicing
    /// will panic.
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        use core::ops::Bound;

        let len = self.len();

        let begin = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n.checked_add(1).expect("out of range"),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        assert!(
            begin <= end,
            "range start must not be greater than end: {:?} <= {:?}",
            begin,
            end,
        );
        assert!(
            end <= len,
            "range end out of bounds: {:?} <= {:?}",
            end,
            len,
        );

        if end == begin {
            return Bytes::new_empty_with_ptr(self.ptr.wrapping_add(begin));
        }

        let mut ret = self.clone();

        ret.len = end - begin;
        ret.ptr = unsafe { ret.ptr.add(begin) };

        ret
    }

    /// Returns a slice of self that is equivalent to the given `subset`.
    ///
    /// When processing a `Bytes` buffer with other tools, one often gets a
    /// `&[u8]` which is in fact a slice of the `Bytes`, i.e. a subset of it.
    /// This function turns that `&[u8]` into another `Bytes`, as if one had
    /// called `self.slice()` with the offsets that correspond to `subset`.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let bytes = Bytes::from(&b"012345678"[..]);
    /// let as_slice = bytes.as_ref();
    /// let subset = &as_slice[2..6];
    /// let subslice = bytes.slice_ref(&subset);
    /// assert_eq!(&subslice[..], b"2345");
    /// ```
    ///
    /// # Panics
    ///
    /// Requires that the given `sub` slice is in fact contained within the
    /// `Bytes` buffer; otherwise this function will panic.
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn slice_ref(&self, subset: &[u8]) -> Self {
        // Empty slice and empty Bytes may have their pointers reset
        // so explicitly allow empty slice to be a subslice of any slice.
        if subset.is_empty() {
            return Bytes::new();
        }

        let bytes_p = self.as_ptr() as usize;
        let bytes_len = self.len();

        let sub_p = subset.as_ptr() as usize;
        let sub_len = subset.len();

        assert!(
            sub_p >= bytes_p,
            "subset pointer ({:p}) is smaller than self pointer ({:p})",
            subset.as_ptr(),
            self.as_ptr(),
        );
        assert!(
            sub_p + sub_len <= bytes_p + bytes_len,
            "subset is out of bounds: self = ({:p}, {}), subset = ({:p}, {})",
            self.as_ptr(),
            bytes_len,
            subset.as_ptr(),
            sub_len,
        );

        let sub_offset = sub_p - bytes_p;

        self.slice(sub_offset..(sub_offset + sub_len))
    }

    /// Splits the bytes into two at the given index.
    ///
    /// Afterwards `self` contains elements `[0, at)`, and the returned `Bytes`
    /// contains elements `[at, len)`. It's guaranteed that the memory does not
    /// move, that is, the address of `self` does not change, and the address of
    /// the returned slice is `at` bytes after that.
    ///
    /// This is an `O(1)` operation that just increases the reference count and
    /// sets a few indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let mut a = Bytes::from(&b"hello world"[..]);
    /// let b = a.split_off(5);
    ///
    /// assert_eq!(&a[..], b"hello");
    /// assert_eq!(&b[..], b" world");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[must_use = "consider Bytes::truncate if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> Self {
        if at == self.len() {
            return Bytes::new_empty_with_ptr(self.ptr.wrapping_add(at));
        }

        if at == 0 {
            return mem::replace(self, Bytes::new_empty_with_ptr(self.ptr));
        }

        assert!(
            at <= self.len(),
            "split_off out of bounds: {:?} <= {:?}",
            at,
            self.len(),
        );

        let mut ret = self.clone();

        self.len = at;

        unsafe { ret.inc_start(at) };

        ret
    }

    /// Splits the bytes into two at the given index.
    ///
    /// Afterwards `self` contains elements `[at, len)`, and the returned
    /// `Bytes` contains elements `[0, at)`.
    ///
    /// This is an `O(1)` operation that just increases the reference count and
    /// sets a few indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let mut a = Bytes::from(&b"hello world"[..]);
    /// let b = a.split_to(5);
    ///
    /// assert_eq!(&a[..], b" world");
    /// assert_eq!(&b[..], b"hello");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    #[must_use = "consider Bytes::advance if you don't need the other half"]
    pub fn split_to(&mut self, at: usize) -> Self {
        if at == self.len() {
            let end_ptr = self.ptr.wrapping_add(at);
            return mem::replace(self, Bytes::new_empty_with_ptr(end_ptr));
        }

        if at == 0 {
            return Bytes::new_empty_with_ptr(self.ptr);
        }

        assert!(
            at <= self.len(),
            "split_to out of bounds: {:?} <= {:?}",
            at,
            self.len(),
        );

        let mut ret = self.clone();

        unsafe { self.inc_start(at) };

        ret.len = at;
        ret
    }

    /// Shortens the buffer, keeping the first `len` bytes and dropping the
    /// rest.
    ///
    /// If `len` is greater than the buffer's current length, this has no
    /// effect.
    ///
    /// The [split_off](`Self::split_off()`) method can emulate `truncate`, but this causes the
    /// excess bytes to be returned instead of dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let mut buf = Bytes::from(&b"hello world"[..]);
    /// buf.truncate(5);
    /// assert_eq!(buf, b"hello"[..]);
    /// ```
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            // The Vec "promotable" vtables do not store the capacity,
            // so we cannot truncate while using this repr. We *have* to
            // promote using `split_off` so the capacity can be stored.
            if self.vtable as *const Vtable == &PROMOTABLE_EVEN_VTABLE
                || self.vtable as *const Vtable == &PROMOTABLE_ODD_VTABLE
            {
                drop(self.split_off(len));
            } else {
                self.len = len;
            }
        }
    }

    /// Clears the buffer, removing all data.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::Bytes;
    ///
    /// let mut buf = Bytes::from(&b"hello world"[..]);
    /// buf.clear();
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Try to convert self into `BytesMut`.
    ///
    /// If `self` is unique for the entire original buffer, this will succeed
    /// and return a `BytesMut` with the contents of `self` without copying.
    /// If `self` is not unique for the entire original buffer, this will fail
    /// and return self.
    ///
    /// This will also always fail if the buffer was constructed via either
    /// [from_owner](Bytes::from_owner) or [from_static](Bytes::from_static).
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::{Bytes, BytesMut};
    ///
    /// let bytes = Bytes::from(b"hello".to_vec());
    /// assert_eq!(bytes.try_into_mut(), Ok(BytesMut::from(&b"hello"[..])));
    /// ```
    pub fn try_into_mut(self) -> Result<BytesMut, Bytes> {
        if self.is_unique() {
            Ok(self.into())
        } else {
            Err(self)
        }
    }

    #[inline]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.proof_view_pointer() == ptr))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.proof_view_len() == len@))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.proof_data_cell() == data))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.proof_vtable() == vtable))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.readonly_packet_state() == readonly_packet))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.readonly_packet_public() == match readonly_packet {
            Some(packet) => Some(packet.capsule_public()),
            None => None,
        }
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.readonly_packet_key() == match readonly_packet {
            Some(packet) => Some(packet.lease_key()),
            None => None,
        }
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.readonly_packet_ticket_frame() == match readonly_packet {
            Some(packet) => {
                let ticket = packet.ticket.inner_logic();
                Some((
                    ticket.ledger_id(),
                    ticket.owner_id(),
                    ticket.fragment_resource_id(),
                    ticket.class(),
                    ticket.allocation_binding(),
                    ticket.owned_lo(),
                    ticket.owned_hi(),
                ))
            },
            None => None,
        }
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
        result.readonly_cell_permission_state() == readonly_cell_permission.inner_logic()
    ))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match (readonly_packet, readonly_cell_permission.inner_logic()) {
        (Some(packet), Some(permission)) => packet.invariant() &&
            packet.cursor.raw_pointer() as *const u8 == ptr &&
            packet.len@ == len@ &&
            permission.ward().raw_cell() == &data &&
            permission.val() == packet.capsule_public().0,
        (None, None) => true,
        _ => false,
    }))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(match (readonly_packet, readonly_cell_permission.inner_logic()) {
        (Some(_), Some(_)) => result.readonly_packet_valid(),
        (None, None) => true,
        _ => false,
    }))]
    pub(crate) unsafe fn with_vtable(
        ptr: *const u8,
        len: usize,
        data: BytesDataCell,
        vtable: &'static Vtable,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_packet: Option<crate::bytes_mut::shared_capsule::SharedReadonlyPacket>,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_cell_permission: creusot_std::ghost::Ghost<Option<
            creusot_std::ghost::perm::Perm<crate::loom::sync::atomic::NativePtr<()>>
        >>,
    ) -> Bytes {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            proof_assert!(match readonly_packet {
                Some(packet) => packet.invariant() &&
                    packet.cursor.raw_pointer() as *const u8 == ptr &&
                    packet.len@ == len@,
                None => true,
            });
            proof_assert!(match (readonly_packet, readonly_cell_permission.inner_logic()) {
                (Some(packet), Some(permission)) =>
                    permission.ward().raw_cell() == &data &&
                    permission.val() == packet.capsule_public().0,
                (None, None) => true,
                _ => false,
            });
        }
        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr,
            len,
            data,
            vtable,
        }
    }

    // private

    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result@ == self.original_frozen_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), requires(self.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), requires(self.original_borrowable_valid()))]
    #[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result@ == self.original_borrowable_bytes()))]
    #[inline]
    fn as_slice(&self) -> &[u8] {
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { original_bytes_as_slice(self) }
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { original_shared_as_slice(self) }
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        {
            let proof = self.original_frozen.as_ref().unwrap();
            unsafe { raw_vec::borrow_bound(&proof.base, self.len, ghost! { &proof.capabilities.1 }) }
        }
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            match self.original_boxed.as_ref() {
                Some(proof) => unsafe {
                    raw_vec::borrow_bound(&proof.base, self.len, ghost! { &proof.capabilities.1 })
                },
                None => {
                    let proof = self.original_spare_shared.as_ref().unwrap();
                    unsafe {
                        raw_vec::borrow_bound(&proof.base, self.len, ghost! { &proof.capabilities.1 })
                    }
                }
            }
        }
        #[cfg(not(any(all(creusot, bytes_original_freeze_gate), all(creusot, bytes_original_shared_gate), all(creusot, bytes_original_constructor_gate), all(creusot, bytes_original_unique_gate))))]
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(by == 0usize))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(^self == *self))]
    #[inline]
    unsafe fn inc_start(&mut self, by: usize) {
        // should already be asserted, but debug assert for tests
        debug_assert!(self.len >= by, "internal: inc_start out of bounds");
        self.len -= by;
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        { self.ptr = self.ptr.add(by); }
        // Selected by == 0: exact same pointer, no new raw access boundary.
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        { proof_assert!(by == 0usize); }
    }
}

// Vtable must enforce this behavior
#[cfg(not(creusot))]
unsafe impl Send for Bytes {}
#[cfg(not(creusot))]
unsafe impl Sync for Bytes {}

impl Drop for Bytes {
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(&mut self.data, self.ptr, self.len) }
    }
}

impl Clone for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_bytes() == self.original_shared_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.ptr == self.ptr && result.len == self.len))]
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn clone(&self) -> Bytes {
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { original_shared_clone(self) }
        #[cfg(not(all(creusot, bytes_original_shared_gate)))]
        unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }
    }
}

impl Buf for Bytes {
    #[inline]
    fn remaining(&self) -> usize {
        self.len()
    }

    #[inline]
    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(cnt == 0usize))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(^self == *self))]
    #[inline]
    fn advance(&mut self, cnt: usize) {
        assert!(
            cnt <= self.len(),
            "cannot advance past `remaining`: {:?} <= {:?}",
            cnt,
            self.len(),
        );

        unsafe {
            self.inc_start(cnt);
        }
    }

    fn copy_to_bytes(&mut self, len: usize) -> Self {
        self.split_to(len)
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    #[cfg_attr(creusot, creusot_std::prelude::check(ghost))]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsRef<[u8]> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[inline]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl hash::Hash for Bytes {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_slice().hash(state);
    }
}

#[cfg(not(creusot))]
impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

#[cfg(not(creusot))]
impl IntoIterator for Bytes {
    type Item = u8;
    type IntoIter = IntoIter<Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl FromIterator<u8> for Bytes {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        Vec::from_iter(into_iter).into()
    }
}

// impl Eq

#[cfg(not(creusot))]
impl PartialEq for Bytes {
    fn eq(&self, other: &Bytes) -> bool {
        self.as_slice() == other.as_slice()
    }
}

#[cfg(not(creusot))]
impl PartialOrd for Bytes {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(not(creusot))]
impl Ord for Bytes {
    fn cmp(&self, other: &Bytes) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

#[cfg(not(creusot))]
impl Eq for Bytes {}

#[cfg(not(creusot))]
impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

#[cfg(not(creusot))]
impl PartialOrd<[u8]> for Bytes {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for [u8] {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<str> for Bytes {
    fn eq(&self, other: &str) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

#[cfg(not(creusot))]
impl PartialOrd<str> for Bytes {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for str {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for str {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<Vec<u8>> for Bytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Vec<u8>> for Bytes {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(&other[..])
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for Vec<u8> {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for Vec<u8> {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<String> for Bytes {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

#[cfg(not(creusot))]
impl PartialOrd<String> for Bytes {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for String {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for String {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for &[u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for &[u8] {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

#[cfg(not(creusot))]
impl PartialEq<Bytes> for &str {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

#[cfg(not(creusot))]
impl PartialOrd<Bytes> for &str {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

#[cfg(not(creusot))]
impl<'a, T: ?Sized> PartialEq<&'a T> for Bytes
where
    Bytes: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

#[cfg(not(creusot))]
impl<'a, T: ?Sized> PartialOrd<&'a T> for Bytes
where
    Bytes: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}

// impl From

impl Default for Bytes {
    #[inline]
    fn default() -> Bytes {
        Bytes::new()
    }
}

impl From<&'static [u8]> for Bytes {
    fn from(slice: &'static [u8]) -> Bytes {
        Bytes::from_static(slice)
    }
}

impl From<&'static str> for Bytes {
    fn from(slice: &'static str) -> Bytes {
        Bytes::from_static(slice.as_bytes())
    }
}

/// Construct the ordinary shared header for a Vec with spare capacity.
/// The proof profile carries the actual B1 allocation and the permissions
/// for the header, counter, and data cell; the native profile performs the
/// same Vec-to-Shared transfer and leaves the existing vtable lifecycle
/// unchanged.
#[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    vec@.len() < creusot_std::std::vec::capacity_model(vec)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.original_spare_shared_valid()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_spare_shared_bytes() == vec@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.ptr == creusot_std::std::vec::pointer_model(vec) as *const u8
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.len@ == vec@.len()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_spare_shared_capacity() == creusot_std::std::vec::capacity_model(vec)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.vtable == vtable))]
fn from_spare_capacity_vec(vec: Vec<u8>, vtable: &'static Vtable) -> Bytes {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (ptr, len, cap, raw, base, capabilities, contents) = {
        let contents = snapshot!(vec@);
        let (raw, len, capabilities) = raw_vec::detach_vec(vec);
        let (base, cap) = raw.bound_ptr_at_zero();
        (base.as_ptr(), len, cap, raw, base, capabilities, contents)
    };
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let (ptr, len, cap) = {
        let mut vec = ManuallyDrop::new(vec);
        (vec.as_mut_ptr(), vec.len(), vec.capacity())
    };

    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (ref_cnt, counter_owner) = RefCountCell::new_with_owner(1);
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let ref_cnt = RefCountCell::new(1);

    // Keep the native shared-header construction in one source body.
    let shared = Box::new(Shared { buf: ptr, cap, ref_cnt });
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (shared, shared_owner) = boxed_alignment::into_raw_aligned(shared);
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let shared = Box::into_raw(shared);

    #[cfg(all(creusot, bytes_original_unique_gate))]
    boxed_alignment::aligned_address_has_clear_low_bit(
        crate::provenance_specs::pointer_addr(shared), core::mem::align_of::<Shared>()
    );

    debug_assert!(
        0 == (crate::provenance_specs::pointer_addr(shared) & KIND_MASK),
        "internal: Box<Shared> should have an aligned pointer",
    );

    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (data, data_cell_permission) = {
        let (cell, permission) =
            crate::loom::sync::atomic::NativePtr::new_with_permission(shared.cast());
        cell.into_raw_with_permission(permission)
    };
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let data = BytesDataCell::new(shared.cast());

    let result = Bytes {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_packet: None,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_cell_permission: ghost!(None),
        #[cfg(all(creusot, bytes_original_unique_gate))]
        original_boxed: None,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        original_spare_shared: Some(OriginalSpareSharedProof {
            raw,
            base,
            capabilities,
            contents,
            shared,
            shared_owner,
            counter_owner,
            vtable,
            data_cell_permission,
        }),
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        original_frozen: None,
        ptr,
        len,
        data,
        vtable,
    };
    #[cfg(all(creusot, bytes_original_unique_gate))]
    {
        proof_assert!(result.original_spare_shared_allocation_valid());
        proof_assert!(result.original_spare_shared_tag_clear());
        proof_assert!(result.original_spare_shared_header_state_valid());
        proof_assert!(result.original_spare_shared_counter_valid());
        proof_assert!(result.original_spare_shared_cell_valid());
    }
    result
}

impl From<Vec<u8>> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(vec@.len() < creusot_std::std::vec::capacity_model(vec)))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_bytes() == vec@))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(vec@.len() < creusot_std::std::vec::capacity_model(vec)))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_bytes() == vec@))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_pointer() == creusot_std::std::vec::pointer_model(vec) as *const u8))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_content() == vec@))]
        #[cfg_attr(creusot, creusot::no_translate)]
    fn from(vec: Vec<u8>) -> Bytes {
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { return original_bytes_from_vec(vec); }
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { return original_shared_from_vec(vec); }
        #[cfg(not(all(creusot, any(bytes_original_shared_gate, bytes_original_constructor_gate))))]
        {
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let (len, cap) = (vec.len(), vec.capacity());
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        let (ptr, len, cap, base, capabilities) = {
            let (raw, len, capabilities) = raw_vec::detach_vec(vec);
            let (base, cap) = raw.into_bound_ptr_at_zero();
            (base.as_ptr(), len, cap, base, capabilities)
        };

        // Avoid an extra allocation if possible.
        if len == cap {
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            { unreachable!("selected spare-capacity Shared path"); }
            #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
            {
                if len == 0 {
                    return Bytes::from(vec.into_boxed_slice());
                }
                return from_full_capacity_vec(
                    vec, &PROMOTABLE_EVEN_VTABLE, &PROMOTABLE_ODD_VTABLE,
                );
            }
        }

        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        {
            return from_spare_capacity_vec(vec, &SHARED_VTABLE);
        }

        #[cfg(all(creusot, bytes_original_freeze_gate))]
        {
        let shared = Box::new(Shared {
            buf: ptr,
            cap,
            ref_cnt: RefCountCell::new(1),
        });

        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let shared = Box::into_raw(shared);
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        let (shared, shared_owner) = boxed_alignment::into_raw_aligned(shared);
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        boxed_alignment::aligned_address_has_clear_low_bit(
            crate::provenance_specs::pointer_addr(shared), core::mem::align_of::<Shared>()
        );
        // The pointer should be aligned, so this assert should
        // always succeed.
        debug_assert!(
            0 == (crate::provenance_specs::pointer_addr(shared) & KIND_MASK),
            "internal: Box<Shared> should have an aligned pointer",
        );
        Bytes {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_packet: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: Some(OriginalFrozenProof { base, capabilities, shared, shared_owner }),
            ptr,
            len,
            data: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_atomic_ptr_new(shared as _) }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { BytesDataCell::new(shared as _) }
            },
            vtable: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_shared_vtable() }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { &SHARED_VTABLE }
            },
        }
        }
        }
    }
}


/// Perform the native boxed-slice raw transfer shared by the B1 boundary and
/// the ordinary `Bytes` constructor.
pub(crate) fn boxed_slice_into_raw(slice: Box<[u8]>) -> *mut [u8] {
    Box::into_raw(slice)
}

#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), requires(
    ptr.addr_logic()@ % 2 == 0
))]
#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), ensures(
    result.addr_logic()@ == ptr.addr_logic()@ + 1
))]
#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), ensures(
    result == ptr.offset_logic(1)
))]
fn tagged_vec_data_pointer(ptr: *mut u8) -> *mut u8 {
    crate::provenance_specs::wrapping_offset(ptr, KIND_VEC)
}

#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), requires(
    ptr.addr_logic()@ % 2 == 1
))]
#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), ensures(
    result.addr_logic()@ + 1 == ptr.addr_logic()@
))]
#[cfg_attr(all(creusot, any(bytes_original_freeze_gate, bytes_original_unique_gate)), ensures(
    result == ptr.offset_logic(-1)
))]
fn untagged_vec_data_pointer(ptr: *mut u8) -> *mut u8 {
    crate::provenance_specs::signed_wrapping_offset(ptr, -1)
}

#[cfg_attr(creusot, requires(base_addr@ <= view_addr@))]
#[cfg_attr(creusot, requires(view_addr@ - base_addr@ + len@ <= usize::MAX@))]
#[cfg_attr(creusot, ensures(result@ == view_addr@ - base_addr@ + len@))]
#[inline]
fn boxed_slice_capacity_from_addresses(
    base_addr: usize,
    view_addr: usize,
    len: usize,
) -> usize {
    view_addr - base_addr + len
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(base.invariant()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(base.raw_pointer() == ptr))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    capabilities.inner_logic().0.invariant() && capabilities.inner_logic().1.invariant()
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    base@ == Some((capabilities.inner_logic().0.namespace(), len@, 0int))
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    capabilities.inner_logic().0.capacity() == len@ &&
    capabilities.inner_logic().1.capacity() == len@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace() &&
    capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace()
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    capabilities.inner_logic().1.lo() == 0 && capabilities.inner_logic().1.hi() == len@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    forall<i:Int> 0 <= i && i < len@ ==>
        capabilities.inner_logic().1.slot(i) == Some(Some((*bytes)[i]))
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires((*bytes).len() == len@))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.original_boxed_valid()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_bytes() == *bytes
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.ptr == ptr as *const u8))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_vtable() == if ptr.addr_logic()@ % 2 == 0 { even_vtable } else { odd_vtable }
))]
fn promotable_from_raw_parts(
    ptr: *mut u8,
    len: usize,
    even_vtable: &'static Vtable,
    odd_vtable: &'static Vtable,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    bytes: Snapshot<Seq<u8>>,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    base: BoundPtr,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    capabilities: Ghost<(Recovery, PhysicalRegion)>,
) -> Bytes {
    let even_pointer = crate::provenance_specs::pointer_addr(ptr) % 2 == 0;
    let tagged_data = if even_pointer { tagged_vec_data_pointer(ptr) } else { ptr };
    let vtable = if even_pointer { even_vtable } else { odd_vtable };
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (cell, permission) =
        crate::loom::sync::atomic::NativePtr::new_with_permission(tagged_data.cast());
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (data, permission) = cell.into_raw_with_permission(permission);
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let data = BytesDataCell::new(tagged_data.cast());
    Bytes {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_packet: None,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_cell_permission: ghost!(None),
        #[cfg(all(creusot, bytes_original_unique_gate))]
        original_boxed: Some(OriginalBoxedProof {
            base,
            capabilities,
            contents: bytes,
            even_pointer,
            vtable,
            data_cell_permission: permission,
        }),
        #[cfg(all(creusot, bytes_original_unique_gate))]
        original_spare_shared: None,
        #[cfg(all(creusot, bytes_original_freeze_gate))]
        original_frozen: None,
        ptr,
        len,
        data,
        vtable,
    }
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(slice@.len() > 0))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.original_boxed_valid()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_bytes() == slice@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_vtable() == if result.ptr.addr_logic()@ % 2 == 0 { even_vtable } else { odd_vtable }
))]
fn from_boxed_slice_inner(
    slice: Box<[u8]>,
    even_vtable: &'static Vtable,
    odd_vtable: &'static Vtable,
) -> Bytes {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    {
        let contents = snapshot!(slice@);
        let len = slice.len();
        let (raw, _captured_len, capabilities) = raw_vec::detach_boxed_slice(slice);
        let (base, _capacity) = raw.into_bound_ptr_at_zero();
        return promotable_from_raw_parts(
            base.as_ptr(), len, even_vtable, odd_vtable, contents, base, capabilities,
        );
    }

    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    {
        // Box<[u8]> does not allocate for empty slices, so there is no aligned
        // pointer available for KIND_VEC stashing.
        if slice.is_empty() {
            return Bytes::new();
        }
        let len = slice.len();
        let ptr = boxed_slice_into_raw(slice) as *mut u8;
        promotable_from_raw_parts(ptr, len, even_vtable, odd_vtable)
    }
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(input@.len() > 0))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(
    input@.len() == creusot_std::std::vec::capacity_model(input)
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.original_boxed_valid()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_bytes() == input@
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.ptr == creusot_std::std::vec::pointer_model(input) as *const u8
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.original_boxed_vtable() == if result.ptr.addr_logic()@ % 2 == 0 { even_vtable } else { odd_vtable }
))]
fn from_full_capacity_vec(
    input: Vec<u8>,
    even_vtable: &'static Vtable,
    odd_vtable: &'static Vtable,
) -> Bytes {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    {
        let contents = snapshot!(input@);
        let (raw, len, capabilities) = raw_vec::detach_full_capacity_vec(input);
        let (base, _capacity) = raw.into_bound_ptr_at_zero();
        return promotable_from_raw_parts(
            base.as_ptr(), len, even_vtable, odd_vtable, contents, base, capabilities,
        );
    }
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    {
        from_boxed_slice_inner(input.into_boxed_slice(), even_vtable, odd_vtable)
    }
}

impl From<Box<[u8]>> for Bytes {
    #[cfg_attr(creusot, creusot::no_translate)]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_content() == slice@))]
    fn from(slice: Box<[u8]>) -> Bytes {
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { return original_bytes_from_box(slice); }
        #[cfg(all(creusot, bytes_original_unique_gate))]
        {
            if slice.is_empty() {
                return Bytes::new();
            }
            from_boxed_slice_inner(
                slice, &PROMOTABLE_EVEN_VTABLE, &PROMOTABLE_ODD_VTABLE,
            )
        }
        #[cfg(not(all(creusot, any(bytes_original_constructor_gate, bytes_original_unique_gate))))]
        {
            from_boxed_slice_inner(
                slice, &PROMOTABLE_EVEN_VTABLE, &PROMOTABLE_ODD_VTABLE,
            )
        }
    }
}

impl From<Bytes> for BytesMut {
    /// Convert self into `BytesMut`.
    ///
    /// If `bytes` is unique for the entire original buffer, this will return a
    /// `BytesMut` with the contents of `bytes` without copying.
    /// If `bytes` is not unique for the entire original buffer, this will make
    /// a copy of `bytes` subset of the original buffer in a new `BytesMut`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bytes::{Bytes, BytesMut};
    ///
    /// let bytes = Bytes::from(b"hello".to_vec());
    /// assert_eq!(BytesMut::from(bytes), BytesMut::from(&b"hello"[..]));
    /// ```
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from(bytes: Bytes) -> Self {
        let bytes = ManuallyDrop::new(bytes);
        unsafe { (bytes.vtable.into_mut)(&bytes.data, bytes.ptr, bytes.len) }
    }
}

impl From<String> for Bytes {
    fn from(s: String) -> Bytes {
        Bytes::from(s.into_bytes())
    }
}

impl From<Bytes> for Vec<u8> {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn from(bytes: Bytes) -> Vec<u8> {
        let bytes = ManuallyDrop::new(bytes);
        unsafe { (bytes.vtable.into_vec)(&bytes.data, bytes.ptr, bytes.len) }
    }
}

// ===== impl Vtable =====

impl fmt::Debug for Vtable {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vtable")
            .field("clone", &(self.clone as *const ()))
            .field("drop", &(self.drop as *const ()))
            .finish()
    }
}

// ===== impl StaticVtable =====

const STATIC_VTABLE: Vtable = Vtable {
    clone: static_clone,
    into_vec: static_to_vec,
    into_mut: static_to_mut,
    is_unique: static_is_unique,
    drop: static_drop,
};

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn static_clone(_: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let slice = slice::from_raw_parts(ptr, len);
    Bytes::from_static(slice)
}

unsafe fn static_to_vec(_: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    let slice = slice::from_raw_parts(ptr, len);
    slice.to_vec()
}

unsafe fn static_to_mut(_: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    let slice = slice::from_raw_parts(ptr, len);
    BytesMut::from(slice)
}

fn static_is_unique(_: &AtomicPtr<()>) -> bool {
    false
}

unsafe fn static_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) {
    // nothing to drop for &'static [u8]
}

// ===== impl OwnedVtable =====

#[repr(C)]
struct Owned<T> {
    ref_cnt: RefCountCell,
    owner: T,
}

impl<T> Owned<T> {
    const VTABLE: Vtable = Vtable {
        clone: owned_clone::<T>,
        into_vec: owned_to_vec::<T>,
        into_mut: owned_to_mut::<T>,
        is_unique: owned_is_unique,
        drop: owned_drop::<T>,
    };
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_clone<T>(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let owned = data.load(Ordering::Relaxed);
    crate::ref_count_ops::increment(&*owned.cast::<RefCountCell>());

    Bytes {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_packet: None,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,

        #[cfg(all(creusot, bytes_original_freeze_gate))]

        original_frozen: None,
        ptr,
        len,
        data: BytesDataCell::new(owned as _),
        vtable: &Owned::<T>::VTABLE,
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_to_vec<T>(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    let slice = slice::from_raw_parts(ptr, len);
    let vec = slice.to_vec();
    owned_drop_impl::<T>(data.load(Ordering::Relaxed));
    vec
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_to_mut<T>(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    BytesMut::from_vec(owned_to_vec::<T>(data, ptr, len))
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_is_unique(_data: &AtomicPtr<()>) -> bool {
    false
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_drop_impl<T>(owned: *mut ()) {
    {
        let ref_cnt = &*owned.cast::<RefCountCell>();

        let old_cnt = ref_cnt.fetch_sub_release();
        debug_assert!(
            old_cnt > 0 && old_cnt <= crate::ref_count_limit::MAX_REF_COUNT + 1,
            "expected non-zero refcount and no underflow"
        );
        if old_cnt != 1 {
            return;
        }
        ref_cnt.load_acquire();
    }

    drop(Box::<Owned<T>>::from_raw(owned.cast()));
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn owned_drop<T>(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    let owned = data.load(Ordering::Relaxed);
    owned_drop_impl::<T>(owned);
}

// ===== impl PromotableVtable =====

#[cfg(not(creusot))]
static PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {
    clone: promotable_even_clone,
    into_vec: promotable_even_to_vec,
    into_mut: promotable_even_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_even_drop,
};

// Creusot cannot translate address-bearing static vtables. In verification builds only, use
// equivalent constant values; all methods that inspect their identity are explicitly excluded.
#[cfg(creusot)]
const PROMOTABLE_EVEN_VTABLE: Vtable = Vtable {
    clone: promotable_even_clone,
    into_vec: promotable_even_to_vec,
    into_mut: promotable_even_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_even_drop,
};

#[cfg(not(creusot))]
static PROMOTABLE_ODD_VTABLE: Vtable = Vtable {
    clone: promotable_odd_clone,
    into_vec: promotable_odd_to_vec,
    into_mut: promotable_odd_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_odd_drop,
};

#[cfg(creusot)]
const PROMOTABLE_ODD_VTABLE: Vtable = Vtable {
    clone: promotable_odd_clone,
    into_vec: promotable_odd_to_vec,
    into_mut: promotable_odd_to_mut,
    is_unique: promotable_is_unique,
    drop: promotable_odd_drop,
};

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_even_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;

    if kind == KIND_ARC {
        shallow_clone_arc(shared.cast(), ptr, len)
    } else {
        debug_assert_eq!(kind, KIND_VEC);
        let buf = untagged_vec_data_pointer(shared.cast());
        shallow_clone_vec(data, shared, buf, ptr, len)
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_to_vec(
    data: &AtomicPtr<()>,
    ptr: *const u8,
    len: usize,
    f: fn(*mut ()) -> *mut u8,
) -> Vec<u8> {
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;

    if kind == KIND_ARC {
        shared_to_vec_impl(shared.cast(), ptr, len)
    } else {
        // If Bytes holds a Vec, then the offset must be 0.
        debug_assert_eq!(kind, KIND_VEC);

        let buf = f(shared);

        let cap = ptr.offset_from(buf) as usize + len;

        // Copy back buffer
        ptr::copy(ptr, buf, len);

        Vec::from_raw_parts(buf, len, cap)
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_to_mut(
    data: &AtomicPtr<()>,
    ptr: *const u8,
    len: usize,
    f: fn(*mut ()) -> *mut u8,
) -> BytesMut {
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;

    if kind == KIND_ARC {
        shared_to_mut_impl(
            shared.cast(),
            ptr,
            len,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        )
    } else {
        // KIND_VEC is a view of an underlying buffer at a certain offset.
        // The ptr + len always represents the end of that buffer.
        // Before truncating it, it is first promoted to KIND_ARC.
        // Thus, we can safely reconstruct a Vec from it without leaking memory.
        debug_assert_eq!(kind, KIND_VEC);

        let buf = f(shared);
        let off = ptr.offset_from(buf) as usize;
        let cap = off + len;
        let v = Vec::from_raw_parts(buf, cap, cap);

        let mut b = BytesMut::from_vec(v);
        b.advance_unchecked(
            off,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        );
        b
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_even_to_vec(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    promotable_to_vec(data, ptr, len, |shared| {
        untagged_vec_data_pointer(shared.cast())
    })
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_even_to_mut(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    promotable_to_mut(data, ptr, len, |shared| {
        untagged_vec_data_pointer(shared.cast())
    })
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_even_drop(data: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    data.with_mut(|shared| {
        let shared = *shared;
        let kind = shared as usize & KIND_MASK;

        if kind == KIND_ARC {
            release_shared(
                shared.cast(),
                #[cfg(all(creusot, bytes_original_unique_gate))]
                None,
            );
        } else {
            debug_assert_eq!(kind, KIND_VEC);
            let buf = untagged_vec_data_pointer(shared.cast());
            free_boxed_slice(
                buf,
                ptr,
                len,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                None,
            );
        }
    });
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_odd_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;

    if kind == KIND_ARC {
        shallow_clone_arc(shared as _, ptr, len)
    } else {
        debug_assert_eq!(kind, KIND_VEC);
        shallow_clone_vec(data, shared, shared.cast(), ptr, len)
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_odd_to_vec(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    promotable_to_vec(data, ptr, len, |shared| shared.cast())
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_odd_to_mut(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    promotable_to_mut(data, ptr, len, |shared| shared.cast())
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_odd_drop(data: &mut AtomicPtr<()>, ptr: *const u8, len: usize) {
    data.with_mut(|shared| {
        let shared = *shared;
        let kind = shared as usize & KIND_MASK;

        if kind == KIND_ARC {
            release_shared(
                shared.cast(),
                #[cfg(all(creusot, bytes_original_unique_gate))]
                None,
            );
        } else {
            debug_assert_eq!(kind, KIND_VEC);

            free_boxed_slice(
                shared.cast(),
                ptr,
                len,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                None,
            );
        }
    });
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn promotable_is_unique(data: &AtomicPtr<()>) -> bool {
    let shared = data.load(Ordering::Acquire);
    let kind = shared as usize & KIND_MASK;

    if kind == KIND_ARC {
        let ref_cnt = (*shared.cast::<Shared>()).ref_cnt.load_relaxed();
        ref_cnt == 1
    } else {
        true
    }
}

#[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match proof {
    Some(release) => {
        release.base.invariant() && release.view.invariant() &&
        release.capabilities.inner_logic().0.invariant() &&
        release.capabilities.inner_logic().1.invariant() &&
        buf == release.base.raw_pointer() &&
        offset == release.view.raw_pointer() as *const u8 &&
        release.base@ == Some((release.capabilities.inner_logic().0.namespace(),
            release.capabilities.inner_logic().0.capacity(), 0int)) &&
        release.view@ == Some((release.capabilities.inner_logic().0.namespace(),
            release.capabilities.inner_logic().0.capacity(), release.view_offset@)) &&
        release.base.current_address() + release.view_offset@ == release.view.current_address() &&
        release.view_offset@ + len@ == release.capabilities.inner_logic().0.capacity() &&
        buf.addr_logic()@ <= offset.addr_logic()@ &&
        offset.addr_logic()@ - buf.addr_logic()@ + len@ <= usize::MAX@ &&
        release.capabilities.inner_logic().1.capacity() == release.capabilities.inner_logic().0.capacity() &&
        release.capabilities.inner_logic().1.namespace() == release.capabilities.inner_logic().0.namespace() &&
        release.capabilities.inner_logic().1.resource_id() == release.capabilities.inner_logic().0.namespace() &&
        release.capabilities.inner_logic().1.lo() == 0 &&
        release.capabilities.inner_logic().1.hi() == release.capabilities.inner_logic().0.capacity()
    },
    None => false,
}))]
unsafe fn free_boxed_slice(
    buf: *mut u8,
    offset: *const u8,
    len: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    proof: Option<BoxedReleaseProof>,
) {
    let cap = boxed_slice_capacity_from_addresses(
        crate::provenance_specs::pointer_addr(buf),
        crate::provenance_specs::pointer_addr(offset.cast_mut()),
        len,
    );
    #[cfg(all(creusot, bytes_original_unique_gate))]
    if let Some(release) = proof {
        let described_offset = release.view.offset_from_bound_base(release.base);
        proof_assert!(described_offset@ == release.view_offset@);
        proof_assert!(buf.addr_logic()@ == release.base.current_address());
        proof_assert!(offset.addr_logic()@ == release.view.current_address());
        proof_assert!(cap@ == release.view_offset@ + len@);
        proof_assert!(cap@ == release.capabilities.inner_logic().0.capacity());
        unsafe { raw_vec::deallocate_bound_vec(release.base, cap, release.capabilities) };
    }
    #[cfg(not(creusot))]
    crate::allocation_ops::deallocate_u8(buf, cap)
}

// ===== impl SharedVtable =====

include!("bytes/shared_record.rs");

impl Drop for Shared {
    #[cfg_attr(creusot, creusot::no_translate)]
    fn drop(&mut self) {
        unsafe { dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap()) }
    }
}

// Assert that the alignment of `Shared` is divisible by 2.
// This is a necessary invariant since we depend on allocating `Shared` a
// shared object to implicitly carry the `KIND_ARC` flag in its pointer.
// This flag is set when the LSB is 0.
const _: [(); 0 - mem::align_of::<Shared>() % 2] = []; // Assert that the alignment of `Shared` is divisible by 2.

#[cfg(not(creusot))]
static SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};
// This proof-build constant supports Rust typechecking of excluded Shared-vtable
// lifecycle code. Its address and vtable transitions are not part of the proof.
#[cfg(creusot)]
const SHARED_VTABLE: Vtable = Vtable {
    clone: shared_clone,
    into_vec: shared_to_vec,
    into_mut: shared_to_mut,
    is_unique: shared_is_unique,
    drop: shared_drop,
};
const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
    let shared = data.load(Ordering::Relaxed);
    shallow_clone_arc(shared as _, ptr, len)
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_to_vec_impl(shared: *mut Shared, ptr: *const u8, len: usize) -> Vec<u8> {
    // Check that the ref_cnt is 1 (unique).
    //
    // If it is unique, then it is set to 0 with AcqRel fence for the same
    // reason in release_shared.
    //
    // Otherwise, we take the other branch and call release_shared.
    if (*shared)
        .ref_cnt
        .compare_exchange_acqrel_relaxed(1, 0)
        .is_ok()
    {
        // Deallocate the `Shared` instance without running its destructor.
        let shared = *Box::from_raw(shared);
        let shared = ManuallyDrop::new(shared);
        let buf = shared.buf;
        let cap = shared.cap;

        // Copy back buffer
        ptr::copy(ptr, buf, len);

        Vec::from_raw_parts(buf, len, cap)
    } else {
        let v = slice::from_raw_parts(ptr, len).to_vec();
        release_shared(
            shared,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        );
        v
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_to_vec(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Vec<u8> {
    shared_to_vec_impl(data.load(Ordering::Relaxed).cast(), ptr, len)
}

#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(false))]
#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_to_mut_nonunique(
    shared: *mut Shared,
    ptr: *const u8,
    len: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    proof: Option<OriginalSpareSharedProof>,
) -> BytesMut {
    // Keep the original native copy-and-release fallback in one source body.
    let v = slice::from_raw_parts(ptr, len).to_vec();
    release_shared(
        shared,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        proof,
    );
    BytesMut::from_vec(v)
}

#[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match proof {
    Some(release) => release.take_mut_valid(shared, ptr, len),
    None => false,
}))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_vec_rebuildable()))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(
    result.unique_bytes() == match proof {
        Some(release) => *release.contents,
        None => Seq::empty(),
    }
))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_length() == len@))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_offset() == 0))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(match proof {
    Some(release) => result.unique_capacity() == release.raw.capacity(),
    None => false,
}))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(match proof {
    Some(release) => result.unique_allocation_capacity() == release.raw.capacity(),
    None => false,
}))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(result.unique_pointer() == ptr as *mut u8))]
#[cfg_attr(all(creusot, bytes_original_unique_gate), ensures(forall<i:Int>
    result.unique_length() <= i && i < result.unique_capacity() ==>
        result.unique_slot(i) == Some(None)
))]
unsafe fn shared_to_mut_impl(
    shared: *mut Shared,
    ptr: *const u8,
    len: usize,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    proof: Option<OriginalSpareSharedProof>,
) -> BytesMut {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let mut proof = proof.unwrap();

    // The goal is to check if the current handle is the only handle
    // that currently has access to the buffer. This is done by
    // checking if the `ref_cnt` is currently 1.
    //
    // The `Acquire` ordering synchronizes with the `Release` as
    // part of the `fetch_sub` in `release_shared`. The `fetch_sub`
    // operation guarantees that any mutations done in other threads
    // are ordered before the `ref_cnt` is decremented. As such,
    // this `Acquire` will guarantee that those mutations are
    // visible to the current thread.
    //
    // Otherwise, we take the other branch, copy the data and call `release_shared`.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (ref_count, counter_owner) = {
        let shared_ref = unsafe {
            creusot_std::ghost::perm::Perm::as_ref(shared, ghost!(&*proof.shared_owner))
        };
        shared_ref.ref_cnt.native_counter().load_acquire_owned(proof.counter_owner)
    };
    #[cfg(all(creusot, bytes_original_unique_gate))]
    { proof.counter_owner = counter_owner; }
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let ref_count = (*shared).ref_cnt.load_acquire();

    if ref_count == 1 {
        // Deallocate the `Shared` instance without running its destructor.
        let (buf, cap) = {
            #[cfg(all(creusot, bytes_original_unique_gate))]
            let shared_ref = unsafe {
                creusot_std::ghost::perm::Perm::as_ref(shared, ghost!(&*proof.shared_owner))
            };
            #[cfg(not(all(creusot, bytes_original_unique_gate)))]
            let shared_ref = unsafe { &*shared };
            (shared_ref.buf, shared_ref.cap)
        };

        // The original Box move freed only the header allocation at this
        // point; Shared's destructor was suppressed. Use the same generic
        // no-drop Box boundary before computing the byte-pointer offset.
        #[cfg(all(creusot, bytes_original_unique_gate))]
        unsafe {
            crate::allocation_ops::deallocate_box_without_drop(shared, proof.shared_owner)
        };
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        unsafe { crate::allocation_ops::deallocate_box_without_drop(shared) };

        // Rebuild Vec
        let off = unsafe {
            crate::provenance_specs::offset_from_equal_u8(ptr, buf)
        } as usize;
        #[cfg(all(creusot, bytes_original_unique_gate))]
        let v = {
            proof_assert!(buf == proof.base.raw_pointer());
            proof_assert!(cap@ == proof.raw.capacity());
            unsafe { raw_vec::resume_vec(proof.raw, len + off, proof.capabilities) }
        };
        #[cfg(not(all(creusot, bytes_original_unique_gate)))]
        let v = unsafe { Vec::from_raw_parts(buf, len + off, cap) };

        let mut b = BytesMut::from_vec(v);
        b.advance_unchecked(
            off,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        );
        b
    } else {
        // The native fallback is outside the count-one proof carrier's domain.
        unsafe { shared_to_mut_nonunique(
            shared,
            ptr,
            len,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            Some(proof),
        ) }
    }
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_to_mut(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> BytesMut {
    shared_to_mut_impl(
        data.load(Ordering::Relaxed).cast(),
        ptr,
        len,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        None,
    )
}

#[cfg_attr(creusot, creusot::no_translate)]
pub(crate) unsafe fn shared_is_unique(data: &AtomicPtr<()>) -> bool {
    let shared = data.load(Ordering::Acquire);
    let ref_cnt = (*shared.cast::<Shared>()).ref_cnt.load_relaxed();
    ref_cnt == 1
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shared_drop(data: &mut AtomicPtr<()>, _ptr: *const u8, _len: usize) {
    data.with_mut(|shared| {
        release_shared(
            shared.cast(),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            None,
        );
    });
}

#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shallow_clone_arc(shared: *mut Shared, ptr: *const u8, len: usize) -> Bytes {
    crate::ref_count_ops::increment(&(*shared).ref_cnt);

    Bytes {
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_packet: None,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,

        #[cfg(all(creusot, bytes_original_freeze_gate))]

        original_frozen: None,
        ptr,
        len,
        data: BytesDataCell::new(shared as _),
        vtable: &SHARED_VTABLE,
    }
}

#[cold]
#[cfg_attr(creusot, creusot::no_translate)]
unsafe fn shallow_clone_vec(
    atom: &AtomicPtr<()>,
    ptr: *const (),
    buf: *mut u8,
    offset: *const u8,
    len: usize,
) -> Bytes {
    // If the buffer is still tracked in a `Vec<u8>`. It is time to
    // promote the vec to an `Arc`. This could potentially be called
    // concurrently, so some care must be taken.

    // First, allocate a new `Shared` instance containing the
    // `Vec` fields. It's important to note that `ptr`, `len`,
    // and `cap` cannot be mutated without having `&mut self`.
    // This means that these fields will not be concurrently
    // updated and since the buffer hasn't been promoted to an
    // `Arc`, those three fields still are the components of the
    // vector.
    let shared = Box::new(Shared {
        buf,
        cap: offset.offset_from(buf) as usize + len,
        // Initialize refcount to 2. One for this reference, and one
        // for the new clone that will be returned from
        // `shallow_clone`.
        ref_cnt: RefCountCell::new(2),
    });

    let shared = Box::into_raw(shared);

    // The pointer should be aligned, so this assert should
    // always succeed.
    debug_assert!(
        0 == (shared as usize & KIND_MASK),
        "internal: Box<Shared> should have an aligned pointer",
    );

    // Try compare & swapping the pointer into the `arc` field.
    // `Release` is used synchronize with other threads that
    // will load the `arc` field.
    //
    // If the `compare_exchange` fails, then the thread lost the
    // race to promote the buffer to shared. The `Acquire`
    // ordering will synchronize with the `compare_exchange`
    // that happened in the other thread and the `Shared`
    // pointed to by `actual` will be visible.
    match atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire) {
        Ok(actual) => {
            debug_assert!(core::ptr::eq(actual, ptr));
            // The upgrade was successful, the new handle can be
            // returned.
            Bytes {
                #[cfg(all(creusot, bytes_original_unique_gate))]
                readonly_packet: None,
                #[cfg(all(creusot, bytes_original_unique_gate))]
                readonly_cell_permission: ghost!(None),
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_boxed: None,
            #[cfg(all(creusot, bytes_original_unique_gate))]
            original_spare_shared: None,
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr: offset,
                len,
                data: BytesDataCell::new(shared as _),
                vtable: &SHARED_VTABLE,
            }
        }
        Err(actual) => {
            // The upgrade failed, a concurrent clone happened. Release
            // the allocation that was made in this thread, it will not
            // be needed.
            let shared = Box::from_raw(shared);
            mem::forget(*shared);

            // Buffer already promoted to shared storage, so increment ref
            // count.
            shallow_clone_arc(actual as _, offset, len)
        }
    }
}

#[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match proof {
    Some(release) => release.release_valid(ptr, 1usize),
    None => false,
}))]
unsafe fn release_shared(
    ptr: *mut Shared,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    proof: Option<OriginalSpareSharedProof>,
) {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let mut proof = proof.unwrap();

    // In the checked carrier path, inspect the header through its actual Box
    // permission. Other paths preserve the original unverified raw access.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let shared_ref = unsafe {
        creusot_std::ghost::perm::Perm::as_ref(ptr, ghost!(&*proof.shared_owner))
    };

    // `Shared` storage... follow the drop steps from Arc.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (old_count, counter_owner) = shared_ref
        .ref_cnt
        .native_counter()
        .fetch_sub_release_owned(proof.counter_owner);
    #[cfg(all(creusot, bytes_original_unique_gate))]
    { proof.counter_owner = counter_owner; }
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let old_count = (*ptr).ref_cnt.fetch_sub_release();

    if old_count != 1 {
        return;
    }

    // This fence is needed to prevent reordering of use of the data and
    // deletion of the data.  Because it is marked `Release`, the decreasing
    // of the reference count synchronizes with this `Acquire` fence. This
    // means that use of the data happens before decreasing the reference
    // count, which happens before this fence, which happens before the
    // deletion of the data.
    //
    // As explained in the [Boost documentation][1],
    //
    // > It is important to enforce any possible access to the object in one
    // > thread (through an existing reference) to *happen before* deleting
    // > the object in a different thread. This is achieved by a "release"
    // > operation after dropping a reference (any access to the object
    // > through this reference must obviously happened before), and an
    // > "acquire" operation before deleting the object.
    //
    // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
    //
    // Thread sanitizer does not support atomic fences. Use an atomic load
    // instead.
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let (_observed_count, counter_owner) = shared_ref
        .ref_cnt
        .native_counter()
        .load_acquire_owned(proof.counter_owner);
    #[cfg(all(creusot, bytes_original_unique_gate))]
    { proof.counter_owner = counter_owner; }
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    (*ptr).ref_cnt.load_acquire();

    // Explicit cleanup has the same payload/control effects as dropping the Box.
    free_shared(
        ptr,
        #[cfg(all(creusot, bytes_original_unique_gate))]
        Some(proof),
    );
}

// The explicit final cleanup below bypasses Shared::drop. All of its fields
// are non-owning scalar/atomic values; re-evaluate it if an owning field is added.
// In particular a substituted atomic type must not carry destructor effects.
const _: [(); 0] = [(); mem::needs_drop::<AtomicUsize>() as usize];

#[cfg_attr(all(creusot, not(bytes_original_unique_gate)), creusot::no_translate)]
#[cfg_attr(all(creusot, bytes_original_unique_gate), requires(match proof {
    Some(release) => release.release_valid(ptr, 0usize),
    None => false,
}))]
unsafe fn free_shared(
    ptr: *mut Shared,
    #[cfg(all(creusot, bytes_original_unique_gate))]
    proof: Option<OriginalSpareSharedProof>,
) {
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let release = proof.unwrap();
    #[cfg(all(creusot, bytes_original_unique_gate))]
    let shared_ref = unsafe {
        creusot_std::ghost::perm::Perm::as_ref(ptr, ghost!(&*release.shared_owner))
    };
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    let shared_ref = unsafe { &*ptr };

    // SAFETY: release_shared has recovered the sole reference after Acquire.
    // The allocation was made by Box<Shared>; buf/cap describe its separate
    // global-allocator byte allocation. Preserve the original destruction order.
    let buf = shared_ref.buf;
    let cap = shared_ref.cap;
    #[cfg(all(creusot, bytes_original_unique_gate))]
    {
        proof_assert!(release.shared == ptr);
        proof_assert!(buf == release.base.raw_pointer());
        proof_assert!(cap@ == release.capabilities.inner_logic().0.capacity());
        unsafe { raw_vec::deallocate_bound_vec(release.base, cap, release.capabilities) };
        unsafe {
            crate::allocation_ops::deallocate_box_without_drop(ptr, release.shared_owner)
        };
    }
    #[cfg(not(all(creusot, bytes_original_unique_gate)))]
    {
        unsafe { crate::allocation_ops::deallocate_u8(buf, cap) };
        unsafe { crate::allocation_ops::deallocate_box_without_drop(ptr) };
    }
}

fn without_provenance(ptr: usize) -> *const u8 {
    core::ptr::null::<u8>().wrapping_add(ptr)
}

// compile-fails

/// ```compile_fail
/// use bytes::Bytes;
/// #[deny(unused_must_use)]
/// {
///     let mut b1 = Bytes::from("hello world");
///     b1.split_to(6);
/// }
/// ```
fn _split_to_must_use() {}

/// ```compile_fail
/// use bytes::Bytes;
/// #[deny(unused_must_use)]
/// {
///     let mut b1 = Bytes::from("hello world");
///     b1.split_off(6);
/// }
/// ```
fn _split_off_must_use() {}

// fuzz tests
#[cfg(all(test, loom))]
mod fuzz {
    use loom::sync::Arc;
    use loom::thread;

    use super::Bytes;
    #[test]
    fn bytes_cloning_vec() {
        loom::model(|| {
            let a = Bytes::from(b"abcdefgh".to_vec());
            let addr = a.as_ptr() as usize;

            // test the Bytes::clone is Sync by putting it in an Arc
            let a1 = Arc::new(a);
            let a2 = a1.clone();

            let t1 = thread::spawn(move || {
                let b: Bytes = (*a1).clone();
                assert_eq!(b.as_ptr() as usize, addr);
            });

            let t2 = thread::spawn(move || {
                let b: Bytes = (*a2).clone();
                assert_eq!(b.as_ptr() as usize, addr);
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}

#[cfg(all(test, not(loom)))]
mod original_release_tests {
    use super::Bytes;
    use crate::Buf;
    use std::vec::Vec;

    #[test]
    fn explicitly_dispose_spare_capacity_shared_handle() {
        let mut input = Vec::with_capacity(32);
        input.extend_from_slice(b"spare capacity payload");
        assert!(input.capacity() > input.len());

        let bytes = Bytes::from(input);
        assert_eq!(&bytes[..], b"spare capacity payload");

        // SAFETY: this handle came directly from the spare-capacity Vec
        // constructor and has not been cloned or otherwise transferred.
        unsafe { bytes.dispose_spare_shared() };
    }

    #[test]
    fn try_into_mut_reclaims_spare_capacity_shared_buffer_at_offsets() {
        let mut input = Vec::with_capacity(32);
        input.extend_from_slice(b"spare capacity payload");
        let capacity = input.capacity();

        let mut bytes = Bytes::from(input);
        bytes.advance(6);
        let mut bytes_mut = bytes
            .try_into_mut()
            .expect("the unshared spare-capacity buffer is unique");
        assert_eq!(&bytes_mut[..], b"capacity payload");
        assert_eq!(bytes_mut.capacity(), capacity - 6);

        bytes_mut.truncate(5);
        assert_eq!(&bytes_mut[..], b"capac");

        let mut empty_input = Vec::with_capacity(24);
        empty_input.extend_from_slice(b"empty view");
        let empty_capacity = empty_input.capacity();
        let end = empty_input.len();
        let mut empty_bytes = Bytes::from(empty_input);
        empty_bytes.advance(end);
        assert!(empty_bytes.is_empty());

        let empty_mut = empty_bytes
            .try_into_mut()
            .expect("the empty spare-capacity view is unique");
        assert!(empty_mut.is_empty());
        assert_eq!(empty_mut.capacity(), empty_capacity - end);
    }
}
