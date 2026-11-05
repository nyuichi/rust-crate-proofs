use bit_field::BitField;
pub fn original_u64_get_range(value:u64,start:usize,end:usize)->u64 { <u64 as BitField>::get_bits(&value,start..end) }
pub fn original_u64_set_range(value:&mut u64,start:usize,end:usize,replacement:u64)->&mut u64 { <u64 as BitField>::set_bits(value,start..end,replacement) }
