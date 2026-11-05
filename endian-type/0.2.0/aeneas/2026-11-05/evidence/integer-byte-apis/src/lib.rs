#[cfg(not(all(target_arch="x86_64",target_endian="little",target_pointer_width="64")))]
compile_error!("These integer-byte checks target x86_64 little-endian with 64-bit pointers");
#[cfg(kani)]
mod checks {
    use endian_type::{BigEndian,LittleEndian};
    #[kani::proof]
    fn integer_byte_apis_u16() {
        let raw: [u8; 2] = kani::any();
        let value: u16 = kani::any();
        assert_eq!(BigEndian::<u16>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<u16>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(u16::from(BigEndian::<u16>::from_bytes(raw)), u16::from_be_bytes(raw));
        assert_eq!(u16::from(LittleEndian::<u16>::from_bytes(raw)), u16::from_le_bytes(raw));
        assert_eq!(BigEndian::<u16>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<u16>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(u16::from(BigEndian::<u16>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(u16::from(LittleEndian::<u16>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_u32() {
        let raw: [u8; 4] = kani::any();
        let value: u32 = kani::any();
        assert_eq!(BigEndian::<u32>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<u32>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(u32::from(BigEndian::<u32>::from_bytes(raw)), u32::from_be_bytes(raw));
        assert_eq!(u32::from(LittleEndian::<u32>::from_bytes(raw)), u32::from_le_bytes(raw));
        assert_eq!(BigEndian::<u32>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<u32>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(u32::from(BigEndian::<u32>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(u32::from(LittleEndian::<u32>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_u64() {
        let raw: [u8; 8] = kani::any();
        let value: u64 = kani::any();
        assert_eq!(BigEndian::<u64>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<u64>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(u64::from(BigEndian::<u64>::from_bytes(raw)), u64::from_be_bytes(raw));
        assert_eq!(u64::from(LittleEndian::<u64>::from_bytes(raw)), u64::from_le_bytes(raw));
        assert_eq!(BigEndian::<u64>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<u64>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(u64::from(BigEndian::<u64>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(u64::from(LittleEndian::<u64>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_u128() {
        let raw: [u8; 16] = kani::any();
        let value: u128 = kani::any();
        assert_eq!(BigEndian::<u128>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<u128>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(u128::from(BigEndian::<u128>::from_bytes(raw)), u128::from_be_bytes(raw));
        assert_eq!(u128::from(LittleEndian::<u128>::from_bytes(raw)), u128::from_le_bytes(raw));
        assert_eq!(BigEndian::<u128>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<u128>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(u128::from(BigEndian::<u128>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(u128::from(LittleEndian::<u128>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_usize() {
        let raw: [u8; 8] = kani::any();
        let value: usize = kani::any();
        assert_eq!(BigEndian::<usize>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<usize>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(usize::from(BigEndian::<usize>::from_bytes(raw)), usize::from_be_bytes(raw));
        assert_eq!(usize::from(LittleEndian::<usize>::from_bytes(raw)), usize::from_le_bytes(raw));
        assert_eq!(BigEndian::<usize>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<usize>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(usize::from(BigEndian::<usize>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(usize::from(LittleEndian::<usize>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_i16() {
        let raw: [u8; 2] = kani::any();
        let value: i16 = kani::any();
        assert_eq!(BigEndian::<i16>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<i16>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(i16::from(BigEndian::<i16>::from_bytes(raw)), i16::from_be_bytes(raw));
        assert_eq!(i16::from(LittleEndian::<i16>::from_bytes(raw)), i16::from_le_bytes(raw));
        assert_eq!(BigEndian::<i16>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<i16>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(i16::from(BigEndian::<i16>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(i16::from(LittleEndian::<i16>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_i32() {
        let raw: [u8; 4] = kani::any();
        let value: i32 = kani::any();
        assert_eq!(BigEndian::<i32>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<i32>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(i32::from(BigEndian::<i32>::from_bytes(raw)), i32::from_be_bytes(raw));
        assert_eq!(i32::from(LittleEndian::<i32>::from_bytes(raw)), i32::from_le_bytes(raw));
        assert_eq!(BigEndian::<i32>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<i32>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(i32::from(BigEndian::<i32>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(i32::from(LittleEndian::<i32>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_i64() {
        let raw: [u8; 8] = kani::any();
        let value: i64 = kani::any();
        assert_eq!(BigEndian::<i64>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<i64>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(i64::from(BigEndian::<i64>::from_bytes(raw)), i64::from_be_bytes(raw));
        assert_eq!(i64::from(LittleEndian::<i64>::from_bytes(raw)), i64::from_le_bytes(raw));
        assert_eq!(BigEndian::<i64>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<i64>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(i64::from(BigEndian::<i64>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(i64::from(LittleEndian::<i64>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_i128() {
        let raw: [u8; 16] = kani::any();
        let value: i128 = kani::any();
        assert_eq!(BigEndian::<i128>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<i128>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(i128::from(BigEndian::<i128>::from_bytes(raw)), i128::from_be_bytes(raw));
        assert_eq!(i128::from(LittleEndian::<i128>::from_bytes(raw)), i128::from_le_bytes(raw));
        assert_eq!(BigEndian::<i128>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<i128>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(i128::from(BigEndian::<i128>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(i128::from(LittleEndian::<i128>::from_bytes(value.to_le_bytes())), value);
    }
    #[kani::proof]
    fn integer_byte_apis_isize() {
        let raw: [u8; 8] = kani::any();
        let value: isize = kani::any();
        assert_eq!(BigEndian::<isize>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(LittleEndian::<isize>::from_bytes(raw).to_bytes(), raw);
        assert_eq!(isize::from(BigEndian::<isize>::from_bytes(raw)), isize::from_be_bytes(raw));
        assert_eq!(isize::from(LittleEndian::<isize>::from_bytes(raw)), isize::from_le_bytes(raw));
        assert_eq!(BigEndian::<isize>::from(value).to_bytes(), value.to_be_bytes());
        assert_eq!(LittleEndian::<isize>::from(value).to_bytes(), value.to_le_bytes());
        assert_eq!(isize::from(BigEndian::<isize>::from_bytes(value.to_be_bytes())), value);
        assert_eq!(isize::from(LittleEndian::<isize>::from_bytes(value.to_le_bytes())), value);
    }
}
