use bit_field::BitField;
pub fn original_u8_get_range(value:u8,start:usize,end:usize)->u8 { <u8 as BitField>::get_bits(&value,start..end) }
pub fn original_u8_set_range(value:&mut u8,start:usize,end:usize,replacement:u8)->&mut u8 { <u8 as BitField>::set_bits(value,start..end,replacement) }
