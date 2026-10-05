use alloc::{vec::Vec, boxed::Box};
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr::{self, NonNull};
use core::cmp;
use core::sync::atomic::{AtomicUsize, Ordering};
use creusot_std::prelude::*;
use crate::capacity_ops::original_capacity_to_repr;
// BEGIN EXACT RELEASE_UNIQUE_STORAGE
// SAFETY: ptr is a handle into the original global byte allocation, offset
// is its exact displacement, and full allocation ownership is transferred.
#[cfg_attr(creusot, requires(ptr.invariant()))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().0.invariant() && capabilities.inner_logic().1.invariant()))]
#[cfg_attr(creusot, requires(ptr@ == Some((capabilities.inner_logic().0.namespace(), capacity@ + offset@, offset@))))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().0.capacity() == capacity@ + offset@))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().1.capacity() == capacity@ + offset@))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().1.namespace() == capabilities.inner_logic().0.namespace()))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().1.resource_id() == capabilities.inner_logic().0.namespace()))]
#[cfg_attr(creusot, requires(capabilities.inner_logic().1.lo() == 0 && capabilities.inner_logic().1.hi() == capacity@ + offset@))]
unsafe fn release_unique_storage(
    #[cfg(not(any(creusot, bytes_proof_probe)))] ptr: NonNull<u8>,
    #[cfg(any(creusot, bytes_proof_probe))] ptr: crate::ownership_proof::raw_vec::BoundPtr,
    capacity: usize, offset: usize,
    #[cfg(any(creusot, bytes_proof_probe))] capabilities: Ghost<(crate::ownership_proof::raw_vec::Recovery, crate::ownership_proof::raw_vec::PhysicalRegion)>,
) {
    let original_capacity = capacity + offset;
    #[cfg(not(any(creusot, bytes_proof_probe)))]
    unsafe { crate::allocation_ops::deallocate_u8(ptr.as_ptr().sub(offset), original_capacity); }
    #[cfg(any(creusot, bytes_proof_probe))]
    {
        let base = ptr.retreat_within(offset);
        unsafe { crate::ownership_proof::raw_vec::deallocate_bound_vec(base, original_capacity, capabilities); }
    }
}
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
// BEGIN EXACT BYTESMUT INVARIANT
#[cfg(all(creusot, bytes_proof_valid_handle))]
impl creusot_std::invariant::Invariant for BytesMut {
    #[logic(open(self), prophetic)]
    fn invariant(self) -> bool {
        pearlite! { self.proof_empty_valid() || (self.proof_unique_owned() && self.proof_initialized()) }
    }
}
impl AsRef<[u8]> for BytesMut {
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(result@.len() == self.len@))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[inline]
    #[cfg_attr(creusot, check(ghost))]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl AsMut<[u8]> for BytesMut {
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(result@.len() == self.len@ && (^result)@.len() == self.len@))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).proof_owned_valid()))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        (^self).proof_view_slot(index) == Some(Some((^result)@[index]))))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int> !(0 <= index && index < self.len@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_valid_handle), ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}
#[inline]
#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    // This null-derived pointer stores integer metadata only. It carries no
    // allocation permission and must not be used as a dereferenceable pointer.
    let ptr = crate::provenance_specs::metadata_pointer(addr);
    debug_assert_eq!(crate::provenance_specs::pointer_addr(ptr), addr);
    ptr.cast::<T>()
}
const KIND_VEC: usize = 0b1;
const KIND_ARC: usize = 0b0;
const KIND_MASK: usize = 0b1;
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

