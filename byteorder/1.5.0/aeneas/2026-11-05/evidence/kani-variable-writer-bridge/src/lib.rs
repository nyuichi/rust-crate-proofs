//! Bounded-shape supplement: normal-success comparison of two original public APIs.
#[cfg(kani)]
mod checks {
    #[kani::proof]
    #[kani::unwind(34)]
    fn be_write_uint_original_bridge() {
        let value: u64 = kani::any();
        let nbytes: usize = kani::any();
        let slice_len: usize = kani::any();
        let initial: [u8; 32] = kani::any();
        kani::assume(1 <= nbytes && nbytes <= 8);
        kani::assume(nbytes <= slice_len && slice_len <= 32);
        if nbytes < 8 { kani::assume(value >> (8 * nbytes) == 0); }
        let mut original = initial;
        let mut safe = initial;
        <byteorder::BigEndian as byteorder::ByteOrder>::write_uint(&mut original[..slice_len], value, nbytes);
        <byteorder_lite::BigEndian as byteorder_lite::ByteOrder>::write_uint(&mut safe[..slice_len], value, nbytes);
        assert_eq!(original, safe);
        for i in nbytes..32 { assert_eq!(original[i], initial[i]); }
    }
    #[kani::proof]
    #[kani::unwind(34)]
    fn le_write_uint_original_bridge() {
        let value: u64 = kani::any();
        let nbytes: usize = kani::any();
        let slice_len: usize = kani::any();
        let initial: [u8; 32] = kani::any();
        kani::assume(1 <= nbytes && nbytes <= 8);
        kani::assume(nbytes <= slice_len && slice_len <= 32);
        if nbytes < 8 { kani::assume(value >> (8 * nbytes) == 0); }
        let mut original = initial;
        let mut safe = initial;
        <byteorder::LittleEndian as byteorder::ByteOrder>::write_uint(&mut original[..slice_len], value, nbytes);
        <byteorder_lite::LittleEndian as byteorder_lite::ByteOrder>::write_uint(&mut safe[..slice_len], value, nbytes);
        assert_eq!(original, safe);
        for i in nbytes..32 { assert_eq!(original[i], initial[i]); }
    }
    #[kani::proof]
    #[kani::unwind(34)]
    fn be_write_uint128_original_bridge() {
        let value: u128 = kani::any();
        let nbytes: usize = kani::any();
        let slice_len: usize = kani::any();
        let initial: [u8; 32] = kani::any();
        kani::assume(1 <= nbytes && nbytes <= 16);
        kani::assume(nbytes <= slice_len && slice_len <= 32);
        if nbytes < 16 { kani::assume(value >> (8 * nbytes) == 0); }
        let mut original = initial;
        let mut safe = initial;
        <byteorder::BigEndian as byteorder::ByteOrder>::write_uint128(&mut original[..slice_len], value, nbytes);
        <byteorder_lite::BigEndian as byteorder_lite::ByteOrder>::write_uint128(&mut safe[..slice_len], value, nbytes);
        assert_eq!(original, safe);
        for i in nbytes..32 { assert_eq!(original[i], initial[i]); }
    }
    #[kani::proof]
    #[kani::unwind(34)]
    fn le_write_uint128_original_bridge() {
        let value: u128 = kani::any();
        let nbytes: usize = kani::any();
        let slice_len: usize = kani::any();
        let initial: [u8; 32] = kani::any();
        kani::assume(1 <= nbytes && nbytes <= 16);
        kani::assume(nbytes <= slice_len && slice_len <= 32);
        if nbytes < 16 { kani::assume(value >> (8 * nbytes) == 0); }
        let mut original = initial;
        let mut safe = initial;
        <byteorder::LittleEndian as byteorder::ByteOrder>::write_uint128(&mut original[..slice_len], value, nbytes);
        <byteorder_lite::LittleEndian as byteorder_lite::ByteOrder>::write_uint128(&mut safe[..slice_len], value, nbytes);
        assert_eq!(original, safe);
        for i in nbytes..32 { assert_eq!(original[i], initial[i]); }
    }
}
