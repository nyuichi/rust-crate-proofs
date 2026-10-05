use alloc::{vec::Vec,boxed::Box};
use core::mem::{self,ManuallyDrop,MaybeUninit};
use core::ptr::{self,NonNull};
use core::cmp;
use core::sync::atomic::{AtomicUsize,Ordering};
use creusot_std::prelude::*;
use crate::capacity_ops::{original_capacity_to_repr,MAX_VEC_POS};
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
#[inline]
#[cfg_attr(creusot, ensures(result.addr_logic() == addr))]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    // This null-derived pointer stores integer metadata only. It carries no
    // allocation permission and must not be used as a dereferenceable pointer.
    let ptr = crate::provenance_specs::metadata_pointer(addr);
    debug_assert_eq!(crate::provenance_specs::pointer_addr(ptr), addr);
    ptr.cast::<T>()
}
// BEGIN EXACT INCREMENT_SHARED
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(ptr == control.pointer))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(context.inner_logic().valid(control) && context.inner_logic().active()))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), requires((*context.inner_logic().status.pending).len() < isize::MAX@))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^context.inner_logic()).valid_count(control, 1) && (^context.inner_logic()).active()))]
#[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^context.inner_logic()).status == context.inner_logic().status))]
unsafe fn increment_shared(ptr: *mut Shared,
    #[cfg(bytes_proof_repeated_split)] control: sequential_shared_control::ControlPtr,
    #[cfg(bytes_proof_repeated_split)] context: Ghost<&mut sequential_shared_control::ControlContext>,
) {
    #[cfg(not(bytes_proof_repeated_split))]
    let old_size = (*ptr).ref_count.fetch_add(1, Ordering::Relaxed);
    #[cfg(bytes_proof_repeated_split)]
    let old_size = sequential_shared_control::increment(control, context);
    if old_size > isize::MAX as usize {
        crate::abort();
    }
}
const KIND_VEC: usize = 0b1;
const KIND_ARC: usize = 0b0;
const KIND_MASK: usize = 0b1;
#[path = "/workspace/bytes-runtime-verification/bytes/1.11.1/src/ownership_proof/carrier_protocol.rs"]
pub(crate) mod sequential_shared_control;
impl BytesMut {
    // BEGIN EXACT GET_VEC_POS
    #[inline]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures(result == self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET))]
    unsafe fn get_vec_pos(&self) -> usize {
        debug_assert_eq!(self.kind(), KIND_VEC);

        crate::capacity_ops::vec_pos_from_data(
            crate::provenance_specs::pointer_addr(self.data),
        )
    }
    // BEGIN EXACT SET_VEC_POS
    #[inline]
    #[cfg_attr(creusot, requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, requires(pos <= crate::capacity_ops::MAX_VEC_POS))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == pos))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    unsafe fn set_vec_pos(&mut self, pos: usize) {
        debug_assert_eq!(self.kind(), KIND_VEC);
        debug_assert!(pos <= MAX_VEC_POS);

