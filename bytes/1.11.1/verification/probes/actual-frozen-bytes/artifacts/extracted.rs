use creusot_std::prelude::*;
use core::sync::atomic::AtomicPtr;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::mem::{self,ManuallyDrop};
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use crate::capacity_ops::original_capacity_to_repr;
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
    #[logic]
    pub(crate) fn proof_unique_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.unique_at_zero.inner_logic() {
                None => None,
                Some((_, region)) => region.slot(index),
            }
        }
    }
    // BEGIN EXACT FROZEN BYTESMUT FREEZE
    #[inline]
    #[cfg_attr(all(creusot, bytes_proof_frozen), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), requires(*coordinator == None))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures((^coordinator) != None && (^coordinator).unwrap_logic().valid()))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.frozen != None && result.frozen.unwrap_logic().valid(result.ptr,result.len)))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.frozen.unwrap_logic().shared == (^coordinator).unwrap_logic().shared))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.frozen.unwrap_logic().ticket.frac() == creusot_std::logic::real::PositiveReal::from_int(1)))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.len == self.len))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(forall<i:Int> 0 <= i && i < self.len@ ==> result.frozen.unwrap_logic().shared.val().cur().slot(i) == self.proof_unique_slot(i)))]
    pub fn freeze(self,
        #[cfg(bytes_proof_frozen)] coordinator: &mut Option<crate::ownership_proof::frozen_region::FrozenOwner>,
        #[cfg(bytes_proof_frozen)] table: &'static crate::bytes::Vtable,
    ) -> Bytes {
        // Extraction-only explicit coordinator: transfers the actual receiver's
        // affine physical ownership and leaves native callback dispatch aside.
        #[cfg(bytes_proof_frozen)]
        {
            let mut bytes=self;
            let ptr=bytes.ptr;
            let len=bytes.len;
            let cap=bytes.cap;
            let capabilities=ghost! { bytes.unique_at_zero.take().unwrap() };
            mem::forget(bytes);
            let (owner,reader)=crate::ownership_proof::frozen_region::FrozenOwner::new(ptr,cap,len,capabilities);
            *coordinator=Some(owner);
            return unsafe { Bytes::with_vtable(ptr,len,AtomicPtr::new(core::ptr::null_mut()),table,reader) };
        }
        #[cfg(not(bytes_proof_frozen))]
        {
        let bytes = ManuallyDrop::new(self);
        if bytes.kind() == KIND_VEC {
            // Just re-use `Bytes` internal Vec vtable
            unsafe {
                let off = bytes.get_vec_pos();
                let vec = rebuild_vec(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off);
                let mut b: Bytes = vec.into();
                b.advance(off);
                b
            }
        } else {
            debug_assert_eq!(bytes.kind(), KIND_ARC);

            let ptr = bytes.ptr.as_ptr();
            let len = bytes.len;
            let data = AtomicPtr::new(bytes.data.cast());
            unsafe { Bytes::with_vtable(ptr, len, data, &SHARED_VTABLE) }
        }
        }
    }
}
pub struct Bytes {
    #[cfg(not(bytes_proof_frozen))]
    ptr: *const u8,
    #[cfg(bytes_proof_frozen)]
    pub(crate) ptr: crate::ownership_proof::raw_vec::BoundPtr,
    #[cfg(bytes_proof_frozen)]
    pub(crate) frozen: Option<crate::ownership_proof::frozen_region::FrozenReader>,
    #[cfg(not(bytes_proof_frozen))]
    len: usize,
    #[cfg(bytes_proof_frozen)]
    pub(crate) len: usize,
    // inlined "trait object"
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
}
pub(crate) struct Vtable {
    /// fn(data, ptr, len, current_vtable)
    ///
    /// Pass the current table so callbacks that preserve the storage kind do
    /// not have to refer back to the table that contains the callback.
    pub clone: unsafe fn(&AtomicPtr<()>, *const u8, usize, &'static Vtable) -> Bytes,
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
impl Bytes {
    // BEGIN EXACT FROZEN BYTES CONSTRUCTOR
    #[inline]
    #[cfg_attr(all(creusot, bytes_proof_frozen), requires(frozen.valid(ptr, len)))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.ptr == ptr && result.len == len))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result.frozen == Some(frozen)))]
    pub(crate) unsafe fn with_vtable(
        #[cfg(not(bytes_proof_frozen))] ptr: *const u8,
        #[cfg(bytes_proof_frozen)] ptr: crate::ownership_proof::raw_vec::BoundPtr,
        len: usize,
        data: AtomicPtr<()>,
        vtable: &'static Vtable,
        #[cfg(bytes_proof_frozen)] frozen: crate::ownership_proof::frozen_region::FrozenReader,
    ) -> Bytes {
        Bytes {
            ptr,
            len,
            data,
            vtable,
            #[cfg(bytes_proof_frozen)] frozen: Some(frozen),
        }
    }
    // BEGIN EXACT FROZEN BYTES READ
    #[inline]
    #[cfg_attr(all(creusot, bytes_proof_frozen), requires(self.frozen != None))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), requires(self.frozen.unwrap_logic().valid(self.ptr, self.len)))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(result@.len() == self.len@))]
    #[cfg_attr(all(creusot, bytes_proof_frozen), ensures(forall<i: Int> 0 <= i && i < self.len@ ==>
        self.frozen.unwrap_logic().shared.val().cur().slot(self.ptr@.unwrap_logic().2 + i) == Some(Some(result@[i]))))]
    fn as_slice(&self) -> &[u8] {
        #[cfg(not(bytes_proof_frozen))]
        { unsafe { slice::from_raw_parts(self.ptr, self.len) } }
        #[cfg(bytes_proof_frozen)]
        {
            let reader = self.frozen.as_ref().unwrap();
            crate::ownership_proof::frozen_region::borrow_frozen(
                &self.ptr, self.len, reader.shared, &reader.ticket,
            )
        }
    }
    // BEGIN EXACT FROZEN BYTES SHARE
    /// Explicit sequential sharing adapter. Its mutable receiver splits an
    /// affine read ticket; native `Clone::clone(&self)` has a separate frontier.
    #[cfg(bytes_proof_frozen)]
    #[cfg_attr(creusot, requires(self.frozen != None && self.frozen.unwrap_logic().valid(self.ptr,self.len)))]
    #[cfg_attr(creusot, requires(at <= self.len && len <= self.len-at))]
    #[cfg_attr(creusot, ensures(result.frozen != None && result.frozen.unwrap_logic().valid(result.ptr,result.len)))]
    #[cfg_attr(creusot, ensures((^self).frozen != None && (^self).frozen.unwrap_logic().valid((^self).ptr,(^self).len)))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len))]
    #[cfg_attr(creusot, ensures(result.len == len))]
    #[cfg_attr(creusot, ensures(result.ptr@.unwrap_logic().2 == self.ptr@.unwrap_logic().2 + at@))]
    #[cfg_attr(creusot, ensures(result.frozen.unwrap_logic().shared == self.frozen.unwrap_logic().shared && (^self).frozen.unwrap_logic().shared == self.frozen.unwrap_logic().shared))]
    #[cfg_attr(creusot, ensures(result.frozen.unwrap_logic().ticket.frac() + (^self).frozen.unwrap_logic().ticket.frac() == self.frozen.unwrap_logic().ticket.frac()))]
    pub(crate) fn proof_share_frozen(&mut self,at:usize,len:usize)->Bytes {
        let reader=self.frozen.as_mut().unwrap();
        let ticket=reader.ticket.split_off();
        let shared=reader.shared;
        let ptr=self.ptr.advance_within(at);
        unsafe { Bytes::with_vtable(ptr,len,AtomicPtr::new(core::ptr::null_mut()),self.vtable,
            crate::ownership_proof::frozen_region::FrozenReader{shared,ticket}) }
    }
    // BEGIN EXACT FROZEN BYTES CLOSE
    /// Explicit extraction-only cleanup entry. Automatic Drop is not modeled.
    #[cfg(bytes_proof_frozen)]
    #[cfg_attr(creusot, requires(self.frozen != None))]
    #[cfg_attr(creusot, ensures(result.lft() == self.frozen.unwrap_logic().ticket.lft()))]
    #[cfg_attr(creusot, ensures(result.frac() == self.frozen.unwrap_logic().ticket.frac()))]
    pub(crate) fn proof_return_frozen_ticket(mut self) -> creusot_std::ghost::lifetime_logic::LifetimeToken {
        let reader = self.frozen.take().unwrap();
        core::mem::forget(self);
        reader.ticket
    }
}
use creusot_std::ghost::{
    lifetime_logic::{FullBorrow, LifetimeToken}, GhostShared,
};
use crate::frozen_region::FrozenReader;

