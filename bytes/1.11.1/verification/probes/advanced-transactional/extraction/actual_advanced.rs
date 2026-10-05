use alloc::{boxed::Box, vec::Vec};
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use creusot_std::prelude::*;
use crate::capacity_ops::{original_capacity_to_repr, MAX_VEC_POS};
pub struct BytesMut {
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    ptr: NonNull<u8>,
    #[cfg(any(creusot, bytes_proof_probe))]
    ptr: crate::ownership_proof::raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    data: *mut Shared,
    // Present only in proof builds. Other construction paths remain outside
    // the offset-zero unique ownership gate until their protocols are proved.
    #[cfg(any(creusot, bytes_proof_probe))]
    unique_at_zero: Ghost<Option<(
        crate::ownership_proof::raw_vec::Recovery,
        crate::ownership_proof::raw_vec::PhysicalRegion,
    )>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    pending_control: Ghost<Option<sequential_shared_control::PendingControl>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    shared_registration: Ghost<Option<sequential_shared_control::HandleRegistration>>,
    #[cfg(any(creusot, bytes_proof_probe))]
    shared_context: Ghost<Option<sequential_shared_control::ControlContext>>,
}
struct Shared {
    buffer: SharedBuffer,
    original_capacity_repr: usize,
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    ref_count: AtomicUsize,
    #[cfg(any(creusot, bytes_proof_probe))]
    ref_count: crate::ownership_proof::sequential_counter::SequentialCounter,
}
struct SharedBuffer {
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    base: NonNull<u8>,
    #[cfg(any(creusot, bytes_proof_probe))]
    base: crate::ownership_proof::raw_vec::BoundPtr,
    capacity: usize,
}
const KIND_ARC: usize = 0b0;
const KIND_VEC: usize = 0b1;
const KIND_MASK: usize = 0b1;
#[inline]
#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    // This null-derived pointer stores integer metadata only. It carries no
    // allocation permission and must not be used as a dereferenceable pointer.
    let ptr = crate::provenance_specs::metadata_pointer(addr);
    debug_assert_eq!(crate::provenance_specs::pointer_addr(ptr), addr);
    ptr.cast::<T>()
}

