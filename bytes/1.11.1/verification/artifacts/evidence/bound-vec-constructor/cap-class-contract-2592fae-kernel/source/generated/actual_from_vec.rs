// Generated from exact source fragments by build.rs; do not edit.
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop};
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use creusot_std::prelude::*;
use crate::capacity_ops::original_capacity_to_repr;

pub struct BytesMut {
    #[cfg(not(creusot))]
    ptr: NonNull<u8>,
    #[cfg(creusot)]
    ptr: crate::ownership_proof::raw_vec::BoundPtr,
    len: usize,
    cap: usize,
    data: *mut Shared,
    // Present only in proof builds. Other construction paths remain outside
    // the offset-zero unique ownership gate until their protocols are proved.
    #[cfg(creusot)]
    unique_at_zero: Ghost<Option<(
        crate::ownership_proof::raw_vec::Recovery,
        crate::ownership_proof::raw_vec::PhysicalRegion,
    )>>,
}

struct Shared {
    vec: Vec<u8>,
    original_capacity_repr: usize,
    ref_count: AtomicUsize,
}

const KIND_VEC: usize = 0b1;

const KIND_MASK: usize = 0b1;

#[inline]
#[cfg(not(creusot))]
fn vptr(ptr: *mut u8) -> NonNull<u8> {
    if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

// Unadapted pointer updates carry no physical binding. In particular,
// replacing a pointer cannot preserve a previous allocation witness.
#[cfg(creusot)]
#[inline]
#[requires(!ptr.is_null_logic())]
fn vptr(ptr: *mut u8) -> crate::ownership_proof::raw_vec::BoundPtr {
    let pointer = if cfg!(debug_assertions) {
        NonNull::new(ptr).expect("Vec pointer should be non-null")
    } else {
        unsafe { NonNull::new_unchecked(ptr) }
    };
    crate::ownership_proof::raw_vec::BoundPtr::unbound(pointer)
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
        #[cfg(not(creusot))]
        let (ptr, len, cap) = {
            let mut vec = ManuallyDrop::new(vec);
            (vptr(vec.as_mut_ptr()), vec.len(), vec.capacity())
        };
        #[cfg(creusot)]
        let (ptr, len, cap, capabilities) =
            crate::ownership_proof::bound_ptr::detach_bound_vec(vec);

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = crate::capacity_ops::pack_vec_metadata(original_capacity_repr);

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
            #[cfg(creusot)]
            unique_at_zero: ghost! { Some(capabilities.into_inner()) },
        }
    }

    #[cfg(creusot)]
    #[logic(prophetic)]
    pub(crate) fn proof_unique_at_zero_valid(self) -> bool {
        pearlite! {
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

    // Explicit restricted proof path. Forgetting the handle after B3 prevents
    // its ordinary destructor from freeing the same native allocation again.
    // This does not establish scope-exit Drop or a Shared cleanup protocol.
    #[cfg(creusot)]
    #[requires(self.proof_unique_at_zero_valid())]
    #[requires(self.data.addr_logic() & KIND_MASK == KIND_VEC)]
    #[requires(self.data.addr_logic() >> crate::capacity_ops::VEC_POS_OFFSET == 0usize)]
    pub(crate) fn proof_release_unique_at_zero(mut self) {
        let state = mem::replace(&mut self.unique_at_zero, ghost! { None });
        let capabilities = ghost! { state.into_inner().unwrap() };
        // SAFETY: the predicate provides exact offset-zero full authority.
        unsafe {
            crate::ownership_proof::raw_vec::deallocate_bound_vec(
                self.ptr, self.cap, capabilities,
            );
        }
        mem::forget(self);
    }

}
