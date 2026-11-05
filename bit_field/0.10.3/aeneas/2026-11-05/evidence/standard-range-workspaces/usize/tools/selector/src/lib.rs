use bit_field::BitField;
pub fn original_usize_get_range(value:usize,start:usize,end:usize)->usize { <usize as BitField>::get_bits(&value,start..end) }
pub fn original_usize_set_range(value:&mut usize,start:usize,end:usize,replacement:usize)->&mut usize { <usize as BitField>::set_bits(value,start..end,replacement) }
