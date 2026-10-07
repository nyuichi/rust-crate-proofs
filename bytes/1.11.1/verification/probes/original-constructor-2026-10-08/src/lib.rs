#![allow(dead_code, unexpected_cfgs)]
#![recursion_limit = "512"]

extern crate alloc;

#[cfg(creusot)]
use creusot_std::prelude::*;

mod actual_original {
    use alloc::vec::Vec;
    use core::cmp;
    use core::mem::{self, ManuallyDrop};
    use core::ptr::NonNull;
    use core::sync::atomic::AtomicUsize;

    include!(concat!(env!("OUT_DIR"), "/original_constructor.rs"));

    #[cfg(test)]
    mod native_tests {
        use super::*;

        fn recover_vec(owner: BytesMut) -> Vec<u8> {
            let ptr = owner.ptr.as_ptr();
            let len = owner.len;
            let cap = owner.cap;
            // The isolated item set deliberately omits BytesMut::drop. Recover
            // the exact allocation from the extracted native fields instead.
            mem::forget(owner);
            unsafe { Vec::from_raw_parts(ptr, len, cap) }
        }

        #[test]
        fn original_with_capacity_path_keeps_zero_length_and_requested_capacity() {
            for requested in [0, 1, 17, 1024, 65536] {
                let owner = BytesMut::with_capacity(requested);
                assert_eq!(owner.len(), 0);
                assert!(owner.capacity() >= requested);

                let metadata = owner.data as usize;
                let encoded_repr = (metadata & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET;
                assert_eq!(metadata & KIND_MASK, KIND_VEC);
                assert_eq!(encoded_repr, original_capacity_to_repr(owner.cap));

                let recovered = recover_vec(owner);
                assert_eq!(recovered.len(), 0);
                assert!(recovered.capacity() >= requested);
            }
        }

        #[test]
        fn original_from_vec_path_preserves_allocation_length_and_bytes() {
            let mut input = Vec::with_capacity(128);
            input.extend_from_slice(&[0x00, 0xff, 0x31, 0xa5, 0x7e]);
            let expected = input.clone();
            let input_ptr = input.as_mut_ptr();
            let input_cap = input.capacity();
            let input_len = input.len();

            let owner = BytesMut::from_vec(input);
            assert_eq!(owner.ptr.as_ptr(), input_ptr);
            assert_eq!(owner.len(), input_len);
            assert_eq!(owner.capacity(), input_cap);

            let recovered = recover_vec(owner);
            assert_eq!(recovered, expected);
            assert_eq!(recovered.capacity(), input_cap);
        }
    }
}

pub use actual_original::BytesMut;
