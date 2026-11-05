use bit_field::BitField;
#[kani::proof]
fn get_u16_all_valid_intervals() {
 let value:u16=kani::any();let before=value;
 let start:usize=kani::any();let end:usize=kani::any();
 kani::assume(start<16 && end<=16 && start<=end);
 let width=end-start;
 let actual=<u16 as BitField>::get_bits(&value,start..end);
 let inlined_caller=bitfield_range_u16_inline::original_u16_get_range(value,start,end);
 let expected=(u32::from(before)/(1u32<<start))%(1u32<<width);
 assert_eq!(u32::from(actual),expected);
 assert_eq!(inlined_caller,actual);
 assert_eq!(value,before);
}
#[kani::proof]
fn set_u16_all_valid_intervals_and_fit_values() {
 let before:u16=kani::any();let replacement:u16=kani::any();let writeback:u16=kani::any();
 let start:usize=kani::any();let end:usize=kani::any();
 kani::assume(start<16 && end<=16 && start<=end);
 let width=end-start;
 kani::assume(u32::from(replacement)<(1u32<<width));
 let mask=((1u32<<width)-1)<<start;
 let expected=(u32::from(before)&!mask)|(u32::from(replacement)<<start);
 let mut original=before;let mut caller=before;
 let original_ref=<u16 as BitField>::set_bits(&mut original,start..end,replacement);
 let caller_ref=bitfield_range_u16_inline::original_u16_set_range(&mut caller,start,end,replacement);
 let observed=*original_ref;let caller_observed=*caller_ref;
 *original_ref=writeback;*caller_ref=writeback;
 assert_eq!(u32::from(observed),expected);
 assert_eq!(caller_observed,observed);
 assert_eq!(u32::from(observed)&!mask,u32::from(before)&!mask);
 assert_eq!(original,writeback);assert_eq!(caller,writeback);
}
