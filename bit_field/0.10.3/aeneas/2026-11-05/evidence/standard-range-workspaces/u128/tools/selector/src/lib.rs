use bit_field::BitField;
pub fn original_u128_get_range(value:u128,start:usize,end:usize)->u128 { <u128 as BitField>::get_bits(&value,start..end) }
pub fn original_u128_set_range(value:&mut u128,start:usize,end:usize,replacement:u128)->&mut u128 { <u128 as BitField>::set_bits(value,start..end,replacement) }
