#[cfg(not(all(target_arch = "x86_64", target_endian = "little", target_pointer_width = "64")))]
compile_error!("These target-specific model checks require x86_64 little-endian with 64-bit pointers");
#[cfg(kani)]
mod checks {
    #[kani::proof]
    fn std_u16_endian_models() {
        let x: u16 = kani::any();
        let bits: u16 = x as u16;
        let le = bits.to_le_bytes();
        let reversed = [le[1], le[0]];
        let swap_reference = u16::from_le_bytes(reversed) as u16;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(u16::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(u16::from_le(x), x);
        assert_eq!(u16::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_u32_endian_models() {
        let x: u32 = kani::any();
        let bits: u32 = x as u32;
        let le = bits.to_le_bytes();
        let reversed = [le[3], le[2], le[1], le[0]];
        let swap_reference = u32::from_le_bytes(reversed) as u32;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(u32::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(u32::from_le(x), x);
        assert_eq!(u32::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_u64_endian_models() {
        let x: u64 = kani::any();
        let bits: u64 = x as u64;
        let le = bits.to_le_bytes();
        let reversed = [le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = u64::from_le_bytes(reversed) as u64;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(u64::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(u64::from_le(x), x);
        assert_eq!(u64::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_u128_endian_models() {
        let x: u128 = kani::any();
        let bits: u128 = x as u128;
        let le = bits.to_le_bytes();
        let reversed = [le[15], le[14], le[13], le[12], le[11], le[10], le[9], le[8], le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = u128::from_le_bytes(reversed) as u128;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(u128::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(u128::from_le(x), x);
        assert_eq!(u128::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_usize_endian_models() {
        let x: usize = kani::any();
        let bits: usize = x as usize;
        let le = bits.to_le_bytes();
        let reversed = [le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = usize::from_le_bytes(reversed) as usize;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(usize::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(usize::from_le(x), x);
        assert_eq!(usize::from_le_bytes(x.to_be_bytes()), swap_reference);
        assert_eq!(core::mem::size_of::<usize>(), 8);
        assert_eq!((u64::from_le_bytes((x as u64).to_be_bytes())) as usize, swap_reference);
    }
    #[kani::proof]
    fn std_i16_endian_models() {
        let x: i16 = kani::any();
        let bits: u16 = x as u16;
        let le = bits.to_le_bytes();
        let reversed = [le[1], le[0]];
        let swap_reference = u16::from_le_bytes(reversed) as i16;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(i16::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(i16::from_le(x), x);
        assert_eq!(i16::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_i32_endian_models() {
        let x: i32 = kani::any();
        let bits: u32 = x as u32;
        let le = bits.to_le_bytes();
        let reversed = [le[3], le[2], le[1], le[0]];
        let swap_reference = u32::from_le_bytes(reversed) as i32;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(i32::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(i32::from_le(x), x);
        assert_eq!(i32::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_i64_endian_models() {
        let x: i64 = kani::any();
        let bits: u64 = x as u64;
        let le = bits.to_le_bytes();
        let reversed = [le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = u64::from_le_bytes(reversed) as i64;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(i64::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(i64::from_le(x), x);
        assert_eq!(i64::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_i128_endian_models() {
        let x: i128 = kani::any();
        let bits: u128 = x as u128;
        let le = bits.to_le_bytes();
        let reversed = [le[15], le[14], le[13], le[12], le[11], le[10], le[9], le[8], le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = u128::from_le_bytes(reversed) as i128;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(i128::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(i128::from_le(x), x);
        assert_eq!(i128::from_le_bytes(x.to_be_bytes()), swap_reference);
    }
    #[kani::proof]
    fn std_isize_endian_models() {
        let x: isize = kani::any();
        let bits: usize = x as usize;
        let le = bits.to_le_bytes();
        let reversed = [le[7], le[6], le[5], le[4], le[3], le[2], le[1], le[0]];
        let swap_reference = usize::from_le_bytes(reversed) as isize;
        assert_eq!(x.swap_bytes(), swap_reference);
        assert_eq!(x.to_be(), swap_reference);
        assert_eq!(isize::from_be(x), swap_reference);
        assert_eq!(x.to_le(), x);
        assert_eq!(isize::from_le(x), x);
        assert_eq!(isize::from_le_bytes(x.to_be_bytes()), swap_reference);
        assert_eq!(core::mem::size_of::<isize>(), 8);
        assert_eq!((i64::from_le_bytes((x as i64).to_be_bytes())) as isize, swap_reference);
    }
}
