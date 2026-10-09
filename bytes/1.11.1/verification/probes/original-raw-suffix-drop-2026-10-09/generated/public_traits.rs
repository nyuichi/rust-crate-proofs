// ORIGINAL_SHARED_BEGIN bytes_clone_impl
impl Clone for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_bytes() == self.original_shared_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.ptr == self.ptr && result.len == self.len))]
    #[inline]
    fn clone(&self) -> Bytes {
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { original_shared_clone(self) }
        #[cfg(not(all(creusot, bytes_original_shared_gate)))]
        unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }
    }
}
// ORIGINAL_SHARED_END bytes_clone_impl
// ORIGINAL_SHARED_BEGIN bytes_from_vec_impl
impl From<Vec<u8>> for Bytes {
// ORIGINAL_FREEZE_BEGIN bytes_from_vec
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(vec@.len() < creusot_std::std::vec::capacity_model(vec)))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result.original_shared_bytes() == vec@))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(vec@.len() < creusot_std::std::vec::capacity_model(vec)))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_bytes() == vec@))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result.original_frozen_pointer() == creusot_std::std::vec::pointer_model(vec) as *const u8))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_content() == vec@))]
    fn from(vec: Vec<u8>) -> Bytes {
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { return original_bytes_from_vec(vec); }
        #[cfg(all(creusot, bytes_original_shared_gate))]
        { return original_shared_from_vec(vec); }
        #[cfg(not(all(creusot, any(bytes_original_shared_gate, bytes_original_constructor_gate))))]
        {
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let mut vec = ManuallyDrop::new(vec);
        #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
        let (ptr, len, cap) = (vec.as_mut_ptr(), vec.len(), vec.capacity());
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
                let vec = ManuallyDrop::into_inner(vec);
                return Bytes::from(vec.into_boxed_slice());
            }
        }

        let shared = Box::new(Shared {
            buf: ptr,
            cap,
            ref_cnt: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_atomic_usize_new(1) }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { AtomicUsize::new(1) }
            },
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
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: Some(OriginalFrozenProof { base, capabilities, shared, shared_owner }),
            ptr,
            len,
            data: {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                { original_atomic_ptr_new(shared as _) }
                #[cfg(not(all(creusot, bytes_original_freeze_gate)))]
                { AtomicPtr::new(shared as _) }
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
// ORIGINAL_FREEZE_END bytes_from_vec
}

// ORIGINAL_SHARED_END bytes_from_vec_impl
impl Bytes {
// ORIGINAL_SHARED_BEGIN bytes_cleanup
    /// Releases this handle explicitly. The backing storage is reclaimed when
    /// its last owner releases it.
    ///
    /// This calls the same destructor as `Drop`, exactly once.
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
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
    // ORIGINAL_SHARED_END bytes_cleanup
// ORIGINAL_SHARED_BEGIN bytes_as_slice
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), requires(self.original_frozen_valid()))]
    #[cfg_attr(all(creusot, bytes_original_freeze_gate), ensures(result@ == self.original_frozen_bytes()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), requires(self.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
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
        #[cfg(not(any(all(creusot, bytes_original_freeze_gate), all(creusot, bytes_original_shared_gate), all(creusot, bytes_original_constructor_gate))))]
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
    // ORIGINAL_SHARED_END bytes_as_slice
}
// ORIGINAL_SHARED_BEGIN bytes_as_ref_impl
impl AsRef<[u8]> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_shared_gate), requires(self.original_shared_valid()))]
    #[cfg_attr(all(creusot, bytes_original_shared_gate), ensures(result@ == self.original_shared_bytes()))]
    #[inline]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result@ == self.original_bytes_content()))]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
// ORIGINAL_SHARED_END bytes_as_ref_impl
