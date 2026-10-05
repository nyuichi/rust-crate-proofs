use bytes_actual_bufmut_slices::{ClosedMaybeUninitSliceBufMut, ClosedU8SliceBufMut};
use core::mem::MaybeUninit;

#[test]
fn initialized_slice_methods_preserve_chunk_and_advance_the_cursor() {
    let mut backing = [10u8, 20, 30, 40];
    let mut cursor = &mut backing[..];

    assert_eq!(ClosedU8SliceBufMut::remaining_mut(&cursor), 4);
    {
        let chunk = ClosedU8SliceBufMut::chunk_mut(&mut cursor);
        assert_eq!(chunk.len(), 4);
        chunk.write_byte(1, 99);
    }
    unsafe { ClosedU8SliceBufMut::advance_mut(&mut cursor, 2) };
    assert_eq!(ClosedU8SliceBufMut::remaining_mut(&cursor), 2);
    assert_eq!(&*cursor, &[30, 40]);
    assert_eq!(&backing, &[10, 99, 30, 40]);
}

#[test]
fn maybeuninit_slice_methods_keep_the_option_model_and_advance() {
    let mut backing = [MaybeUninit::<u8>::uninit(); 3];
    let mut cursor = &mut backing[..];

    assert_eq!(ClosedMaybeUninitSliceBufMut::remaining_mut(&cursor), 3);
    {
        let chunk = ClosedMaybeUninitSliceBufMut::chunk_mut(&mut cursor);
        assert_eq!(chunk.len(), 3);
        chunk.write_byte(0, 7);
    }
    unsafe { ClosedMaybeUninitSliceBufMut::advance_mut(&mut cursor, 1) };
    assert_eq!(ClosedMaybeUninitSliceBufMut::remaining_mut(&cursor), 2);
    {
        let chunk = ClosedMaybeUninitSliceBufMut::chunk_mut(&mut cursor);
        chunk.write_byte(0, 8);
    }
    unsafe { ClosedMaybeUninitSliceBufMut::advance_mut(&mut cursor, 1) };
    assert_eq!(ClosedMaybeUninitSliceBufMut::remaining_mut(&cursor), 1);

    let initialized = unsafe { [backing[0].assume_init(), backing[1].assume_init()] };
    assert_eq!(initialized, [7, 8]);
}

#[test]
fn empty_slice_methods_have_zero_remaining_and_empty_chunks() {
    let mut initialized: [u8; 0] = [];
    let mut initialized_cursor = &mut initialized[..];
    assert_eq!(ClosedU8SliceBufMut::remaining_mut(&initialized_cursor), 0);
    assert_eq!(
        ClosedU8SliceBufMut::chunk_mut(&mut initialized_cursor).len(),
        0
    );
    unsafe { ClosedU8SliceBufMut::advance_mut(&mut initialized_cursor, 0) };

    let mut uninitialized: [MaybeUninit<u8>; 0] = [];
    let mut uninitialized_cursor = &mut uninitialized[..];
    assert_eq!(
        ClosedMaybeUninitSliceBufMut::remaining_mut(&uninitialized_cursor),
        0
    );
    assert_eq!(
        ClosedMaybeUninitSliceBufMut::chunk_mut(&mut uninitialized_cursor).len(),
        0
    );
    unsafe { ClosedMaybeUninitSliceBufMut::advance_mut(&mut uninitialized_cursor, 0) };
}