#[requires(at@ <= input@.len())]
#[ensures(input@.len() > 0 ==> result.0 == input@[0])]
#[ensures(at@ < input@.len() ==> result.1 == input@[at@])]
pub(crate) fn actual_receiver_views(input: Vec<u8>, at: usize, table: &'static Vtable) -> (u8, u8) {
    let (base, len, capacity, capabilities) = crate::bound_ptr::detach_bound_vec(input);
    let (recovery, region) = capabilities.split();
    let lifetime = LifetimeToken::new();
    let (full, end) = FullBorrow::new(region, snapshot!(lifetime.lft()));
    let shared = ghost! { GhostShared::new(full).into_inner() };
    let (first, second) = lifetime.split();
    let suffix = base.advance_within(at);
    let first = unsafe { Bytes::with_vtable(base, len, AtomicPtr::new(core::ptr::null_mut()), table,
        FrozenReader { shared, ticket: first }) };
    let second = unsafe { Bytes::with_vtable(suffix, len - at, AtomicPtr::new(core::ptr::null_mut()), table,
        FrozenReader { shared, ticket: second }) };
    let all = first.as_slice();
    let tail = second.as_slice();
    proof_assert!(forall<i: Int> 0 <= i && i < (len-at)@ ==> tail@[i] == all@[at@+i]);
    let a = if len == 0 { 0 } else { all[0] };
    let b = if len == at { 0 } else { tail[0] };
    let _ = (all, tail);
    let first = first.proof_return_frozen_ticket();
    let second = second.proof_return_frozen_ticket();
    let dead = second.join(first).end();
    let region = ghost! { end.into_inner().get(dead) };
    unsafe { crate::raw_vec::deallocate_bound_vec(base, capacity,
        ghost! { (recovery.into_inner(), region.into_inner()) }); }
    (a,b)
}

