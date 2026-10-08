//! Native checks for the generic B1 boxed-slice boundary.
//!
//! These checks exercise only allocation transfer, pointer/content observation,
//! and explicit B3 cleanup. They do not stand in for Creusot proof or Bytes
//! constructor verification.

#[cfg(test)]
mod tests {
    use crate::raw_vec;
    use alloc::{boxed::Box, vec, vec::Vec};
    use core::slice;

    fn inspect_and_deallocate(boxed: Box<[u8]>, expected: &[u8]) {
        // Record the Box's real native slice base before ownership transfer;
        // content equality alone cannot establish allocation identity.
        let original_base = boxed.as_ptr();
        let (raw, len, capabilities) = raw_vec::detach_boxed_slice(boxed);
        assert_eq!(len, expected.len());
        let (bound, capacity) = raw.into_bound_ptr_at_zero();
        assert_eq!(capacity, expected.len());
        let pointer = bound.as_ptr();
        assert_eq!(pointer as *const u8, original_base);
        let observed = unsafe { slice::from_raw_parts(pointer, len) };
        assert_eq!(observed, expected);
        unsafe { raw_vec::deallocate_bound_vec(bound, capacity, capabilities) };
    }

    #[test]
    fn boxed_b1_empty_and_nonempty_keep_exact_base_and_content() {
        inspect_and_deallocate(Vec::new().into_boxed_slice(), &[]);
        inspect_and_deallocate(vec![3, 1, 4, 1, 5].into_boxed_slice(), &[3, 1, 4, 1, 5]);
    }

    #[test]
    fn equal_content_boxes_keep_distinct_live_native_allocations() {
        let first = vec![8, 6, 7, 5, 3, 0, 9].into_boxed_slice();
        let second = vec![8, 6, 7, 5, 3, 0, 9].into_boxed_slice();
        let (raw_first, len_first, capabilities_first) = raw_vec::detach_boxed_slice(first);
        let (raw_second, len_second, capabilities_second) = raw_vec::detach_boxed_slice(second);
        let (bound_first, capacity_first) = raw_first.into_bound_ptr_at_zero();
        let (bound_second, capacity_second) = raw_second.into_bound_ptr_at_zero();
        let first_pointer = bound_first.as_ptr();
        let second_pointer = bound_second.as_ptr();

        // Both capabilities remain live here. Equal slice contents do not
        // identify or merge the two independently allocated Box resources.
        assert_ne!(first_pointer, second_pointer);
        assert_eq!(len_first, len_second);
        assert_eq!(capacity_first, capacity_second);
        assert_eq!(unsafe { slice::from_raw_parts(first_pointer, len_first) }, &[8, 6, 7, 5, 3, 0, 9]);
        assert_eq!(unsafe { slice::from_raw_parts(second_pointer, len_second) }, &[8, 6, 7, 5, 3, 0, 9]);

        unsafe { raw_vec::deallocate_bound_vec(bound_first, capacity_first, capabilities_first) };
        unsafe { raw_vec::deallocate_bound_vec(bound_second, capacity_second, capabilities_second) };
    }

    #[test]
    fn equal_capacity_vec_conversion_reaches_exact_boxed_b1_extent() {
        let mut input = Vec::with_capacity(5);
        input.extend_from_slice(&[2, 7, 1, 8, 2]);
        input.shrink_to_fit();
        assert_eq!(input.capacity(), input.len());

        // This is the native len == cap branch used by From<Vec>: ownership
        // moves through into_boxed_slice without retaining Vec spare capacity.
        inspect_and_deallocate(input.into_boxed_slice(), &[2, 7, 1, 8, 2]);
    }
}
