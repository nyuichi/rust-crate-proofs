include!(concat!(env!("OUT_DIR"), "/actual_from_vec.rs"));

#[cfg(creusot)]
#[ensures(result@ == input@.len())]
pub(crate) fn constructor_then_release(input: alloc::vec::Vec<u8>) -> usize {
    let bytes = BytesMut::from_vec(input);
    let len = bytes.len;
    bytes.proof_release_unique_at_zero();
    len
}

#[cfg(all(test, not(creusot)))]
mod native_tests {
    use super::BytesMut;
    use alloc::vec::Vec;
    use core::mem::{align_of, size_of};

    #[test]
    fn actual_from_vec_preserves_allocation_and_bytes() {
        let mut input = Vec::with_capacity(48);
        input.extend_from_slice(b"source-extracted constructor");
        let expected = input.clone();
        let expected_ptr = input.as_mut_ptr();
        let expected_capacity = input.capacity();
        let expected_len = input.len();

        let bytes = BytesMut::from_vec(input);
        assert_eq!(bytes.ptr.as_ptr(), expected_ptr);
        assert_eq!(bytes.len, expected_len);
        assert_eq!(bytes.cap, expected_capacity);

        // This isolated extraction omits the runtime Drop impl. Rebuild the
        // exact Vec allocation explicitly so the native test neither leaks nor
        // claims anything about BytesMut's destructor.
        let recovered = unsafe {
            Vec::from_raw_parts(bytes.ptr.as_ptr(), bytes.len, bytes.cap)
        };
        assert_eq!(recovered.as_ptr(), expected_ptr);
        assert_eq!(recovered.capacity(), expected_capacity);
        assert_eq!(recovered, expected);
    }

    #[test]
    fn native_bytes_mut_layout_remains_four_words() {
        assert_eq!(size_of::<BytesMut>(), 4 * size_of::<usize>());
        assert_eq!(align_of::<BytesMut>(), align_of::<usize>());
    }
}