#[cfg(all(creusot, bytes_proof_valid_handle))]
impl creusot_std::invariant::Invariant for BytesMut {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.proof_empty_valid() || (self.proof_unique_owned() && self.proof_initialized()) }
    }
}
impl BytesMut {
    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_view_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(self.ptr@.unwrap_logic().2 + index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(self.ptr@.unwrap_logic().2 + index),
                },
            }
        }
    }
    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_owned_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(index),
                },
            }
        }
    }
    #[cfg(creusot)]
    #[logic]
    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                None => None,
                Some((_, region)) => region.slot(index),
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_empty_valid(self) -> bool {
        pearlite! {
            self.len == 0usize && self.cap == 0usize &&
            self.data.addr_logic() == KIND_VEC && self.ptr.invariant() && self.ptr@ == None &&
            self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None
        }
    }
    #[cfg(all(creusot, bytes_proof_valid_handle))]
    #[logic(prophetic)]
    fn proof_owned_valid(self) -> bool {
        pearlite! {
            self.proof_unique_owned() || self.proof_registered_valid() || self.proof_empty_valid()
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_initialized(self) -> bool {
        pearlite! {
            self.proof_owned_valid() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_registered_valid(self) -> bool {
        pearlite! {
            self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None &&
            self.len <= self.cap && self.ptr.invariant() &&
            self.data.addr_logic() & KIND_MASK == KIND_ARC &&
            match self.shared_registration.inner_logic() {
                None => false,
                Some(registration) => registration.valid() && registration.control.pointer == self.data &&
                    self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == registration.status.allocation &&
                    self.ptr@.unwrap_logic().1 == registration.status.capacity &&
                    self.ptr@.unwrap_logic().2 + self.cap@ <= registration.status.capacity &&
                    registration.view_lo() <= self.ptr@.unwrap_logic().2 &&
                    self.ptr@.unwrap_logic().2 + self.cap@ ==
                        registration.view_hi(),
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_unique_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_VEC &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() && self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == recovery.namespace() &&
                    self.ptr@.unwrap_logic().1 == recovery.capacity() &&
                    self.ptr@.unwrap_logic().2 == (self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET)@ &&
                    self.ptr@.unwrap_logic().2 + self.cap@ == recovery.capacity() &&
                    recovery.invariant() && region.invariant() &&
                    region.capacity() == recovery.capacity() &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == recovery.capacity() &&
                    self.len <= self.cap
            }
        }
    }

    #[cfg_attr(creusot, requires(self.proof_unique_owned()))]
    #[cfg_attr(creusot, requires(count <= self.cap))]
    #[cfg_attr(creusot, requires(self.ptr@.unwrap_logic().2 + count@ <= MAX_VEC_POS@))]
    #[cfg_attr(creusot, ensures((^self).proof_unique_owned() && (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if count <= self.len { self.len@ - count@ } else { 0int })))]
    #[cfg_attr(creusot, ensures((^self).cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).proof_view_slot(index) == self.proof_view_slot(index + count@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    pub(crate) fn advance_transactionally(&mut self, count: usize) {
        let empty = BytesMut {
            ptr: crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling()),
            len: 0, cap: 0, data: invalid_ptr(crate::capacity_ops::KIND_VEC),
            unique_at_zero: ghost! { None }, pending_control: ghost! { None },
            shared_registration: ghost! { None }, shared_context: ghost! { None },
        };
        let old = mem::replace(self, empty);
        let raw = RawTransition::from_valid(old);
        let updated = raw.advance_to_valid(count);
        // `self` still holds the canonical empty descriptor. Replace it with
        // the fully armed result, then forget the resource-free placeholder
        // returned by mem::replace so BytesMut::drop cannot run on that shell.
        let placeholder = mem::replace(self, updated);
        mem::forget(placeholder);
    }
}

/// A private field carrier. It deliberately has no `Invariant` implementation.
struct RawTransition {
    ptr: crate::ownership_proof::raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    data: *mut Shared,
    unique_at_zero: Ghost<Option<(crate::ownership_proof::raw_vec::Recovery, crate::ownership_proof::raw_vec::PhysicalRegion)>>,
    pending_control: Ghost<Option<sequential_shared_control::PendingControl>>,
    shared_registration: Ghost<Option<sequential_shared_control::HandleRegistration>>,
    shared_context: Ghost<Option<sequential_shared_control::ControlContext>>,
}

impl RawTransition {
    #[cfg(creusot)]
    #[logic]
    fn proof_view_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(self.ptr@.unwrap_logic().2 + index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(self.ptr@.unwrap_logic().2 + index),
                },
            }
        }
    }

    #[cfg(creusot)]
    #[logic]
    fn proof_owned_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                Some((_, region)) => region.slot(index),
                None => match self.shared_registration.inner_logic() {
                    None => None,
                    Some(registration) => registration.packet.1.slot(index),
                },
            }
        }
    }

    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_unique_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & crate::capacity_ops::KIND_MASK == crate::capacity_ops::KIND_VEC &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() && self.ptr@ != None &&
                    self.ptr@.unwrap_logic().0 == recovery.namespace() &&
                    self.ptr@.unwrap_logic().1 == recovery.capacity() &&
                    self.ptr@.unwrap_logic().2 == (self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET)@ &&
                    self.ptr@.unwrap_logic().2 + self.cap@ == recovery.capacity() &&
                    recovery.invariant() && region.invariant() &&
                    region.capacity() == recovery.capacity() &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == recovery.capacity() &&
                    self.len <= self.cap
            }
        }
    }

    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_initialized(self) -> bool {
        pearlite! {
            self.proof_unique_owned() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))
        }
    }

    #[cfg_attr(creusot, requires(value.proof_unique_owned() && value.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result.proof_unique_owned() && result.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result.ptr == value.ptr && result.len == value.len && result.cap == value.cap && result.data == value.data))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero == value.unique_at_zero && result.pending_control == value.pending_control && result.shared_registration == value.shared_registration && result.shared_context == value.shared_context))]
    #[cfg_attr(creusot, ensures(forall<index: Int> result.proof_view_slot(index) == value.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> result.proof_owned_slot(index) == value.proof_owned_slot(index)))]
    fn from_valid(value: BytesMut) -> Self {
        let mut value = value;
        let raw = Self {
            ptr: mem::replace(&mut value.ptr, crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling())),
            len: mem::replace(&mut value.len, 0),
            cap: mem::replace(&mut value.cap, 0),
            data: mem::replace(&mut value.data, invalid_ptr(crate::capacity_ops::KIND_VEC)),
            unique_at_zero: mem::replace(&mut value.unique_at_zero, ghost! { None }),
            pending_control: mem::replace(&mut value.pending_control, ghost! { None }),
            shared_registration: mem::replace(&mut value.shared_registration, ghost! { None }),
            shared_context: mem::replace(&mut value.shared_context, ghost! { None }),
        };
        // Every field has been transferred into the raw carrier. Suppress the
        // emptied BytesMut shell's production Drop path, which assumes an
        // armed allocation capability even for a canonical empty descriptor.
        mem::forget(value);
        raw
    }

    #[cfg_attr(creusot, requires(self.proof_unique_owned() && self.proof_initialized()))]
    #[cfg_attr(creusot, requires(count <= self.cap))]
    #[cfg_attr(creusot, requires(self.ptr@.unwrap_logic().2 + count@ <= MAX_VEC_POS@))]
    #[cfg_attr(creusot, ensures(result.proof_unique_owned() && result.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result.len@ == (if count <= self.len { self.len@ - count@ } else { 0int })))]
    #[cfg_attr(creusot, ensures(result.cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures(result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero == self.unique_at_zero))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures(result.pending_control == self.pending_control && result.shared_registration == self.shared_registration && result.shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(forall<index: Int> result.proof_view_slot(index) == self.proof_view_slot(index + count@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> result.proof_owned_slot(index) == self.proof_owned_slot(index)))]
    fn advance_to_valid(self, count: usize) -> BytesMut {
        let Self { ptr, len, cap, data, unique_at_zero, pending_control, shared_registration, shared_context } = self;
        let old_addr = crate::provenance_specs::pointer_addr(data);
        let pos = crate::capacity_ops::vec_pos_from_data(old_addr) + count;
        let packed = crate::capacity_ops::set_vec_pos_in_data(old_addr, pos);
        BytesMut {
            ptr: ptr.advance_within(count),
            len: len.saturating_sub(count),
            cap: cap - count,
            data: invalid_ptr(packed),
            unique_at_zero, pending_control, shared_registration, shared_context,
        }
    }
}
// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE
// Restricted helper over the actual Shared layout. Automatic Drop and concurrent
// access remain outside this gate; native Release/Acquire calls execute normally.
#[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
pub(crate) mod sequential_shared_control {
    use super::*;
    use creusot_std::{ghost::perm::Perm, logic::Id, prelude::*};
    use crate::ownership_proof::{raw_vec::{self, BoundPtr}, sequential_counter::{SequentialCounter, CounterOwn}, shared_protocol::{self, Coordinator, Packet, Status}};

