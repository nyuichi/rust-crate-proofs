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

/// Resize a global byte allocation, preserving its old bytes on growth.
///
/// # Safety
/// A nonzero old capacity names the complete live allocation at `base` with
/// byte layout. The caller has exclusive full-allocation ownership. The new
/// capacity is positive and no greater than isize::MAX. OOM does not return.
pub(crate) unsafe fn reallocate_u8(base: *mut u8, old_capacity: usize, capacity: usize) -> core::ptr::NonNull<u8> {
    use alloc::alloc::{alloc, handle_alloc_error, realloc, Layout};
    let layout = Layout::array::<u8>(capacity).expect("capacity overflow");
    assert!(capacity > 0);
    let pointer = if old_capacity == 0 {
        unsafe { alloc(layout) }
    } else {
        let old_layout = unsafe { Layout::from_size_align_unchecked(old_capacity, 1) };
        unsafe { realloc(base, old_layout, capacity) }
    };
    match core::ptr::NonNull::new(pointer) {
        Some(pointer) => pointer,
        None => handle_alloc_error(layout),
    }
}