        self.data = invalid_ptr(crate::capacity_ops::set_vec_pos_in_data(
            crate::provenance_specs::pointer_addr(self.data),
            pos,
        ));
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
    // BEGIN EXACT AS_SLICE
    #[inline]
    #[cfg_attr(creusot, requires(self.proof_initialized()))]
    #[cfg_attr(creusot, ensures(result@.len() == self.len@))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ ==>
        self.proof_view_slot(index) == Some(Some(result@[index]))))]
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
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.len))]
    pub fn len(&self) -> usize {
        self.len
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.cap))]
    pub fn capacity(&self) -> usize {
        self.cap
    }
    #[inline]
    #[cfg_attr(creusot, ensures(result == (self.data.addr_logic() & KIND_MASK)))]
    fn kind(&self) -> usize {
        crate::provenance_specs::pointer_addr(self.data) & KIND_MASK
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
    // BEGIN EXACT SPLIT_OFF
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(at <= self.cap))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.proof_split_pair_valid(^self)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr@ == self.ptr@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, at@))))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).len@ == if self.len < at { self.len@ } else { at@ }))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.len@ == if self.len < at { 0int } else { self.len@ - at@ }))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < (^self).cap@ ==>
        (^self).proof_view_slot(index) == self.proof_unique_slot(index)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < result.cap@ ==>
        result.proof_view_slot(index) == self.proof_unique_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready() && at <= self.cap))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_carrier_pair((^self))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).ptr@ == self.ptr@ && result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).len@ == (if self.len < at { self.len@ } else { at@ }) && result.len@ == (if self.len < at { 0int } else { self.len@ - at@ }) && (^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_initialized() && (^self).proof_initialized()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < at@ ==> (^self).proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < self.cap@ - at@ ==> result.proof_view_slot(index) == self.proof_view_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(result.shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((* result.shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.shared_context.inner_logic() == None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) ==>
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.proof_carrier_initialized() && (^self).proof_carrier_initialized()))]
    #[must_use = "consider BytesMut::truncate if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> BytesMut {
        assert!(
            at <= self.capacity(),
            "split_off out of bounds: {:?} <= {:?}",
            at,
            self.capacity(),
        );
        unsafe {
            let mut other = self.shallow_clone();
            #[cfg(bytes_proof_repeated_split)]
            { self.proof_finish_carrier_split_off(other, at) }
            #[cfg(not(bytes_proof_repeated_split))]
            {
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            {
                // The returned right handle owns the sole coordinator, as in split_to.
                other.pending_control = ghost! { self.pending_control.take() };
                other.proof_register_split(self, at);
            }
            // SAFETY: We've checked that `at` <= `self.capacity()` above.
            other.advance_unchecked(at);
            self.cap = at;
            self.len = cmp::min(self.len, at);
            other
            }
        }
    }
    // BEGIN EXACT SPLIT_TO
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(at <= self.len))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_split_pair_valid(result)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr@ == self.ptr@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, at@))))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.len == at && (^self).len@ == self.len@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < at@ ==>
        result.proof_view_slot(index) == self.proof_unique_slot(index)))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==>
        (^self).proof_view_slot(index) == self.proof_unique_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready() && at <= self.len))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_pair(result)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.ptr@ == self.ptr@ && (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.len == at && (^self).len@ == self.len@ - at@ && result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < at@ ==> result.proof_view_slot(index) == self.proof_view_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==> (^self).proof_view_slot(index) == self.proof_view_slot(index + at@)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.shared_context.inner_logic() == None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.matches(self.shared_context.inner_logic().unwrap_logic()) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_initialized() && result.proof_carrier_initialized()))]
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
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            self.proof_register_split(&mut other, at);
            #[cfg(bytes_proof_repeated_split)]
            { self.proof_finish_carrier_split(other, at) }
            #[cfg(not(bytes_proof_repeated_split))]
            {
                // SAFETY: We've checked that `at` <= `self.len()` and we know that `self.len()` <=
                // `self.capacity()`.
                self.advance_unchecked(at);
                other.cap = at;
                other.len = at;
                other
            }
        }
    }
    // BEGIN EXACT ADVANCE_UNCHECKED
    #[cfg_attr(creusot, requires(self.ptr.invariant() && self.ptr@ != None))]
    #[cfg_attr(creusot, requires(self.ptr@.unwrap_logic().2 + self.cap@ <= self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, requires(count <= self.cap && self.len <= self.cap))]
    #[cfg_attr(creusot, requires((self.data.addr_logic() & KIND_MASK == KIND_ARC && self.proof_shared_allocation_registered()) ||
        (self.proof_unique_owned() && self.ptr@.unwrap_logic().2 + count@ <= crate::capacity_ops::MAX_VEC_POS@)))]
    #[cfg_attr(creusot, ensures((^self).ptr.invariant()))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + count@))))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if count <= self.len { self.len@ - count@ } else { 0int }) && (^self).cap@ == self.cap@ - count@))]
    #[cfg_attr(creusot, ensures(self.data.addr_logic() & KIND_MASK == KIND_ARC ==> (^self).data == self.data))]
    #[cfg_attr(creusot, ensures(self.proof_unique_owned() ==> (^self).proof_unique_owned()))]
    #[cfg_attr(creusot, ensures((^self).data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK == self.data.addr_logic() & crate::capacity_ops::NOT_VEC_POS_MASK))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero == self.unique_at_zero && (^self).pending_control == self.pending_control && (^self).shared_registration == self.shared_registration && (^self).shared_context == self.shared_context))]
    #[cfg_attr(creusot, ensures(forall<index: Int> (^self).proof_owned_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(creusot, ensures(self.proof_registered_valid() ==> (^self).proof_registered_valid()))]
    #[cfg_attr(creusot, ensures(self.proof_initialized() ==> (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int>
        (^self).proof_view_slot(index) == self.proof_view_slot(index + count@)))]
    pub(crate) unsafe fn advance_unchecked(&mut self, count: usize) {
        // Setting the start to 0 is a no-op, so return early if this is the
        // case.
        if count == 0 {
            return;
        }

        debug_assert!(count <= self.cap, "internal: set_start out of bounds");

        let kind = self.kind();

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
                #[cfg(not(any(creusot, bytes_proof_probe)))]
                self.promote_to_shared(/*ref_count = */ 1);
                #[cfg(any(creusot, bytes_proof_probe))]
                panic!("unique offset overflow promotion is outside this proof gate");
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
    #[cfg_attr(creusot, ensures((^self).pending_control.inner_logic().unwrap_logic().caps == self.unique_at_zero.inner_logic().unwrap_logic()))]
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
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.proof_unique_at_zero_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() & KIND_MASK == KIND_VEC))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).proof_pending_valid()))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.ptr == (^self).ptr && result.len == (^self).len && result.cap == (^self).cap && result.data == (^self).data))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg_attr(all(creusot, not(bytes_proof_repeated_split)), ensures((^self).pending_control.inner_logic().unwrap_logic().caps == self.unique_at_zero.inner_logic().unwrap_logic()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), requires(self.proof_carrier_ready()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).proof_carrier_pending()))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(((^self).shared_context.inner_logic() == None) == (self.shared_context.inner_logic() == None)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_context.inner_logic() == None ==> (^self).pending_control.inner_logic() != None))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.ptr == (^self).ptr && result.len == (^self).len && result.cap == (^self).cap && result.data == (^self).data))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(result.unique_at_zero.inner_logic() == None && result.pending_control.inner_logic() == None && result.shared_registration.inner_logic() == None && result.shared_context.inner_logic() == None))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> (^self).proof_carrier_slot(index) == self.proof_owned_slot(index)))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known((^self).proof_carrier_slot(index))))]
    #[cfg_attr(all(creusot, bytes_proof_repeated_split), ensures(self.shared_context.inner_logic() != None ==> (^self).shared_registration == self.shared_registration &&
        (^self).shared_context.inner_logic().unwrap_logic().status == self.shared_context.inner_logic().unwrap_logic().status))]
    #[inline]
    unsafe fn shallow_clone(&mut self) -> BytesMut {
        if self.kind() == KIND_ARC {
            #[cfg(not(any(creusot, bytes_proof_probe)))]
            increment_shared(self.data);
            #[cfg(all(any(creusot, bytes_proof_probe), not(bytes_proof_repeated_split)))]
            panic!("ARC clone is outside the first sequential split gate");
            #[cfg(bytes_proof_repeated_split)]
            {
                let identity = ghost! { self.shared_registration.as_ref().unwrap().control.identity.into_inner() };
                let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
                increment_shared(self.data, control, ghost! { self.shared_context.as_mut().unwrap() });
            }
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
    fn proof_initialized(self) -> bool {
        pearlite! {
            self.proof_owned_valid() &&
            forall<index: Int> 0 <= index && index < self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_view_slot(index))
        }
    }
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
    fn proof_shared_allocation_registered(self) -> bool {
        pearlite! {
            self.data.addr_logic() & KIND_MASK == KIND_ARC && self.ptr@ != None &&
            match self.shared_registration.inner_logic() {
                None => false,
                Some(registration) => registration.valid() &&
                    registration.control.pointer == self.data &&
                    self.ptr@.unwrap_logic().0 == registration.status.allocation &&
                    self.ptr@.unwrap_logic().1 == registration.status.capacity,
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
    #[cfg(all(creusot, not(bytes_proof_valid_handle)))]
    #[logic(prophetic)]
    fn proof_owned_valid(self) -> bool {
        pearlite! { self.proof_unique_owned() || self.proof_registered_valid() }
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
    // BEGIN EXACT CARRIER SPLIT METHODS
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_initialized(self) -> bool {
        pearlite! { self.proof_owned_valid() &&
            forall<index: Int> self.ptr@.unwrap_logic().2 <= index &&
                index < self.ptr@.unwrap_logic().2 + self.len@ ==>
                crate::ownership_proof::raw_vec::slot_known(self.proof_owned_slot(index)) }
    }
    /// A change of coordinates, with the same physical slots and ownership.
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, check(ghost))]
    #[cfg_attr(creusot, ensures((*handle).proof_initialized() == (*handle).proof_carrier_initialized()))]
    fn proof_initialization_coordinates(handle: Snapshot<Self>) {
        proof_assert!(forall<index: Int> (*handle).proof_view_slot(index) ==
            (*handle).proof_owned_slot(handle.ptr@.unwrap_logic().2 + index));
        proof_assert!(forall<index: Int> (*handle).proof_owned_slot(index) ==
            (*handle).proof_view_slot(index - handle.ptr@.unwrap_logic().2));
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.len))]
    #[cfg_attr(creusot, ensures((^self).proof_carrier_pair(result)))]
    #[cfg_attr(creusot, ensures(result.ptr@ == self.ptr@ && (^self).ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(creusot, ensures(result.len == at && (^self).len@ == self.len@ - at@ && result.cap == at && (^self).cap@ == self.cap@ - at@))]
    #[cfg_attr(creusot, ensures((^self).proof_initialized() && result.proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < at@ ==> result.proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.len@ - at@ ==> (^self).proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index + at@)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_carrier_slot(index))))]
    #[cfg_attr(creusot, ensures((^self).proof_carrier_initialized() && result.proof_carrier_initialized()))]
    fn proof_finish_carrier_split(&mut self, mut other: Self, at: usize) -> Self {
        self.proof_register_carrier_split(&mut other, at);
        unsafe { self.advance_unchecked(at); }
        other.cap = at;
        other.len = at;
        ghost! { Self::proof_initialization_coordinates(snapshot!(*self)); };
        ghost! { Self::proof_initialization_coordinates(snapshot!(other)); };
        other
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.cap))]
    #[cfg_attr(creusot, ensures(result.proof_carrier_pair((^self))))]
    #[cfg_attr(creusot, ensures((^self).ptr@ == self.ptr@ && result.ptr@ == Some((self.ptr@.unwrap_logic().0, self.ptr@.unwrap_logic().1, self.ptr@.unwrap_logic().2 + at@))))]
    #[cfg_attr(creusot, ensures((^self).len@ == (if self.len < at { self.len@ } else { at@ }) && result.len@ == (if self.len < at { 0int } else { self.len@ - at@ }) && (^self).cap == at && result.cap@ == self.cap@ - at@))]
    #[cfg_attr(creusot, ensures(result.proof_initialized() && (^self).proof_initialized()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < at@ ==> (^self).proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> 0 <= index && index < self.cap@ - at@ ==> result.proof_view_slot(index) == self.proof_carrier_slot(self.ptr@.unwrap_logic().2 + index + at@)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(result.shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* result.shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() && registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != result.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + self.len@ ==>
        crate::ownership_proof::raw_vec::slot_known(self.proof_carrier_slot(index))))]
    #[cfg_attr(creusot, ensures(result.proof_carrier_initialized() && (^self).proof_carrier_initialized()))]
    fn proof_finish_carrier_split_off(&mut self, mut other: Self, at: usize) -> Self {
        // Move the affine state, never copy it with the equal descriptor.
        other.pending_control = ghost! { self.pending_control.take() };
        other.shared_registration = ghost! { self.shared_registration.take() };
        other.shared_context = ghost! { self.shared_context.take() };
        other.proof_register_carrier_split(self, at);
        unsafe { other.advance_unchecked(at); }
        self.cap = at;
        self.len = cmp::min(self.len, at);
        ghost! { Self::proof_initialization_coordinates(snapshot!(*self)); };
        ghost! { Self::proof_initialization_coordinates(snapshot!(other)); };
        other
    }

    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_ready(self) -> bool {
        pearlite! {
            self.proof_carrier_initialized() && (self.proof_unique_at_zero_valid() ||
                match (self.shared_registration.inner_logic(), self.shared_context.inner_logic()) {
                    (Some(registration), Some(context)) => registration.matches(context) &&
                        (*context.status.pending).len() < isize::MAX@,
                    _ => false,
                })
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_pending(self) -> bool {
        pearlite! {
            self.proof_pending_valid() || (self.proof_registered_valid() &&
                match (self.shared_registration.inner_logic(), self.shared_context.inner_logic()) {
                    (Some(registration), Some(context)) => registration.matches_incremented(context),
                    _ => false,
                })
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic]
    fn proof_carrier_slot(self, index: Int) -> Option<Option<u8>> {
        pearlite! {
            match self.pending_control.inner_logic() {
                Some(pending) => pending.caps.1.slot(index),
                None => self.proof_owned_slot(index),
            }
        }
    }
    #[cfg(all(creusot, bytes_proof_repeated_split))]
    #[logic(prophetic)]
    fn proof_carrier_pair(self, other: Self) -> bool {
        pearlite! {
            self.proof_registered_valid() && other.proof_registered_valid() &&
            other.shared_context.inner_logic() == None &&
            self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() !=
                other.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            match self.shared_context.inner_logic() {
                Some(context) => self.shared_registration.inner_logic().unwrap_logic().matches(context) &&
                    other.shared_registration.inner_logic().unwrap_logic().matches(context),
                None => false,
            }
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_carrier_pending() && at <= self.cap))]
    #[cfg_attr(creusot, requires(other.ptr == self.ptr && other.len == self.len && other.cap == self.cap && other.data == self.data))]
    #[cfg_attr(creusot, requires(other.unique_at_zero.inner_logic() == None && other.pending_control.inner_logic() == None && other.shared_registration.inner_logic() == None && other.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).ptr == self.ptr && (^self).len == self.len && (^self).cap == self.cap && (^self).data == self.data))]
    #[cfg_attr(creusot, ensures((^other).ptr == other.ptr && (^other).len == other.len && (^other).cap == other.cap && (^other).data == other.data))]
    #[cfg_attr(creusot, ensures((^self).unique_at_zero.inner_logic() == None && (^self).pending_control.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^other).unique_at_zero.inner_logic() == None && (^other).pending_control.inner_logic() == None && (^other).shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, ensures((^self).shared_context.inner_logic() != None && (^self).shared_registration.inner_logic() != None && (^other).shared_registration.inner_logic() != None))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic()) &&
        (^other).shared_registration.inner_logic().unwrap_logic().matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data &&
        (^other).shared_registration.inner_logic().unwrap_logic().control.pointer == self.data))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().status.allocation == self.ptr@.unwrap_logic().0 &&
        (^self).shared_registration.inner_logic().unwrap_logic().status.capacity == self.ptr@.unwrap_logic().1 &&
        (^other).shared_registration.inner_logic().unwrap_logic().status.allocation == self.ptr@.unwrap_logic().0 &&
        (^other).shared_registration.inner_logic().unwrap_logic().status.capacity == self.ptr@.unwrap_logic().1))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().packet.1.lo() == self.ptr@.unwrap_logic().2 + at@ &&
        (^self).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.ptr@.unwrap_logic().2 + self.cap@))]
    #[cfg_attr(creusot, ensures((^other).shared_registration.inner_logic().unwrap_logic().packet.1.lo() <= self.ptr@.unwrap_logic().2 &&
        (^other).shared_registration.inner_logic().unwrap_logic().packet.1.hi() == self.ptr@.unwrap_logic().2 + at@))]
    #[cfg_attr(creusot, ensures((^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() !=
        (^other).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.ptr@.unwrap_logic().2 <= index && index < self.ptr@.unwrap_logic().2 + at@ ==>
        (^other).proof_owned_slot(index) == self.proof_carrier_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<index: Int> self.ptr@.unwrap_logic().2 + at@ <= index && index < self.ptr@.unwrap_logic().2 + self.cap@ ==>
        (^self).proof_owned_slot(index) == self.proof_carrier_slot(index)))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() &&
        registration.control == self.shared_registration.inner_logic().unwrap_logic().control &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches((^self).shared_context.inner_logic().unwrap_logic())))]
    #[cfg_attr(creusot, ensures((* (^self).shared_context.inner_logic().unwrap_logic().status.pending).len() ==
        if self.pending_control.inner_logic() != None { 2int } else { (*self.shared_context.inner_logic().unwrap_logic().status.pending).len() + 1 }))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        self.shared_context.inner_logic() != None && registration.valid() &&
        crate::ownership_proof::scalable_tickets::packet_matches(registration.packet, *self.shared_context.inner_logic().unwrap_logic().status) ==>
            registration.packet.0.logical_id() != (^self).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
            registration.packet.0.logical_id() != (^other).shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    fn proof_register_carrier_split(&mut self, other: &mut Self, at: usize) {
        let pointer = self.data;
        let base = self.ptr;
        let capacity = self.cap;
        let cut = snapshot!(self.ptr@.unwrap_logic().2 + at@);
        let assigned = ghost! {
            if let Some(pending) = self.pending_control.take() {
                sequential_shared_control::activate(pointer, base, capacity, cut, Ghost::new(pending)).into_inner()
            } else {
                let parent = self.shared_registration.take().unwrap();
                let mut context = self.shared_context.take().unwrap();
                let children = sequential_shared_control::split_more(Ghost::new(&mut context), Ghost::new(parent), cut);
                let (left, right) = children.into_inner();
                (context, left, right)
            }
        };
        let (context, left, right) = assigned.split();
        self.shared_context = ghost! { Some(context.into_inner()) };
        other.shared_registration = ghost! { Some(left.into_inner()) };
        self.shared_registration = ghost! { Some(right.into_inner()) };
    }

    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(self.proof_registered_valid() && self.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(self.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, ensures((^context.inner_logic()).valid(self.shared_registration.inner_logic().unwrap_logic().control)))]
    #[cfg_attr(creusot, ensures(result == !(^context.inner_logic()).active()))]
    #[cfg_attr(creusot, ensures((* (^context.inner_logic()).status.pending).len() + 1 == (*context.inner_logic().status.pending).len()))]
    #[cfg_attr(creusot, ensures(forall<registration: sequential_shared_control::HandleRegistration>
        registration.matches(*context.inner_logic()) &&
        registration.packet.0.logical_id() != self.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() ==>
            registration.matches(^context.inner_logic())))]
    #[cfg_attr(creusot, ensures(result == ((*context.inner_logic().status.pending).len() == 1)))]
    fn proof_carrier_release(mut self, context: Ghost<&mut sequential_shared_control::ControlContext>) -> bool {
        let registration = ghost! { self.shared_registration.take().unwrap() };
        let identity = ghost! { registration.control.identity.into_inner() };
        let control = sequential_shared_control::ControlPtr { pointer: self.data, identity };
        let last = sequential_shared_control::release(control, context, registration);
        mem::forget(self);
        last
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    #[cfg_attr(creusot, requires(a.proof_registered_valid() && b.proof_registered_valid() && c.proof_registered_valid()))]
    #[cfg_attr(creusot, requires(a.shared_context.inner_logic() == None && b.shared_context.inner_logic() == None && c.shared_context.inner_logic() == None))]
    #[cfg_attr(creusot, requires(a.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()) &&
        b.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic()) &&
        c.shared_registration.inner_logic().unwrap_logic().matches(*context.inner_logic())))]
    #[cfg_attr(creusot, requires(a.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != b.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
        a.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != c.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() &&
        b.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id() != c.shared_registration.inner_logic().unwrap_logic().packet.0.logical_id()))]
    #[cfg_attr(creusot, requires((*context.inner_logic().status.pending).len() == 3))]
    fn proof_carrier_release_three(a: Self, b: Self, c: Self,
        mut context: Ghost<&mut sequential_shared_control::ControlContext>, order: u8) {
        let (first, second, third) = match order % 6 {
            0 => (a, b, c), 1 => (a, c, b), 2 => (b, a, c),
            3 => (b, c, a), 4 => (c, a, b), _ => (c, b, a),
        };
        assert!(!first.proof_carrier_release(ghost! { &mut **context }));
        assert!(!second.proof_carrier_release(ghost! { &mut **context }));
        assert!(third.proof_carrier_release(context));
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_split(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut carrier = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let first = cmp::min(first, carrier.len());
        let mut left = carrier.split_to(first);
        let second = cmp::min(second, carrier.len());
        let mut middle = carrier.split_to(second);
        if left.len() > 0 { left.as_slice_mut()[0] = value; assert!(left.as_slice()[0] == value); }
        if middle.len() > 0 { middle.as_slice_mut()[0] = value; assert!(middle.as_slice()[0] == value); }
        if carrier.len() > 0 { carrier.as_slice_mut()[0] = value; assert!(carrier.as_slice()[0] == value); }
        let mut context = carrier.proof_take_coordinator();
        Self::proof_carrier_release_three(left, middle, carrier, context.borrow_mut(), order);
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_split_off(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut carrier = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let first = cmp::min(first, carrier.len());
        let mut left = carrier.split_to(first);
        let second = cmp::min(second, carrier.capacity());
        let mut right = carrier.split_off(second);
        if left.len() > 0 { left.as_slice_mut()[0] = value; assert!(left.as_slice()[0] == value); }
        if right.len() > 0 { right.as_slice_mut()[0] = value; assert!(right.as_slice()[0] == value); }
        if carrier.len() > 0 { carrier.as_slice_mut()[0] = value; assert!(carrier.as_slice()[0] == value); }
        let mut context = right.proof_take_coordinator();
        Self::proof_carrier_release_three(left, carrier, right, context.borrow_mut(), order);
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split, feature = "negative_carrier_missing_ticket"))]
    pub(crate) fn proof_carrier_missing_empty_ticket() {
        let mut carrier = Self::from_vec(Vec::new());
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        let left = carrier.split_to(0);
        let middle = carrier.split_to(0);
        let mut context = carrier.proof_take_coordinator();
        mem::forget(left);
        assert!(!middle.proof_carrier_release(context.borrow_mut()));
        // The unreleased empty left handle still owns one affine ticket.
        assert!(carrier.proof_carrier_release(context.borrow_mut()));
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split, feature = "negative_carrier_unknown_publication"))]
    pub(crate) fn proof_carrier_unknown_publication() {
        let mut carrier = Self::from_vec(Vec::with_capacity(2));
        ghost! { Self::proof_initialization_coordinates(snapshot!(carrier)); };
        if carrier.capacity() > 1 {
            let _left = carrier.split_to(0);
            let mut right = carrier.split_off(1);
            assert!(right.len() == 0);
            // The right capacity contains original Unknown slots; no write occurred.
            // Publishing one of them must fail set_len's initialized-byte guard.
            unsafe { right.set_len(1); }
        }
    }
    #[cfg(all(any(creusot, bytes_proof_probe), bytes_proof_repeated_split))]
    pub(crate) fn proof_carrier_storage(input: Vec<u8>, first: usize, second: usize, order: u8, value: u8) {
        let mut middle = Self::from_vec(input);
        ghost! { Self::proof_initialization_coordinates(snapshot!(middle)); };
        let mut left = middle.split_to(cmp::min(first, middle.len()));
        let mut right = middle.split_off(cmp::min(second, middle.capacity()));
        let old_len = right.len();
        if old_len < right.capacity() {
            {
                let spare = right.spare_capacity_mut();
                spare[0] = MaybeUninit::uninit();
                spare[0].write(value);
            }
            // Only the just-written spare byte extends the Known prefix.
            unsafe { right.set_len(old_len + 1); }
            assert!(right.as_slice()[old_len] == value);
        }
        let left_count = cmp::min(value as usize, left.capacity());
        let middle_count = cmp::min(value as usize, middle.capacity());
        let right_count = cmp::min(value as usize, right.capacity());
        let left_len = left.len();
        let middle_len = middle.len();
        let right_len = right.len();
        unsafe {
            left.advance_unchecked(left_count);
            middle.advance_unchecked(middle_count);
            right.advance_unchecked(right_count);
        }
        assert!(left.len() == left_len.saturating_sub(left_count));
        assert!(middle.len() == middle_len.saturating_sub(middle_count));
        assert!(right.len() == right_len.saturating_sub(right_count));
        left.truncate(first);
        middle.truncate(second);
        right.clear();
        assert!(right.len() == 0);
        let mut context = right.proof_take_coordinator();
        Self::proof_carrier_release_three(left, middle, right, context.borrow_mut(), order);
    }

}
#[cfg(not(creusot))]
impl Drop for SharedBuffer {
    fn drop(&mut self) {
        // u8 has no destructor; deallocation does not require initialized bytes.
        unsafe { drop(Vec::from_raw_parts(self.base.as_ptr(), 0, self.capacity)); }
    }
}