    #[derive(core::clone::Clone, Copy)]
    pub(crate) struct ControlPtr {
        pub(super) pointer: *mut Shared,
        pub(super) identity: Ghost<Id>,
    }
    pub(crate) struct ControlContext {
        pub(super) registry: Option<Coordinator>,
        pub(super) counter: Option<CounterOwn>,
        pub(super) owner: Option<Box<Perm<*const Shared>>>,
        pub(super) status: Snapshot<Status>,
    }
    impl ControlContext {
        #[logic(open(super), prophetic)]
        pub(super) fn valid(self, control: ControlPtr) -> bool {
            pearlite! {
                match (self.registry, self.counter, self.owner) {
                    (Some(registry), Some(counter), Some(owner)) =>
                        registry.public() == *self.status &&
                        counter@.0 == *control.identity &&
                        counter@.1 == (if self.status.left_pending { 1 } else { 0 }) + (if self.status.right_pending { 1 } else { 0 }) &&
                        *owner.ward() == control.pointer &&
                        owner.val().ref_count@ == counter@.0 &&
                        owner.val().buffer.base.invariant() &&
                        owner.val().buffer.base@ == Some((self.status.allocation, self.status.capacity, 0int)) &&
                        owner.val().buffer.capacity@ == self.status.capacity,
                    (None, None, None) => !self.status.left_pending && !self.status.right_pending,
                    _ => false,
                }
            }
        }
        #[logic(open(super))]
        pub(super) fn active(self) -> bool { pearlite! { self.registry != None } }
    }

