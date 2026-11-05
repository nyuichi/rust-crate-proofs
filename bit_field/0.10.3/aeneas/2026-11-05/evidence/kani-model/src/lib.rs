#[cfg(kani)]
mod checks {
    use bit_field::BitField;
    #[kani::proof]
    fn u8_layout_model() {
        assert_eq!(core::mem::size_of::<u8>(), 1);
        assert_eq!(u8::BIT_LENGTH, 8);
    }
    #[kani::proof]
    fn original_u8_get_and_set() {
        let x: u8 = kani::any();
        let bit: usize = kani::any();
        let value: bool = kani::any();
        kani::assume(bit < 8);
        assert_eq!(x.get_bit(bit), ((x >> bit) & 1) == 1);
        let mask = 1u8 << bit;
        let expected = if value { x | mask } else { x & !mask };
        let mut y = x;
        let returned = y.set_bit(bit, value);
        assert_eq!(*returned, expected);
        assert_eq!(returned.get_bit(bit), value);
        let replacement: u8 = kani::any();
        *returned = replacement;
        assert_eq!(y, replacement);
    }
}
