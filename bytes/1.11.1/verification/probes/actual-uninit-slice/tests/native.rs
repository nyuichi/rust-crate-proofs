use bytes_actual_uninit_slice::{native_write_and_copy, UninitSlice};
use core::mem::MaybeUninit;

#[test]
fn exact_initialized_conversion_write_and_copy_mutate_the_source_borrow() {
    let mut bytes = [0u8; 5];
    native_write_and_copy(&mut bytes, b"hello", 2, b'X');
    assert_eq!(&bytes, b"hello");

    let mut bytes = [0u8; 5];
    {
        let destination: &mut UninitSlice = (&mut bytes[..]).into();
        destination.write_byte(1, b'Y');
        assert_eq!(destination.len(), 5);
    }
    assert_eq!(&bytes, &[0, b'Y', 0, 0, 0]);
}

#[test]
fn exact_maybeuninit_conversion_initializes_each_copied_slot() {
    let mut slots = [MaybeUninit::<u8>::uninit(); 4];
    {
        let destination: &mut UninitSlice = (&mut slots[..]).into();
        destination.copy_from_slice(b"data");
    }

    let initialized = unsafe {
        [
            slots[0].assume_init(),
            slots[1].assume_init(),
            slots[2].assume_init(),
            slots[3].assume_init(),
        ]
    };
    assert_eq!(&initialized, b"data");
}

#[test]
fn empty_exact_conversions_preserve_zero_length() {
    let mut bytes = [];
    let destination = UninitSlice::new(&mut bytes);
    destination.copy_from_slice(&[]);
    assert_eq!(destination.len(), 0);

    let mut slots: [MaybeUninit<u8>; 0] = [];
    let destination: &mut UninitSlice = (&mut slots[..]).into();
    assert_eq!(destination.len(), 0);
}
