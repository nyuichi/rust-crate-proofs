include!(concat!(env!("OUT_DIR"), "/actual_public_constructors.rs"));

#[cfg(creusot)]
#[ensures(result@ == len@)]
pub(crate) fn zeroed_then_release(len: usize) -> usize {
    let owner = BytesMut::zeroed(len);
    let result = owner.len;
    owner.proof_release_unique_at_zero();
    result
}

#[cfg(creusot)]
#[ensures(result@ == src@.len())]
pub(crate) fn from_slice_then_release(src: &[u8]) -> usize {
    let owner = BytesMut::from(src);
    let result = owner.len;
    owner.proof_release_unique_at_zero();
    result
}

#[cfg(creusot)]
pub(crate) fn generic_from_slice<'a, T: From<&'a [u8]>>(src: &'a [u8]) -> T {
    T::from(src)
}

#[cfg(all(test, not(creusot)))]
mod native_tests {
    use super::BytesMut;
    use alloc::vec::Vec;
    use core::mem;

    fn recover_vec(owner: BytesMut) -> Vec<u8> {
        let ptr = owner.ptr.as_ptr();
        let len = owner.len;
        let cap = owner.cap;
        mem::forget(owner);
        // The actual BytesMut Drop implementation is omitted from this isolated
        // extraction. Rebuild the exact Vec allocation to inspect and free it.
        unsafe { Vec::from_raw_parts(ptr, len, cap) }
    }

    #[test]
    fn zeroed_preserves_length_and_initializes_every_byte() {
        for len in [0, 1, 19] {
            let owner = BytesMut::zeroed(len);
            let recovered = recover_vec(owner);
            assert_eq!(recovered.len(), len);
            assert!(recovered.iter().all(|byte| *byte == 0));
        }
    }

    #[test]
    fn from_slice_copies_empty_and_arbitrary_bytes() {
        for source in [&[][..], &[0, 0xff, 0x40, 0x7f][..]] {
            let owner = BytesMut::from(source);
            let recovered = recover_vec(owner);
            assert_eq!(recovered.as_slice(), source);
        }
    }

    #[test]
    fn from_slice_owns_an_independent_copy() {
        let mut source = [0x00, 0xff, 0x31, 0xa5];
        let expected = source;
        let owner = BytesMut::from(&source[..]);
        source.fill(0x55);

        let recovered = recover_vec(owner);
        assert_eq!(source, [0x55; 4]);
        assert_eq!(recovered, expected);
    }
}