impl BytesMut {
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle, feature = "negative_unregistered_as_ref"))]
    #[cfg_attr(creusot, requires(creusot_std::invariant::inv(owner.ptr)))]
    #[cfg_attr(creusot, requires(owner.len > 0usize && owner.len <= owner.cap))]
    #[cfg_attr(creusot, requires(owner.unique_at_zero.inner_logic() == None && owner.pending_control.inner_logic() == None && owner.shared_registration.inner_logic() == None && owner.shared_context.inner_logic() == None))]
    pub(crate) fn proof_reject_unregistered_as_ref(#[cfg_attr(creusot, creusot::open_inv)] owner: &Self) {
        // Structurally valid pointer metadata alone is not a valid positive-
        // length handle. The real safe trait call must establish its invariant.
        let _ = AsRef::<[u8]>::as_ref(owner);
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_owned(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_VEC &&
            self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize &&
            self.pending_control.inner_logic() == None &&
            self.shared_registration.inner_logic() == None &&
            self.shared_context.inner_logic() == None &&
            match self.unique_at_zero.inner_logic() {
                None => false,
                Some((recovery, region)) =>
                    self.ptr.invariant() &&
                    self.ptr@ == Some((recovery.namespace(), self.cap@, 0int)) &&
                    recovery.invariant() && region.invariant() &&
                    recovery.capacity() == self.cap@ &&
                    region.capacity() == self.cap@ &&
                    region.namespace() == recovery.namespace() &&
                    region.resource_id() == recovery.namespace() &&
                    region.lo() == 0 && region.hi() == self.cap@ &&
                    self.len@ <= self.cap@
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {
        pearlite! {
            self.proof_unique_at_zero_owned() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_unique_slot(index))
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
    #[inline]
    #[cfg_attr(creusot, ensures(result.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, ensures(result.len@ == vec@.len()))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures(result.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, ensures(
        ((result.data.addr_logic() & crate::capacity_ops::ORIGINAL_CAPACITY_MASK)
            >> crate::capacity_ops::ORIGINAL_CAPACITY_OFFSET)@
            == crate::capacity_ops::original_capacity_class_logic(result.cap)
    ))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < vec@.len() ==>
        result.proof_unique_slot(index) == Some(Some(vec@[index]))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> vec@.len() <= index && index < result.cap@ ==>
        result.proof_unique_slot(index) == Some(None)))]
    pub(crate) fn from_vec(vec: Vec<u8>) -> BytesMut {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        let (ptr, len, cap) = {
            let mut vec = ManuallyDrop::new(vec);
            (vptr(vec.as_mut_ptr()), vec.len(), vec.capacity())
        };
        #[cfg(any(creusot, bytes_proof_probe))]
        let (ptr, len, cap, capabilities) =
            crate::ownership_proof::bound_ptr::detach_bound_vec(vec);

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = crate::capacity_ops::pack_vec_metadata(original_capacity_repr);

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
            #[cfg(any(creusot, bytes_proof_probe))]
            unique_at_zero: ghost! { Some(capabilities.into_inner()) },
            #[cfg(any(creusot, bytes_proof_probe))]
            pending_control: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_registration: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_context: ghost! { None },
        }
    }
    // Explicit restricted proof path. Forgetting the handle before B3 prevents
    // its ordinary destructor from freeing the same native allocation again.
    // This does not establish scope-exit Drop or a Shared cleanup protocol.
    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_owned()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    pub(crate) fn proof_release_unique_at_zero(mut self) {
        let state = mem::replace(&mut self.unique_at_zero, ghost! { None });
        let capabilities = ghost! { state.into_inner().unwrap() };
        let ptr = self.ptr;
        let capacity = self.cap;
        #[cfg(bytes_proof_valid_handle)]
        {
            // Authority has been extracted from the handle. Normalize individual fields;
            // assigning a whole handle here would run the old armed destructor.
            proof_assert!(self.unique_at_zero.inner_logic() == None && self.pending_control.inner_logic() == None && self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None);
            self.ptr = crate::ownership_proof::raw_vec::BoundPtr::unbound(NonNull::dangling());
            self.len = 0;
            self.cap = 0;
            self.data = invalid_ptr(KIND_VEC);
        }
        mem::forget(self);
        // SAFETY: the predicate provides exact offset-zero full authority.
        unsafe { release_unique_storage(ptr, capacity, 0, capabilities); }
    }
    // BEGIN EXACT AS_SLICE
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[cfg_attr(creusot, check(ghost))]
    fn as_slice(&self) -> &[u8] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) } }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.len == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_bound(
                    &self.ptr, self.len, ghost! { &self.unique_at_zero.as_ref().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_ref().unwrap();
                (&registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet(&self.ptr, self.len, packet, status, left) }
        }
    }
    // BEGIN EXACT AS_SLICE_MUT
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.len@ && (^result)@.len() == self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
    #[cfg_attr(creusot, ensures((^self).proof_owned_valid()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(creusot, ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        (^self).proof_view_slot(index) == Some(Some((^result)@[index]))))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(0 <= index && index < self.len@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) } }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.len == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound_mut(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_bound_mut(
                    &self.ptr, self.len, ghost! { &mut self.unique_at_zero.as_mut().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_mut().unwrap();
                (&mut registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet_mut(&self.ptr, self.len, packet, status, left) }
        }
    }
    // BEGIN EXACT SPARE_CAPACITY_MUT
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.cap@ - self.len@ && (^result)@.len() == self.cap@ - self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < result@.len() ==>
        self.proof_view_slot(self.len@ + index) == Some(result@[index]@)))]
    // Absolute visible indices let publication and frame clients use the same
    // slot term without reconstructing the spare slice's relative index.
    #[cfg_attr(creusot, ensures(forall<index: Int> self.len@ <= index && index < self.cap@ ==>
        self.proof_view_slot(index) == Some(result@[index - self.len@]@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.len@ <= index && index < self.cap@ ==>
        (^self).proof_view_slot(index) == Some((^result)@[index - self.len@]@)))]
    #[cfg_attr(creusot, ensures((^self).proof_owned_valid()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^self).len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(((^self).unique_at_zero.inner_logic() == None) == (self.unique_at_zero.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures(self.unique_at_zero.inner_logic() != None ==> (^self).unique_at_zero.inner_logic().unwrap_logic().0 == self.unique_at_zero.inner_logic().unwrap_logic().0))]
    #[cfg_attr(creusot, ensures(((^self).shared_registration.inner_logic() == None) == (self.shared_registration.inner_logic() == None)))]
    #[cfg_attr(creusot, ensures((^self).pending_control == self.pending_control && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().control == self.shared_registration.inner_logic().unwrap_logic().control))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().status == self.shared_registration.inner_logic().unwrap_logic().status))]
    #[cfg_attr(creusot, ensures(self.shared_registration.inner_logic() != None ==> (^self).shared_registration.inner_logic().unwrap_logic().left == self.shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < (^result)@.len() ==>
        (^self).proof_view_slot(self.len@ + index) == Some((^result)@[index]@)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> !(self.len@ <= index && index < self.cap@) ==>
        (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        !(self.ptr@.unwrap_logic().2 + self.len@ <= index && index < self.ptr@.unwrap_logic().2 + self.cap@) ==>
            (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_registration.inner_logic() != None ==>
        (^self).shared_registration.inner_logic().unwrap_logic().packet.0 == self.shared_registration.inner_logic().unwrap_logic().packet.0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.shared_registration.inner_logic().unwrap_logic().packet.1.lo() &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.shared_registration.inner_logic().unwrap_logic().packet.1.hi()))]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            let len = self.cap - self.len;
            slice::from_raw_parts_mut(ptr.cast(), len)
        }
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            #[cfg(bytes_proof_valid_handle)]
            if self.cap == 0 {
                return unsafe { crate::ownership_proof::raw_vec::borrow_empty_bound_uninit_mut(&self.ptr) };
            }
            if self.kind() == KIND_VEC {
                return unsafe { crate::ownership_proof::raw_vec::borrow_spare_preserving_prefix(
                    self.ptr, self.len, self.cap,
                    ghost! { &mut self.unique_at_zero.as_mut().unwrap().1 }) };
            }
            let (packet, status, left) = ghost! {
                let registration = self.shared_registration.as_mut().unwrap();
                (&mut registration.packet, registration.status, registration.left)
            }.split();
            unsafe { crate::ownership_proof::shared_protocol::borrow_packet_uninit_mut(self.ptr, self.len, self.cap, packet, status, left) }
        }
    }
    // BEGIN EXACT TRUNCATE
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len@ == if len <= self.len { len@ } else { self.len@ }))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    pub fn truncate(&mut self, len: usize) {
        if len <= self.len() {
            // SAFETY: Shrinking the buffer cannot expose uninitialized bytes.
            unsafe { self.set_len(len) };
        }
    }
    // BEGIN EXACT CLEAR
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures((^self).len == 0usize))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    pub fn clear(&mut self) {
        // SAFETY: Setting the length to zero cannot expose uninitialized bytes.
        unsafe { self.set_len(0) };
    }
    // BEGIN EXACT SET_LEN
    #[cfg_attr(creusot, requires(self.proof_owned_valid()))]
    #[cfg_attr(creusot, requires(len <= self.cap))]
    #[cfg_attr(creusot, requires(forall<index: Int> 0 <= index && index < len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).len == len))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized()))]
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap, "set_len out of bounds");
        self.len = len;
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.len))]
    #[cfg_attr(creusot, check(ghost))]
    pub fn len(&self) -> usize {
        self.len
    }
    // BEGIN EXACT IS_EMPTY
    #[cfg_attr(creusot, ensures(result == (self.len == 0usize)))]
    #[cfg_attr(creusot, check(ghost))]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.cap))]
    pub fn capacity(&self) -> usize {
        self.cap
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]
    #[cfg_attr(creusot, check(ghost))]
    fn kind(&self) -> usize {
        crate::provenance_specs::pointer_addr(self.data) & KIND_MASK
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    fn proof_generic_as_ref<T: AsRef<[u8]>>(owner: &T) -> Option<u8> {
        let bytes = owner.as_ref();
        if bytes.len() > 0 { Some(bytes[0]) } else { None }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    fn proof_generic_as_mut<T: AsMut<[u8]>>(owner: &mut T, value: u8) {
        let bytes = owner.as_mut();
        if bytes.len() > 0 { bytes[0] = value; }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_valid_handle))]
    pub(crate) fn proof_traits_unique(input: Vec<u8>, value: u8) {
        let mut owner = Self::from_vec(input);
        let length = owner.len();
        assert!(AsRef::<[u8]>::as_ref(&owner).len() == length);
        let _ = Self::proof_generic_as_ref(&owner);
        if length > 0 {
            AsMut::<[u8]>::as_mut(&mut owner)[0] = value;
            assert!(AsRef::<[u8]>::as_ref(&owner)[0] == value);
        }
        owner.proof_release_unique_at_zero();
    }
}
#[cfg(not(creusot))]
impl Drop for SharedBuffer {
    fn drop(&mut self) {
        // u8 has no destructor; deallocation does not require initialized bytes.
        unsafe { crate::allocation_ops::deallocate_u8(self.base.as_ptr(), self.capacity); }
    }
}
#[cfg(creusot)]
fn proof_observe_is_empty(owner: &BytesMut) {
    assert!(owner.is_empty() == (owner.len() == 0));
}