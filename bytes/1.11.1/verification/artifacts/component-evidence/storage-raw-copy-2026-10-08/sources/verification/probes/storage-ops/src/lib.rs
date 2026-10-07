//! Standalone proof and native tests for prefix writes into uninitialized storage.
#![allow(unexpected_cfgs)]

use core::mem::MaybeUninit;
use creusot_std::prelude::*;

#[path = "../../../../src/storage_ops.rs"]
mod storage_ops;

#[requires(count@ <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < count@ ==> (^dst)@[i]@ == Some(value))]
#[ensures(forall<i> count@ <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
pub fn fill_prefix(dst: &mut [MaybeUninit<u8>], count: usize, value: u8) {
    storage_ops::fill_uninit_prefix(dst, count, value)
}

#[requires(src@.len() <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < src@.len() ==> (^dst)@[i]@ == Some(src@[i]))]
#[ensures(forall<i> src@.len() <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
pub fn copy_prefix(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    storage_ops::copy_to_uninit_prefix(dst, src)
}

/// Caller proof conditional on the local generic memcpy-effect contract.
/// This does not prove the raw-copy leaf or a BytesMut ownership invariant.
#[requires(src@.len() <= dst@.len())]
#[ensures((^dst)@.len() == dst@.len())]
#[ensures(forall<i> 0 <= i && i < src@.len() ==> (^dst)@[i]@ == Some(src@[i]))]
#[ensures(forall<i> src@.len() <= i && i < dst@.len() ==> (^dst)@[i]@ == dst@[i]@)]
pub fn copy_prefix_raw(dst: &mut [MaybeUninit<u8>], src: &[u8]) {
    unsafe { storage_ops::copy_to_uninit_prefix_raw(dst, src) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialized_values(bytes: &[MaybeUninit<u8>]) -> Vec<u8> {
        bytes
            .iter()
            .map(|byte| unsafe { byte.assume_init() })
            .collect()
    }

    #[test]
    fn fill_initializes_only_the_requested_prefix() {
        let mut storage = [MaybeUninit::new(0xa5); 5];
        fill_prefix(&mut storage, 3, 0x3c);
        assert_eq!(initialized_values(&storage), [0x3c, 0x3c, 0x3c, 0xa5, 0xa5]);
    }

    #[test]
    fn fill_supports_empty_and_full_prefixes() {
        let mut empty: [MaybeUninit<u8>; 0] = [];
        fill_prefix(&mut empty, 0, 0x3c);
        assert_eq!(empty.len(), 0);

        let mut storage = [MaybeUninit::uninit(); 3];
        fill_prefix(&mut storage, 3, 0x71);
        assert_eq!(initialized_values(&storage), [0x71; 3]);
    }

    #[test]
    fn copy_initializes_only_the_source_length_prefix() {
        let mut storage = [MaybeUninit::new(0xa5); 5];
        copy_prefix(&mut storage, &[4, 8, 15]);
        assert_eq!(initialized_values(&storage), [4, 8, 15, 0xa5, 0xa5]);
    }

    #[test]
    fn copy_supports_empty_and_full_prefixes() {
        let mut storage = [MaybeUninit::new(0x22); 3];
        copy_prefix(&mut storage, &[]);
        assert_eq!(initialized_values(&storage), [0x22; 3]);
        copy_prefix(&mut storage, &[7, 9, 11]);
        assert_eq!(initialized_values(&storage), [7, 9, 11]);

        let mut uninitialized = [MaybeUninit::uninit(); 3];
        copy_prefix(&mut uninitialized, &[1, 2, 3]);
        assert_eq!(initialized_values(&uninitialized), [1, 2, 3]);
    }

    #[test]
    fn raw_copy_matches_initialized_prefix_and_frames_suffix() {
        for count in 0..=5 {
            let input = [1, 2, 3, 4, 5];
            let mut raw = [MaybeUninit::new(0xa5); 5];
            let mut reference = [MaybeUninit::new(0xa5); 5];
            copy_prefix_raw(&mut raw, &input[..count]);
            copy_prefix(&mut reference, &input[..count]);
            assert_eq!(initialized_values(&raw), initialized_values(&reference));
        }
        let mut uninitialized = [MaybeUninit::uninit(); 3];
        copy_prefix_raw(&mut uninitialized, &[7, 8, 9]);
        assert_eq!(initialized_values(&uninitialized), [7, 8, 9]);
        copy_prefix_raw(&mut [], &[]);
    }
}