    // Before split_to assigns two ranges, all allocation authority remains in
    // this private pending owner. It grants neither handle a byte-access API.
    pub(super) struct PendingControl {
        pub(super) counter: CounterOwn,
        pub(super) owner: Box<Perm<*const Shared>>,
        pub(super) caps: (raw_vec::Recovery, raw_vec::PhysicalRegion),
    }
    impl PendingControl {
        #[logic(open(super), prophetic)]
        pub(super) fn valid(self, pointer: *mut Shared, base: BoundPtr, capacity: usize) -> bool {
            pearlite! {
                base.invariant() && base@ == Some((self.caps.0.namespace(), capacity@, 0int)) &&
                self.caps.0.invariant() && self.caps.1.invariant() &&
                self.caps.0.capacity() == capacity@ && self.caps.1.capacity() == capacity@ &&
                self.caps.1.namespace() == self.caps.0.namespace() &&
                self.caps.1.resource_id() == self.caps.0.namespace() &&
                self.caps.1.lo() == 0 && self.caps.1.hi() == capacity@ &&
                *self.owner.ward() == pointer && self.counter@.1 == 2 &&
                self.owner.val().ref_count@ == self.counter@.0 &&
                self.owner.val().buffer.base == base &&
                self.owner.val().buffer.capacity == capacity
            }
        }
    }
    pub(super) struct HandleRegistration {
        pub(super) control: ControlPtr,
        pub(super) status: Snapshot<Status>,
        pub(super) packet: Packet,
        pub(super) left: bool,
    }
    impl HandleRegistration {
        #[logic(open(super))]
        pub(super) fn view_lo(self) -> Int {
            pearlite! { if self.left { 0int } else { self.status.split } }
        }
        #[logic(open(super))]
        pub(super) fn view_hi(self) -> Int {
            pearlite! { if self.left { self.status.split } else { self.status.capacity } }
        }

        #[logic(open(super), prophetic)]
        pub(super) fn valid(self) -> bool {
            shared_protocol::packet_matches(self.packet, self.status.allocation,
                self.status.capacity, self.status.split, self.status.registration, self.left)
        }
        #[logic(open(super))]
        pub(super) fn same_registration(self, other: Self) -> bool {
            pearlite! {
                self.control == other.control &&
                self.status.allocation == other.status.allocation &&
                self.status.capacity == other.status.capacity &&
                self.status.split == other.status.split &&
                self.status.registration == other.status.registration
            }
        }
        #[logic(open(super), prophetic)]
        pub(super) fn matches(self, context: ControlContext) -> bool {
            pearlite! {
                self.valid() && context.valid(self.control) && context.active() &&
                self.status.allocation == context.status.allocation &&
                self.status.capacity == context.status.capacity &&
                self.status.split == context.status.split &&
                self.status.registration == context.status.registration &&
                (if self.left { context.status.left_pending } else { context.status.right_pending })
            }
        }
    }

