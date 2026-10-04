// Exact source fragments, recorded below.
use alloc::{vec::Vec, boxed::Box};
use core::mem::{self, ManuallyDrop};
use core::ptr::{self,NonNull};
use core::sync::atomic::{AtomicUsize,Ordering};
use creusot_std::prelude::*;
use crate::capacity_ops::original_capacity_to_repr;
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
const KIND_VEC: usize = 0b1;
const KIND_ARC: usize = 0b0;
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
// BEGIN EXACT SEQUENTIAL SHARED CONTROL GATE
// Restricted helper over the actual Shared layout. Automatic Drop and concurrent
// access remain outside this gate; native Release/Acquire calls execute normally.
#[cfg(any(creusot, bytes_proof_probe))]
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
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {
        pearlite! {
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
                    self.len@ <= self.cap@ &&
                    forall<index: Int> 0 <= index && index < self.len@ ==>
                        exists<value: u8> region.slot(index) == Some(Some(value))
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
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.len))]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]
    fn kind(&self) -> usize {
        crate::provenance_specs::pointer_addr(self.data) & KIND_MASK
    }
    // BEGIN EXACT SPLIT_TO
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, requires(at <= self.len))]
    #[cfg_attr(creusot, ensures((^self).proof_split_pair_valid(result)))]
    #[cfg_attr(creusot, ensures(result.len == at && (^self).len@ == self.len@ - at@))]
    #[cfg_attr(creusot, ensures(result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[must_use = "consider BytesMut::advance if you don't need the other half"]
    pub fn split_to(&mut self, at: usize) -> BytesMut {
        assert!(
            at <= self.len(),
            "split_to out of bounds: {:?} <= {:?}",
            at,
            self.len(),
        );

        unsafe {
            let mut other = self.shallow_clone();
            #[cfg(any(creusot, bytes_proof_probe))]
            self.proof_register_split(&mut other, at);
            // SAFETY: We've checked that `at` <= `self.len()` and we know that `self.len()` <=
            // `self.capacity()`.
            self.advance_unchecked(at);
            other.cap = at;
            other.len = at;
            other
        }
    }
    // BEGIN EXACT ADVANCE_UNCHECKED
    #[cfg_attr(creusot, requires(self.ptr.invariant() && self.ptr@ != None))]
    #[cfg_attr(creusot, requires(self.ptr@.unwrap_logic().2 + self.cap@ <= self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, requires(count <= self.len && self.len <= self.cap))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_ARC))]
    #[cfg_attr(creusot, ensures((^self).ptr.invariant()))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(creusot, ensures((^self).len@ == self.len@ - count@ && (^self).cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures((^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {
        // Setting the start to 0 is a no-op, so return early if this is the
        // case.
        if count == 0 {
            return;
        }

        debug_assert!(count <= self.cap, "internal: set_start out of bounds");

        let kind = self.kind();

        #[cfg(any(creusot, bytes_proof_probe))]
        if kind == KIND_VEC { panic!("unique advance is outside the sequential split gate"); }
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        if kind == KIND_VEC {
            // Setting the start when in vec representation is a little more
            // complicated. First, we have to track how far ahead the
            // "start" of the byte buffer from the beginning of the vec. We
            // also have to ensure that we don't exceed the maximum shift.
            let pos = self.get_vec_pos() + count;

            if pos <= MAX_VEC_POS {
                self.set_vec_pos(pos);
            } else {
                // The repr must be upgraded to ARC. This will never happen
                // on 64 bit systems and will only happen on 32 bit systems
                // when shifting past 134,217,727 bytes. As such, we don't
                // worry too much about performance here.
                self.promote_to_shared(/*ref_count = */ 1);
            }
        }

        // Updating the start of the view is setting `ptr` to point to the
        // new start and updating the `len` field to reflect the new length
        // of the view.
        #[cfg(not(any(creusot, bytes_proof_probe)))]
        { self.ptr = vptr(self.ptr.as_ptr().add(count)); }
        #[cfg(any(creusot, bytes_proof_probe))]
        { self.ptr = self.ptr.advance_within(count); }
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }
    // BEGIN EXACT PROMOTE_TO_SHARED
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, requires(ref_cnt == 2usize))]
    #[cfg_attr(creusot, ensures((^self).proof_pending_valid()))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    unsafe fn promote_to_shared(&mut self, ref_cnt: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(ref_cnt == 1 || ref_cnt == 2);

        let original_capacity_repr =
            crate::capacity_ops::original_capacity_repr_from_data(
                crate::provenance_specs::pointer_addr(self.data),
            );

        // The vec offset cannot be concurrently mutated, so there
        // should be no danger reading it.
        let off = crate::capacity_ops::vec_pos_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        );

        // First, allocate a new `Shared` instance containing the
        // `Vec` fields. It's important to note that `ptr`, `len`,
        // and `cap` cannot be mutated without having `&mut self`.
        // This means that these fields will not be concurrently
        // updated and since the buffer hasn't been promoted to an
        // `Arc`, those three fields still are the components of the
        // vector.
        #[cfg(any(creusot, bytes_proof_probe))]
        let (ref_count, counter_own) = crate::ownership_proof::sequential_counter::SequentialCounter::new(ref_cnt);
        let shared = Box::new(Shared {
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            buffer: SharedBuffer::from_handle(self.ptr.as_ptr(), self.cap, off),
            #[cfg(any(creusot, bytes_proof_probe))]
            buffer: SharedBuffer { base: self.ptr, capacity: self.cap },
            original_capacity_repr,
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            ref_count: AtomicUsize::new(ref_cnt),
            #[cfg(any(creusot, bytes_proof_probe))]
            ref_count,
        });

        #[cfg(not(any(creusot, bytes_proof_probe)))]
        let shared = Box::into_raw(shared);
        #[cfg(any(creusot, bytes_proof_probe))]
        let (shared, owner) = crate::ownership_proof::boxed_alignment::into_raw_aligned(shared);

        #[cfg(any(creusot, bytes_proof_probe))]
        crate::ownership_proof::boxed_alignment::aligned_address_has_clear_low_bit(
            crate::provenance_specs::pointer_addr(shared), core::mem::align_of::<Shared>(),
        );

        // The pointer should be aligned, so this assert should
        // always succeed.
        debug_assert_eq!(
            crate::provenance_specs::pointer_addr(shared) & KIND_MASK,
            KIND_ARC
        );

        self.data = shared;
        #[cfg(any(creusot, bytes_proof_probe))]
        {
            self.pending_control = ghost! {
                let caps = self.unique_at_zero.take().unwrap();
                Some(sequential_shared_control::PendingControl { counter: counter_own.into_inner(), owner: owner.into_inner(), caps })
            };
        }
    }
    // BEGIN EXACT SHALLOW_CLONE
    #[cfg_attr(creusot, requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(creusot, ensures((^self).proof_pending_valid()))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(creusot, ensures(result.ptr == (^self).ptr && result.len == (^self).len && result.cap == (^self).cap && result.data == (^self).data))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[inline]
    unsafe fn shallow_clone(&mut self) -> BytesMut {
        if self.kind() == KIND_ARC {
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            increment_shared(self.data);
            #[cfg(any(creusot, bytes_proof_probe))]
            panic!("ARC clone is outside the first sequential split gate");
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            { ptr::read(self) }
            #[cfg(any(creusot, bytes_proof_probe))]
            { self.pending_descriptor_copy() }
        } else {
            self.promote_to_shared(/*ref_count = */ 2);
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            { ptr::read(self) }
            #[cfg(any(creusot, bytes_proof_probe))]
            { self.pending_descriptor_copy() }
        }
    }
    // Copies metadata only. The completed split protocol must distribute
    // regions before this pending descriptor can become a verified handle.
    #[cfg_attr(creusot, ensures(result.ptr == self.ptr && result.len == self.len && result.cap == self.cap && result.data == self.data))]
    #[cfg_attr(creusot, ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg(any(creusot, bytes_proof_probe))]
    fn pending_descriptor_copy(&self) -> BytesMut {
        BytesMut {
            ptr: self.ptr, len: self.len, cap: self.cap, data: self.data,
            unique_at_zero: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            pending_control: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_registration: ghost! { None },
            #[cfg(any(creusot, bytes_proof_probe))]
            shared_context: ghost! { None },
        }
    }
    // BEGIN EXACT BYTESMUT SEQUENTIAL SPLIT METHODS
    #[cfg(creusot)]
    #[logic(prophetic)]
    fn proof_pending_valid(self) -> bool {
        pearlite! {
            self.unique_at_zero.inner_logic() == None &&
            self.shared_registration.inner_logic() == None && self.shared_context.inner_logic() == None &&
            self.len <= self.cap && self.data.addr_logic() & KIND_MASK == KIND_ARC &&
            match self.pending_control.inner_logic() {
                Some(pending) => pending.valid(self.data, self.ptr, self.cap), None => false,
            }
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
                    self.ptr@ == Some((registration.status.allocation, registration.status.capacity,
                        if registration.left { 0int } else { registration.status.split })) &&
                    self.cap@ == (if registration.left { registration.status.split }
                        else { registration.status.capacity - registration.status.split }),
            }
        }
    }
    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_split_pair_valid(self, left: Self) -> bool {
        pearlite! {
            self.proof_registered_valid() && left.proof_registered_valid() &&
            !self.shared_registration.inner_logic().unwrap_logic().left &&
            left.shared_registration.inner_logic().unwrap_logic().left &&
            left.shared_context.inner_logic() == None &&
            match self.shared_context.inner_logic() {
                None => false,
                Some(context) =>
                    self.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    left.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    context.status.left_pending && context.status.right_pending,
            }
        }
    }
    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_pending_valid()))]
    #[cfg_attr(creusot, requires(at <= self.len))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^other).ptr == other.ptr && (^other).len == other.len && (^other).cap == other.cap && (^other).data == other.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero.inner_logic() == None && (^self).pending_control.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^other).unique_at_zero.inner_logic() == None && (^other).pending_control.inner_logic() == None && (^other).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic() != None && (^other).shared_registration.inner_logic() != None && (^self).shared_context.inner_logic() != None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().valid() && (^other).shared_registration.inner_logic().unwrap_logic().valid()))]
    #[cfg_attr(creusot, ensures(!(^self).shared_registration.inner_logic().unwrap_logic().left && (^other).shared_registration.inner_logic().unwrap_logic().left))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic()) && (^other).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic().unwrap_logic().status.left_pending && (^self).shared_context.inner_logic().unwrap_logic().status.right_pending))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data && (^other).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().status.capacity == self.cap@ && (^self).shared_registration.inner_logic().unwrap_logic().status.split == at@))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().same_registration((^other).shared_registration.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures(self.ptr@ == Some(((^self).shared_registration.inner_logic().unwrap_logic().status.allocation, self.cap@, 0int))))]
    fn proof_register_split(&mut self, other: &mut Self, at: usize) {
        let pending = ghost! { self.pending_control.take().unwrap() };
        let (context, left, right) = sequential_shared_control::activate(self.data, self.ptr, self.cap, at, pending);
        self.shared_context = ghost! { Some(context.into_inner()) };
        self.shared_registration = ghost! { Some(right.into_inner()) };
        other.shared_registration = ghost! { Some(left.into_inner()) };
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.shared_context.inner_logic() != None))]
    #[cfg_attr(creusot, ensures(result.inner_logic() == self.shared_context.inner_logic().unwrap_logic()))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration))]
    fn proof_take_coordinator(&mut self) -> Ghost<sequential_shared_control::ControlContext> {
        ghost! { self.shared_context.take().unwrap() }
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(self.proof_registered_valid()))]
    #[cfg_attr(creusot, requires(self.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().left == left))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).valid(self.shared_registration.inner_logic().unwrap_logic().control)))]
    #[cfg_attr(creusot, ensures(result == !(^context.inner_logic()).active()))]
    #[cfg_attr(creusot, ensures(result == (!(^context.inner_logic()).status.left_pending && !(^context.inner_logic()).status.right_pending)))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.allocation == context.inner_logic().status.allocation && (^context.inner_logic()).status.capacity == context.inner_logic().status.capacity && (^context.inner_logic()).status.split == context.inner_logic().status.split && (^context.inner_logic()).status.registration == context.inner_logic().status.registration))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.left_pending == (if left { false } else { context.inner_logic().status.left_pending })))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).status.right_pending == (if left { context.inner_logic().status.right_pending } else { false })))]
    fn proof_release(mut self, context: Ghost<&mut sequential_shared_control::ControlContext>, left: bool) -> bool {
        let registration = ghost! { self.shared_registration.take().unwrap() };
        let identity = ghost! { registration.control.identity.into_inner() };
        let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
        let packet = ghost! { registration.into_inner().packet };
        let last = sequential_shared_control::release(control, context, packet, left);
        mem::forget(self);
        last
    }

    #[cfg(any(creusot, bytes_proof_probe))]
    #[cfg_attr(creusot, requires(at@ <= input@.len()))]
    pub(crate) fn proof_split_then_release(input: Vec<u8>, at: usize, right_first: bool) {
        let mut right = Self::from_vec(input);
        let left = right.split_to(at);
        let mut context = right.proof_take_coordinator();
        if right_first {
            let first = right.proof_release(context.borrow_mut(), false);
            assert!(!first);
            let last = left.proof_release(context.borrow_mut(), true);
            assert!(last);
        } else {
            let first = left.proof_release(context.borrow_mut(), true);
            assert!(!first);
            let last = right.proof_release(context.borrow_mut(), false);
            assert!(last);
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), feature = "negative_missing_split_ticket"))]
    pub(crate) fn proof_reject_missing_empty_ticket(input: Vec<u8>) {
        let mut right = Self::from_vec(input);
        let left = right.split_to(0);
        let mut context = right.proof_take_coordinator();
        mem::forget(left);
        let last = right.proof_release(context.borrow_mut(), false);
        assert!(!last);
        proof_assert!(context.status.left_pending && !context.status.right_pending);
        // Full byte coverage cannot replace the missing empty-handle ticket.
        let _forbidden = crate::ownership_proof::shared_protocol::finish(
            ghost! { context.registry.take().unwrap() },
        );
    }

}
#[cfg(not(creusot))]
impl Drop for SharedBuffer {
    fn drop(&mut self) {
        // u8 has no destructor; deallocation does not require initialized bytes.
        unsafe { drop(Vec::from_raw_parts(self.base.as_ptr(), 0, self.capacity)); }
    }
}