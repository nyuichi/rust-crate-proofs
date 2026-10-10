//! Opaque native storage for a Vec-backed shared allocation.
//!
//! This type keeps the original `Vec<u8>` value and allocation in place while
//! the bytes crate exposes disjoint raw-pointer views. It deliberately has no
//! `Deref`, slice conversion, or public vector access. The internal transfer
//! methods are currently native runtime operations; their B1/B2/B3 proof-side
//! authority contracts are still being developed.

use alloc::vec::Vec;
use core::mem::ManuallyDrop;

/// A native Vec allocation retained behind a non-accessible storage handle.
///
/// `repr(transparent)` preserves the size, alignment, and ABI of the wrapped
/// `ManuallyDrop<Vec<u8>>`; `ManuallyDrop` itself has the layout of `Vec<u8>`.
/// The wrapper is private to the crate and does not expose initialized bytes.
#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub(crate) struct VecStorage(ManuallyDrop<Vec<u8>>);

impl Default for VecStorage {
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn default() -> Self {
        Self::hold(Vec::new())
    }
}

impl Drop for VecStorage {
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    fn drop(&mut self) {
        // The wrapper's normal terminal path is the native Vec destructor.
        // Shared's proof-side drop contract must establish that no packet or
        // borrow remains before this generic storage retirement takes place.
        unsafe { ManuallyDrop::drop(&mut self.0) }
    }
}

#[cfg(not(creusot))]
const _: [(); core::mem::size_of::<Vec<u8>>()] = [(); core::mem::size_of::<VecStorage>()];
#[cfg(not(creusot))]
const _: [(); core::mem::align_of::<Vec<u8>>()] = [(); core::mem::align_of::<VecStorage>()];

impl VecStorage {
    /// Retain a Vec without exposing it to ordinary Vec operations.
    ///
    /// The matching physical allocation authority must be established by the
    /// B1 bridge before this value is used as shared storage.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn hold(vec: Vec<u8>) -> Self {
        Self(ManuallyDrop::new(vec))
    }

    /// Observe only the native Vec pointer and capacity metadata.
    ///
    /// No initialized slice or reference to the Vec is returned. The raw
    /// pointer grants no byte access by itself; access still requires the
    /// matching physical region capability.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) fn metadata(&mut self) -> (*mut u8, usize, usize) {
        let vec = &mut *self.0;
        (vec.as_mut_ptr(), vec.len(), vec.capacity())
    }

    /// Mutate Vec length after the caller has physically initialized the
    /// entire newly exposed interval and holds the matching exclusive region.
    ///
    /// # Safety
    ///
    /// `len` must not exceed capacity and every element in `0..len` must be
    /// initialized. The caller must also exclude every overlapping view.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn set_len(&mut self, len: usize) {
        unsafe { (&mut *self.0).set_len(len) }
    }

    /// Reserve Vec capacity while the caller exclusively owns the allocation.
    ///
    /// # Safety
    ///
    /// No pointer or region capability may be used across a possible
    /// reallocation. The caller must repair or replace all allocation-relative
    /// metadata after this call.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn reserve(&mut self, additional: usize) {
        (&mut *self.0).reserve(additional)
    }

    /// Transfer the hidden Vec to the generic B2 restoration boundary.
    ///
    /// # Safety
    ///
    /// The caller must have recovered all physical and lifetime authority for
    /// this allocation. It must use the returned Vec only after proving its
    /// requested initialized prefix and capacity. This method moves the Vec
    /// value; it does not rebind a stale Vec model after raw-pointer writes.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn take_for_restore(self) -> Vec<u8> {
        let mut storage = ManuallyDrop::new(self);
        // SAFETY: `storage` owns an initialized VecStorage and is itself
        // ManuallyDrop, so taking its sole Vec field transfers that owner
        // without running VecStorage's destructor.
        unsafe { ManuallyDrop::take(&mut storage.0) }
    }

    /// Explicitly transfer the retained Vec to a B2/B3 ownership boundary.
    ///
    /// # Safety
    ///
    /// The caller must own the matching Recovery and complete unborrowed
    /// PhysicalRegion and must invoke the exact generic B3 deallocator once.
    #[inline]
    #[cfg_attr(creusot, creusot::no_translate)]
    pub(crate) unsafe fn take_for_retirement(self) -> Vec<u8> {
        let mut storage = ManuallyDrop::new(self);
        // SAFETY: same ownership transfer as `take_for_restore`; the caller
        // must dispose the returned Vec exactly once through the B3 boundary.
        unsafe { ManuallyDrop::take(&mut storage.0) }
    }
}
