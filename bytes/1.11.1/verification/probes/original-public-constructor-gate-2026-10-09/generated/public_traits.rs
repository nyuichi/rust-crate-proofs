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
// ORIGINAL_CONSTRUCTOR_BEGIN bytes_from_box_impl
impl From<Box<[u8]>> for Bytes {
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_valid()))]
    #[cfg_attr(all(creusot, bytes_original_constructor_gate), ensures(result.original_bytes_content() == slice@))]
    fn from(slice: Box<[u8]>) -> Bytes {
        #[cfg(all(creusot, bytes_original_constructor_gate))]
        { return original_bytes_from_box(slice); }
        #[cfg(not(all(creusot, bytes_original_constructor_gate)))]
        {
        // Box<[u8]> doesn't contain a heap allocation for empty slices,
        // so the pointer isn't aligned enough for the KIND_VEC stashing to
        // work.
        if slice.is_empty() {
            return Bytes::new();
        }

        let len = slice.len();
        let ptr = Box::into_raw(slice) as *mut u8;

        if ptr as usize & 0x1 == 0 {
            let data = ptr_map(ptr, |addr| addr | KIND_VEC);
            Bytes {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr,
                len,
                data: AtomicPtr::new(data.cast()),
                vtable: &PROMOTABLE_EVEN_VTABLE,
            }
        } else {
            Bytes {
                #[cfg(all(creusot, bytes_original_freeze_gate))]
                original_frozen: None,
                ptr,
                len,
                data: AtomicPtr::new(ptr.cast()),
                vtable: &PROMOTABLE_ODD_VTABLE,
            }
        }
        }
    }
}
// ORIGINAL_CONSTRUCTOR_END bytes_from_box_impl
impl Bytes {
// ORIGINAL_CONSTRUCTOR_BEGIN bytes_new
    #[inline]
    #[cfg(not(any(all(loom, test), all(creusot, bytes_original_constructor_gate))))]
    pub const fn new() -> Self {
        // Make it a named const to work around
        // "unsizing casts are not allowed in const fn"
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
    // ORIGINAL_CONSTRUCTOR_END bytes_new
// ORIGINAL_CONSTRUCTOR_BEGIN bytes_from_static
    #[inline]
    #[cfg(not(any(all(loom, test), all(creusot, bytes_original_constructor_gate))))]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    /// Creates a new `Bytes` from a static slice.
    #[cfg(all(loom, test, not(all(creusot, bytes_original_constructor_gate))))]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            #[cfg(all(creusot, bytes_original_freeze_gate))]
            original_frozen: None,
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &STATIC_VTABLE,
        }
    }

    #[cfg(all(creusot, bytes_original_constructor_gate))]
    #[ensures(result.original_bytes_valid())]
    #[ensures(result.original_bytes_content() == bytes@)]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        original_bytes_from_static(bytes)
    }
    // ORIGINAL_CONSTRUCTOR_END bytes_from_static
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