#[cfg(feature = "negative_shared_clone")]
impl Bytes {
    // Exact receiver and signature of Clone::clone. The attempted affine
    // witness update is rejected because cloning only grants shared access.
    fn attempt_clone_from_shared_receiver(&self) -> LifetimeToken {
        self.frozen.as_ref().unwrap().ticket.split_off()
    }
}

#[cfg(all(test, not(creusot)))]
mod tests {
    use super::*;
    unsafe fn forbidden_clone(_: &AtomicPtr<()>, _: *const u8, _: usize, _: &'static Vtable) -> Bytes { panic!("dispatch excluded") }
    unsafe fn forbidden_vec(_: &AtomicPtr<()>, _: *const u8, _: usize) -> Vec<u8> { panic!("dispatch excluded") }
    unsafe fn forbidden_mut(_: &AtomicPtr<()>, _: *const u8, _: usize) -> crate::BytesMut { panic!("dispatch excluded") }
    unsafe fn forbidden_unique(_: &AtomicPtr<()>) -> bool { panic!("dispatch excluded") }
    unsafe fn forbidden_drop(_: &mut AtomicPtr<()>, _: *const u8, _: usize) { panic!("dispatch excluded") }
    static TABLE: Vtable = Vtable { clone: forbidden_clone, into_vec: forbidden_vec, into_mut: forbidden_mut, is_unique: forbidden_unique, drop: forbidden_drop };
    #[test]
    fn actual_receiver_reads_and_explicit_close() {
        for values in [alloc::vec![], alloc::vec![7], alloc::vec![2,4,6,8]] {
            for at in 0..=values.len() {
                let expected = (values.first().copied().unwrap_or(0), values.get(at).copied().unwrap_or(0));
                assert_eq!(actual_receiver_views(values.clone(), at, &TABLE), expected);
                assert_eq!(actual_freeze_then_read_and_reclaim(values.clone(), &TABLE),expected.0);
            }
        }
    }
}

#[ensures(input@.len() > 0 ==> result == input@[0])]
pub(crate) fn actual_freeze_then_read_and_reclaim(input:Vec<u8>,table:&'static Vtable)->u8 {
    let mutable=BytesMut::from_vec(input);
    let mut coordinator=None;
    let mut bytes=mutable.freeze(&mut coordinator,table);
    let other=bytes.proof_share_frozen(0,bytes.len);
    let contents=bytes.as_slice();
    let first=if contents.len() == 0 {0} else {contents[0]};
    let _=contents;
    let ticket=bytes.proof_return_frozen_ticket();
    let other_ticket=other.proof_return_frozen_ticket();
    let ticket=ticket.join(other_ticket);
    proof_assert!(ticket.frac().ext_eq(creusot_std::logic::real::PositiveReal::from_int(1)));
    coordinator.unwrap().reclaim(ticket);
    first
}
