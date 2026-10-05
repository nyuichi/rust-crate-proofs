//! Native-only bridge for temporarily owning a `Vec<u8>` allocation as raw
//! pointer metadata.
//!
//! These are ordinary Rust unsafe bodies. They have no Creusot trusted
//! contracts and create no Creusot memory permissions. The caller must carry
//! the actual exclusive allocation ownership described by each safety
//! requirement.

use alloc::vec::Vec;
use core::mem::ManuallyDrop;

/// A detached global-allocator byte allocation.
///
/// This is intentionally not `Clone`, `Copy`, `Deref`, or `Drop`. Pointer and
/// capacity are metadata only; possession of this value is a native ownership
/// convention, not a verified permission.
pub(crate) struct RawBuffer {
    base: *mut u8,
    capacity: usize,
}

/// Detach a `Vec<u8>` without freeing its allocation.
///
/// The returned length marks the initialized prefix at detachment time. The
/// capacity and pointer are retained in `RawBuffer`; no bytes are read or
/// written by this operation.
pub(crate) fn detach(buffer: Vec<u8>) -> (RawBuffer, usize) {
    let mut buffer = ManuallyDrop::new(buffer);
    let len = buffer.len();
    let capacity = buffer.capacity();
    let base = buffer.as_mut_ptr();

    (RawBuffer { base, capacity }, len)
}

impl RawBuffer {
    /// Allocation base pointer.
    ///
    /// This reports pointer metadata only. It does not grant permission to
    /// dereference or mutate the allocation.
    pub(crate) fn base_ptr(&self) -> *mut u8 {
        self.base
    }

    /// Allocation capacity in bytes.
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    /// Rebuild the original allocation as a `Vec<u8>` with initialized prefix
    /// `0..len`.
    ///
    /// # Safety
    ///
    /// The caller must still exclusively own the complete allocation
    /// represented by `self`; no other owner may free it, and no live
    /// reference may access it. `len` must not exceed `self.capacity`, and
    /// every byte in `0..len` must be initialized. The pointer and capacity
    /// must still be the original pair obtained from the global allocator.
    pub(crate) unsafe fn into_vec(self, len: usize) -> Vec<u8> {
        let base = self.base;
        let capacity = self.capacity;
        // SAFETY: The caller promises the original allocation, unique full
        // ownership, `len <= capacity`, and an initialized prefix.
        unsafe { Vec::from_raw_parts(base, len, capacity) }
    }

    /// Deallocate the represented allocation without treating any spare
    /// capacity as initialized elements.
    ///
    /// # Safety
    ///
    /// The caller must exclusively own the complete original allocation, with
    /// no live references or other owners. The pointer and capacity must still
    /// be the original pair obtained from the global allocator. No prefix
    /// initialization is required because the reconstructed Vec has length 0
    /// and `u8` has no drop glue.
    pub(crate) unsafe fn deallocate(self) {
        let base = self.base;
        let capacity = self.capacity;
        // SAFETY: The caller promises unique ownership of the original
        // allocation. A zero-length Vec permits deallocation without reading
        // any spare bytes, including when `capacity == 0`.
        unsafe { drop(Vec::from_raw_parts(base, 0, capacity)) };
    }
}
