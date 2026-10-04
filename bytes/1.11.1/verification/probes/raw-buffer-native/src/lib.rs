//! Native tests for the raw allocation bridge. These do not test Creusot
//! permissions or prove any Bytes/BytesMut integration.

extern crate alloc;

#[path = "../../../../src/ownership_proof/raw_buffer.rs"]
#[allow(dead_code)] // The isolated harness consumes these APIs only under cfg(test).
mod raw_buffer;

#[cfg(test)]
mod tests {
    use super::raw_buffer;

    #[test]
    fn detach_and_recover_initialized_prefix_with_spare_capacity() {
        let mut input = Vec::with_capacity(32);
        input.extend_from_slice(b"raw buffer");
        let expected_capacity = input.capacity();
        let expected_pointer = input.as_ptr();

        let (raw, len) = raw_buffer::detach(input);
        assert_eq!(len, b"raw buffer".len());
        assert_eq!(raw.capacity(), expected_capacity);
        assert_eq!(raw.base_ptr(), expected_pointer.cast_mut());

        // SAFETY: This test alone retains the detached allocation, uses the
        // original initialized prefix length, and has no other live aliases.
        let recovered = unsafe { raw.into_vec(len) };
        assert_eq!(recovered.as_ptr(), expected_pointer);
        assert_eq!(recovered, b"raw buffer");
    }

    #[test]
    fn mutate_nonempty_raw_regions_then_recover_same_vec() {
        let mut input = Vec::with_capacity(24);
        input.extend_from_slice(b"abcdef");
        let old_capacity = input.capacity();
        let old_pointer = input.as_ptr();
        let split = 3;

        let (raw, len) = raw_buffer::detach(input);
        let base = raw.base_ptr();
        // SAFETY: The detached allocation is uniquely owned here, both slices
        // cover disjoint parts of its initialized prefix, and remain scoped
        // before ownership is rebuilt.
        unsafe {
            let left = core::slice::from_raw_parts_mut(base, split);
            let right = core::slice::from_raw_parts_mut(base.add(split), len - split);
            left[split - 1] = b'X';
            right[0] = b'Y';
        }

        // SAFETY: All aliases above ended; the original initialized prefix is
        // still fully initialized and the complete allocation remains unique.
        let recovered = unsafe { raw.into_vec(len) };
        assert_eq!(recovered.as_ptr(), old_pointer);
        assert_eq!(recovered.capacity(), old_capacity);
        assert_eq!(recovered, b"abXYef");
    }

    #[test]
    fn empty_zero_capacity_vec_can_be_recovered_and_deallocated() {
        let input = Vec::new();
        assert_eq!(input.capacity(), 0);
        let (raw, len) = raw_buffer::detach(input);
        assert_eq!(len, 0);
        assert_eq!(raw.capacity(), 0);
        // SAFETY: The empty Vec's zero-capacity dangling pointer is the exact
        // pair detached above and no aliases exist.
        let recovered = unsafe { raw.into_vec(0) };
        assert!(recovered.is_empty());

        let (raw, len) = raw_buffer::detach(Vec::new());
        assert_eq!(len, 0);
        // SAFETY: This independently detached empty allocation has unique
        // ownership and the zero-capacity Vec pointer is valid for rebuilding.
        unsafe { raw.deallocate() };
    }

    #[test]
    fn zero_length_deallocation_ignores_written_spare_bytes() {
        let input = Vec::<u8>::with_capacity(16);
        let capacity = input.capacity();
        assert!(capacity >= 2);
        let (raw, len) = raw_buffer::detach(input);
        assert_eq!(len, 0);
        assert_eq!(raw.capacity(), capacity);

        // SAFETY: These writes target allocated spare capacity, do not create
        // references to uninitialized bytes, and the detached allocation is
        // uniquely owned. Rebuilding at length zero must not read the writes.
        unsafe {
            raw.base_ptr().write(0xA5);
            raw.base_ptr().add(1).write(0x5A);
        }

        // SAFETY: The same unique original allocation is consumed here; the
        // zero-length reconstruction does not drop or read the spare writes.
        unsafe { raw.deallocate() };
    }

    #[test]
    fn zero_length_deallocation_after_uninitializing_former_prefix() {
        let mut input = Vec::with_capacity(16);
        input.extend_from_slice(b"known!");
        let initialized_len = input.len();
        let expected_capacity = input.capacity();
        let (raw, len) = raw_buffer::detach(input);
        assert_eq!(len, initialized_len);
        assert_eq!(raw.capacity(), expected_capacity);

        // SAFETY: The allocation remains uniquely owned, the target is within
        // its former initialized prefix, and MaybeUninit<u8> has the same
        // layout/alignment as u8. This overwrites without reading the byte.
        unsafe {
            raw.base_ptr()
                .add(2)
                .cast::<core::mem::MaybeUninit<u8>>()
                .write(core::mem::MaybeUninit::uninit());
        }

        // SAFETY: The unique allocation is consumed at Vec length zero. The
        // former prefix is not resumed or read, and u8 has no drop glue.
        unsafe { raw.deallocate() };
    }
}
