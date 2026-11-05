#[cfg(kani)]
mod checks {
    use bit_field::BitField;
    #[kani::proof]
    fn observed_signed_extraction_and_update() {
        assert_eq!(8_i8.get_bits(0..4), -8_i8);
        assert_eq!((-1_i8).get_bits(0..4), -1_i8);
        let mut x = 112_i8;
        x.set_bits(0..4, 1);
        assert_eq!(x, 1_i8);
        let mut y = 112_u8;
        y.set_bits(0..4, 1);
        assert_eq!(y, 113_u8);
    }
    #[kani::proof]
    fn outside_range_preservation_candidate() {
        let original = 112_i8;
        let mut x = original;
        x.set_bits(0..4, 1);
        assert_eq!((x as u8) & !0x0f_u8, (original as u8) & !0x0f_u8);
    }
}
