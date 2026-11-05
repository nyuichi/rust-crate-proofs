use bit_field::BitField;
pub fn original_u32_get_range(value:u32,start:usize,end:usize)->u32 { <u32 as BitField>::get_bits(&value,start..end) }
pub fn original_u32_set_range(value:&mut u32,start:usize,end:usize,replacement:u32)->&mut u32 { <u32 as BitField>::set_bits(value,start..end,replacement) }
