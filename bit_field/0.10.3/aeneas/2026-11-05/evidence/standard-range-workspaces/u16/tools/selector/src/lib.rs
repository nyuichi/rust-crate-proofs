use bit_field::BitField;
pub fn original_u16_get_range(value:u16,start:usize,end:usize)->u16 { <u16 as BitField>::get_bits(&value,start..end) }
pub fn original_u16_set_range(value:&mut u16,start:usize,end:usize,replacement:u16)->&mut u16 { <u16 as BitField>::set_bits(value,start..end,replacement) }
