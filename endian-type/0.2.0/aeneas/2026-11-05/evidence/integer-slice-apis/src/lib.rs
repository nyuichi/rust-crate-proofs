#[cfg(not(all(target_arch="x86_64",target_endian="little",target_pointer_width="64")))]
compile_error!("These checks target x86_64 little-endian with 64-bit pointers");
#[cfg(kani)]
mod checks {
    use endian_type::{BigEndian,LittleEndian};
    // C layout places the sole initialized array field at offset zero.
    // Overalignment supports every selected integer; tail padding is not read.
    #[repr(C,align(16))]
    struct Aligned<const N:usize>([u8;N]);
    #[kani::proof]
    fn u16_valid_integer_slice_apis() {
        let value: u16 = kani::any();
        let replacement: [u8;2] = kani::any();
        let raw: [u8;2] = kani::any();
        let mut be=BigEndian::<u16>::from(value);
        assert_eq!(be.as_byte_slice().len(),2);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),2);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(u16::from(be),u16::from_be_bytes(replacement));
        let mut le=LittleEndian::<u16>::from(value);
        assert_eq!(le.as_byte_slice().len(),2);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),2);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(u16::from(le),u16::from_le_bytes(replacement));
        let buffer=Aligned::<2>(raw);
        assert!(core::mem::align_of::<Aligned<2>>() >= core::mem::align_of::<u16>());
        assert_eq!(core::mem::size_of::<u16>(),2);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<u16>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(u16::from(be_read),u16::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<u16>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(u16::from(le_read),u16::from_le_bytes(raw));
    }
    #[kani::proof]
    fn u32_valid_integer_slice_apis() {
        let value: u32 = kani::any();
        let replacement: [u8;4] = kani::any();
        let raw: [u8;4] = kani::any();
        let mut be=BigEndian::<u32>::from(value);
        assert_eq!(be.as_byte_slice().len(),4);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),4);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(u32::from(be),u32::from_be_bytes(replacement));
        let mut le=LittleEndian::<u32>::from(value);
        assert_eq!(le.as_byte_slice().len(),4);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),4);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(u32::from(le),u32::from_le_bytes(replacement));
        let buffer=Aligned::<4>(raw);
        assert!(core::mem::align_of::<Aligned<4>>() >= core::mem::align_of::<u32>());
        assert_eq!(core::mem::size_of::<u32>(),4);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<u32>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(u32::from(be_read),u32::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<u32>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(u32::from(le_read),u32::from_le_bytes(raw));
    }
    #[kani::proof]
    fn u64_valid_integer_slice_apis() {
        let value: u64 = kani::any();
        let replacement: [u8;8] = kani::any();
        let raw: [u8;8] = kani::any();
        let mut be=BigEndian::<u64>::from(value);
        assert_eq!(be.as_byte_slice().len(),8);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(u64::from(be),u64::from_be_bytes(replacement));
        let mut le=LittleEndian::<u64>::from(value);
        assert_eq!(le.as_byte_slice().len(),8);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(u64::from(le),u64::from_le_bytes(replacement));
        let buffer=Aligned::<8>(raw);
        assert!(core::mem::align_of::<Aligned<8>>() >= core::mem::align_of::<u64>());
        assert_eq!(core::mem::size_of::<u64>(),8);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<u64>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(u64::from(be_read),u64::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<u64>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(u64::from(le_read),u64::from_le_bytes(raw));
    }
    #[kani::proof]
    fn u128_valid_integer_slice_apis() {
        let value: u128 = kani::any();
        let replacement: [u8;16] = kani::any();
        let raw: [u8;16] = kani::any();
        let mut be=BigEndian::<u128>::from(value);
        assert_eq!(be.as_byte_slice().len(),16);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),16);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(u128::from(be),u128::from_be_bytes(replacement));
        let mut le=LittleEndian::<u128>::from(value);
        assert_eq!(le.as_byte_slice().len(),16);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),16);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(u128::from(le),u128::from_le_bytes(replacement));
        let buffer=Aligned::<16>(raw);
        assert!(core::mem::align_of::<Aligned<16>>() >= core::mem::align_of::<u128>());
        assert_eq!(core::mem::size_of::<u128>(),16);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<u128>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(u128::from(be_read),u128::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<u128>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(u128::from(le_read),u128::from_le_bytes(raw));
    }
    #[kani::proof]
    fn usize_valid_integer_slice_apis() {
        let value: usize = kani::any();
        let replacement: [u8;8] = kani::any();
        let raw: [u8;8] = kani::any();
        let mut be=BigEndian::<usize>::from(value);
        assert_eq!(be.as_byte_slice().len(),8);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(usize::from(be),usize::from_be_bytes(replacement));
        let mut le=LittleEndian::<usize>::from(value);
        assert_eq!(le.as_byte_slice().len(),8);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(usize::from(le),usize::from_le_bytes(replacement));
        let buffer=Aligned::<8>(raw);
        assert!(core::mem::align_of::<Aligned<8>>() >= core::mem::align_of::<usize>());
        assert_eq!(core::mem::size_of::<usize>(),8);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<usize>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(usize::from(be_read),usize::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<usize>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(usize::from(le_read),usize::from_le_bytes(raw));
    }
    #[kani::proof]
    fn i16_valid_integer_slice_apis() {
        let value: i16 = kani::any();
        let replacement: [u8;2] = kani::any();
        let raw: [u8;2] = kani::any();
        let mut be=BigEndian::<i16>::from(value);
        assert_eq!(be.as_byte_slice().len(),2);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),2);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(i16::from(be),i16::from_be_bytes(replacement));
        let mut le=LittleEndian::<i16>::from(value);
        assert_eq!(le.as_byte_slice().len(),2);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),2);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(i16::from(le),i16::from_le_bytes(replacement));
        let buffer=Aligned::<2>(raw);
        assert!(core::mem::align_of::<Aligned<2>>() >= core::mem::align_of::<i16>());
        assert_eq!(core::mem::size_of::<i16>(),2);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<i16>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(i16::from(be_read),i16::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<i16>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(i16::from(le_read),i16::from_le_bytes(raw));
    }
    #[kani::proof]
    fn i32_valid_integer_slice_apis() {
        let value: i32 = kani::any();
        let replacement: [u8;4] = kani::any();
        let raw: [u8;4] = kani::any();
        let mut be=BigEndian::<i32>::from(value);
        assert_eq!(be.as_byte_slice().len(),4);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),4);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(i32::from(be),i32::from_be_bytes(replacement));
        let mut le=LittleEndian::<i32>::from(value);
        assert_eq!(le.as_byte_slice().len(),4);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),4);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(i32::from(le),i32::from_le_bytes(replacement));
        let buffer=Aligned::<4>(raw);
        assert!(core::mem::align_of::<Aligned<4>>() >= core::mem::align_of::<i32>());
        assert_eq!(core::mem::size_of::<i32>(),4);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<i32>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(i32::from(be_read),i32::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<i32>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(i32::from(le_read),i32::from_le_bytes(raw));
    }
    #[kani::proof]
    fn i64_valid_integer_slice_apis() {
        let value: i64 = kani::any();
        let replacement: [u8;8] = kani::any();
        let raw: [u8;8] = kani::any();
        let mut be=BigEndian::<i64>::from(value);
        assert_eq!(be.as_byte_slice().len(),8);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(i64::from(be),i64::from_be_bytes(replacement));
        let mut le=LittleEndian::<i64>::from(value);
        assert_eq!(le.as_byte_slice().len(),8);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(i64::from(le),i64::from_le_bytes(replacement));
        let buffer=Aligned::<8>(raw);
        assert!(core::mem::align_of::<Aligned<8>>() >= core::mem::align_of::<i64>());
        assert_eq!(core::mem::size_of::<i64>(),8);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<i64>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(i64::from(be_read),i64::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<i64>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(i64::from(le_read),i64::from_le_bytes(raw));
    }
    #[kani::proof]
    fn i128_valid_integer_slice_apis() {
        let value: i128 = kani::any();
        let replacement: [u8;16] = kani::any();
        let raw: [u8;16] = kani::any();
        let mut be=BigEndian::<i128>::from(value);
        assert_eq!(be.as_byte_slice().len(),16);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),16);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(i128::from(be),i128::from_be_bytes(replacement));
        let mut le=LittleEndian::<i128>::from(value);
        assert_eq!(le.as_byte_slice().len(),16);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),16);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(i128::from(le),i128::from_le_bytes(replacement));
        let buffer=Aligned::<16>(raw);
        assert!(core::mem::align_of::<Aligned<16>>() >= core::mem::align_of::<i128>());
        assert_eq!(core::mem::size_of::<i128>(),16);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<i128>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(i128::from(be_read),i128::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<i128>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(i128::from(le_read),i128::from_le_bytes(raw));
    }
    #[kani::proof]
    fn isize_valid_integer_slice_apis() {
        let value: isize = kani::any();
        let replacement: [u8;8] = kani::any();
        let raw: [u8;8] = kani::any();
        let mut be=BigEndian::<isize>::from(value);
        assert_eq!(be.as_byte_slice().len(),8);
        assert_eq!(be.as_byte_slice(),&value.to_be_bytes());
        {
            let bytes=be.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_be_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(be.to_bytes(),replacement);
        assert_eq!(isize::from(be),isize::from_be_bytes(replacement));
        let mut le=LittleEndian::<isize>::from(value);
        assert_eq!(le.as_byte_slice().len(),8);
        assert_eq!(le.as_byte_slice(),&value.to_le_bytes());
        {
            let bytes=le.as_byte_slice_mut();
            assert_eq!(bytes.len(),8);
            assert_eq!(bytes,&value.to_le_bytes());
            bytes.copy_from_slice(&replacement);
        }
        assert_eq!(le.to_bytes(),replacement);
        assert_eq!(isize::from(le),isize::from_le_bytes(replacement));
        let buffer=Aligned::<8>(raw);
        assert!(core::mem::align_of::<Aligned<8>>() >= core::mem::align_of::<isize>());
        assert_eq!(core::mem::size_of::<isize>(),8);
        // Valid unsafe reads: live, initialized, exact-length, aligned storage;
        // every representation of the selected primitive integer is valid.
        let be_read=unsafe {BigEndian::<isize>::from_byte_slice(&buffer.0)};
        assert_eq!(be_read.to_bytes(),raw);
        assert_eq!(isize::from(be_read),isize::from_be_bytes(raw));
        let le_read=unsafe {LittleEndian::<isize>::from_byte_slice(&buffer.0)};
        assert_eq!(le_read.to_bytes(),raw);
        assert_eq!(isize::from(le_read),isize::from_le_bytes(raw));
    }
}