    #[requires(pending.inner_logic().valid(pointer, base, capacity))]
    #[requires(split <= capacity)]
    #[ensures(result.0.inner_logic().valid(result.1.inner_logic().control))]
    #[ensures(result.1.inner_logic().control.pointer == pointer)]
    #[ensures(result.0.inner_logic().active())]
    #[ensures(result.0.inner_logic().status.left_pending && result.0.inner_logic().status.right_pending)]
    #[ensures(result.0.inner_logic().status.allocation == pending.inner_logic().caps.0.namespace())]
    #[ensures(result.0.inner_logic().status.capacity == capacity@ && result.0.inner_logic().status.split == split@)]
    #[ensures(result.1.inner_logic().valid() && result.2.inner_logic().valid())]
    #[ensures(result.1.inner_logic().left && !result.2.inner_logic().left)]
    #[ensures(result.1.inner_logic().same_registration(result.2.inner_logic()))]
    #[ensures(result.1.inner_logic().status == result.0.inner_logic().status)]
    #[ensures(result.2.inner_logic().status == result.0.inner_logic().status)]
    #[ensures(result.1.inner_logic().matches(result.0.inner_logic()))]
    #[ensures(result.2.inner_logic().matches(result.0.inner_logic()))]
    #[ensures(forall<index: Int> result.1.inner_logic().packet.1.slot(index) ==
        if 0 <= index && index < split@ { pending.inner_logic().caps.1.slot(index) } else { None })]
    #[ensures(forall<index: Int> result.2.inner_logic().packet.1.slot(index) ==
        if split@ <= index && index < capacity@ { pending.inner_logic().caps.1.slot(index) } else { None })]
    pub(super) fn activate(pointer: *mut Shared, base: BoundPtr, capacity: usize, split: usize,
        pending: Ghost<PendingControl>) -> (Ghost<ControlContext>, Ghost<HandleRegistration>, Ghost<HandleRegistration>) {
        let (caps, counter, owner) = ghost! {
            let pending = pending.into_inner();
            (pending.caps, pending.counter, pending.owner)
        }.split();
        let registrations = shared_protocol::initialize(capacity, split, caps);
        let identity = ghost! {
            let id: Snapshot<Id> = snapshot!(counter@.0);
            id.into_ghost().into_inner()
        };
        let control = ControlPtr { pointer, identity };
        ghost! {
            let (registry, left, right) = registrations.into_inner();
            let counter = counter.into_inner();
            let status: Snapshot<Status> = snapshot!(registry.public());
            (ControlContext { registry: Some(registry), counter: Some(counter), owner: Some(owner.into_inner()), status },
             HandleRegistration { control, status, packet: left, left: true },
             HandleRegistration { control, status, packet: right, left: false })
        }.split()
    }

    #[requires(split@ <= input@.len())]
    #[ensures(result.1.inner_logic().valid(result.0))]
    #[ensures(result.1.inner_logic().active())]
    #[ensures(result.1.inner_logic().status.left_pending && result.1.inner_logic().status.right_pending)]
    #[ensures(result.1.inner_logic().status.split == split@)]
    #[ensures(shared_protocol::packet_matches(result.2.inner_logic(), result.1.inner_logic().status.allocation, result.1.inner_logic().status.capacity, result.1.inner_logic().status.split, result.1.inner_logic().status.registration, true))]
    #[ensures(shared_protocol::packet_matches(result.3.inner_logic(), result.1.inner_logic().status.allocation, result.1.inner_logic().status.capacity, result.1.inner_logic().status.split, result.1.inner_logic().status.registration, false))]
    fn new(input: Vec<u8>, split: usize) -> (ControlPtr, Ghost<ControlContext>, Ghost<Packet>, Ghost<Packet>) {
        let (raw, _len, caps) = raw_vec::detach_vec(input);
        let (base, capacity) = raw.into_bound_ptr_at_zero();
        let registrations = shared_protocol::initialize(capacity, split, caps);
        let (ref_count, counter_own) = SequentialCounter::new(2);
        let shared = Box::new(Shared {
            buffer: SharedBuffer { base, capacity },
            original_capacity_repr: crate::capacity_ops::original_capacity_to_repr(capacity),
            ref_count,
        });
        let (pointer, owner) = Perm::from_box(shared);
        let identity = ghost! {
            let identity: Snapshot<Id> = snapshot!(counter_own@.0);
            identity.into_ghost().into_inner()
        };
        let control = ControlPtr { pointer, identity };
        let state = ghost! {
            let (registry, left, right) = registrations.into_inner();
            let status: Snapshot<Status> = snapshot!(registry.public());
            (ControlContext { registry: Some(registry), counter: Some(counter_own.into_inner()), owner: Some(owner.into_inner()), status }, left, right)
        };
        let (context, left, right) = state.split();
        (control, context, left, right)
    }

    #[requires(context.inner_logic().valid(control) && context.inner_logic().active())]
    #[requires(shared_protocol::packet_matches(packet.inner_logic(), context.inner_logic().status.allocation, context.inner_logic().status.capacity, context.inner_logic().status.split, context.inner_logic().status.registration, left))]
    #[requires(if left { context.inner_logic().status.left_pending } else { context.inner_logic().status.right_pending })]
    #[ensures((^context.inner_logic()).valid(control))]
    #[ensures(result == !(^context.inner_logic()).active())]
    #[ensures(result == (!(^context.inner_logic()).status.left_pending && !(^context.inner_logic()).status.right_pending))]
    #[ensures((^context.inner_logic()).status.allocation == context.inner_logic().status.allocation)]
    #[ensures((^context.inner_logic()).status.capacity == context.inner_logic().status.capacity)]
    #[ensures((^context.inner_logic()).status.split == context.inner_logic().status.split)]
    #[ensures((^context.inner_logic()).status.registration == context.inner_logic().status.registration)]
    #[ensures((^context.inner_logic()).status.left_pending == (if left { false } else { context.inner_logic().status.left_pending }))]
    #[ensures((^context.inner_logic()).status.right_pending == (if left { context.inner_logic().status.right_pending } else { false }))]
    pub(super) fn release(control: ControlPtr, mut context: Ghost<&mut ControlContext>, packet: Ghost<Packet>, left: bool) -> bool {
        let old = {
            let (permission, counter) = ghost! {
                let context = &mut **context;
                (&**context.owner.as_ref().unwrap(), context.counter.as_mut().unwrap())
            }.split();
            let shared = unsafe { Perm::as_ref(control.pointer, permission) };
            shared.ref_count.fetch_sub_release(1, counter)
        };
        shared_protocol::retire(ghost! { context.registry.as_mut().unwrap() }, packet, left);
        ghost! {
            context.status = snapshot!(context.registry.unwrap_logic().public());
        };
        if old != 1 { return false; }

        {
            let permission = ghost! { &**context.owner.as_ref().unwrap() };
            let shared = unsafe { Perm::as_ref(control.pointer, permission) };
            let observed = shared.ref_count.load_acquire(ghost! { context.counter.as_ref().unwrap() });
            assert!(observed == 0);
        }
        let full = shared_protocol::finish(ghost! { context.registry.take().unwrap() });
        let (base, capacity) = {
            let permission = ghost! { &mut **context.owner.as_mut().unwrap() };
            let shared = unsafe { Perm::as_mut(control.pointer, permission) };
            let base = shared.buffer.base;
            let capacity = shared.buffer.capacity;
            // Disarm native SharedBuffer::drop before freeing A explicitly.
            shared.buffer.base = BoundPtr::unbound(NonNull::dangling());
            shared.buffer.capacity = 0;
            (base, capacity)
        };
        unsafe { raw_vec::deallocate_bound_vec(base, capacity, full); }
        let owner = ghost! {
            let _counter = context.counter.take().unwrap();
            context.owner.take().unwrap()
        };
        // Existing standard typed ownership operation; no deallocation-event
        // or automatic-Drop proof is inferred from its contract.
        unsafe { Perm::drop(control.pointer, owner); }
        true
    }

    #[requires(split@ <= input@.len())]
    pub(crate) fn release_left_then_right(input: Vec<u8>, split: usize) {
        let (control, mut context, left, right) = new(input, split);
        let first = release(control, context.borrow_mut(), left, true);
        assert!(!first);
        let last = release(control, context.borrow_mut(), right, false);
        assert!(last);
    }

    #[requires(split@ <= input@.len())]
    pub(crate) fn release_right_then_left(input: Vec<u8>, split: usize) {
        let (control, mut context, left, right) = new(input, split);
        let first = release(control, context.borrow_mut(), right, false);
        assert!(!first);
        let last = release(control, context.borrow_mut(), left, true);
        assert!(last);
    }

    #[cfg(feature = "negative_missing_ticket")]
    pub(crate) fn reject_missing_empty_ticket(input: Vec<u8>) {
        let (control, mut context, left, right) = new(input, 0);
        let _ = left;
        let last = release(control, context.borrow_mut(), right, false);
        assert!(!last);
        proof_assert!(context.status.split == 0);
        proof_assert!(context.status.left_pending && !context.status.right_pending);
        // Every byte has retired, but the empty left handle still owns a ticket.
        let _forbidden = shared_protocol::finish(ghost! { context.registry.take().unwrap() });
    }

}
// END EXACT SEQUENTIAL SHARED CONTROL GATE