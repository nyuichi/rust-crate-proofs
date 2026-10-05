//! Native physical deallocation shared by ordinary cleanup and the B3 bridge.
//! This helper has no ownership contract: verified callers enter through B3.

/// Release a complete global-allocator byte allocation.
///
/// # Safety
/// For nonzero capacity, `base` must be the original live allocation base with
/// exactly this capacity and `u8` layout. The caller must own its full recovery
/// authority, with no outstanding accesses. Zero capacity has no allocation.
pub(crate) unsafe fn deallocate_u8(base: *mut u8, capacity: usize) {
    if capacity != 0 {
        // Vec<u8> allocations use size=capacity, alignment=1, and never exceed
        // isize::MAX. Spare bytes need not be initialized for deallocation.
        let layout = unsafe { alloc::alloc::Layout::from_size_align_unchecked(capacity, 1) };
        unsafe { alloc::alloc::dealloc(base, layout); }
    }
}
