use bit_field::BitField;
#[kani::proof]
fn get_u8_all_valid_intervals() {
 let value:u8=kani::any();let before=value;
 let start:usize=kani::any();let end:usize=kani::any();
 kani::assume(start<8 && end<=8 && start<=end);
 let width=end-start;
 let actual=<u8 as BitField>::get_bits(&value,start..end);
 let inlined_caller=bitfield_range_u8_inline::original_u8_get_range(value,start,end);
 let expected=(u16::from(before)/(1u16<<start))%(1u16<<width);
 assert_eq!(u16::from(actual),expected);
 assert_eq!(inlined_caller,actual);
 assert_eq!(value,before);
}
#[kani::proof]
fn set_u8_all_valid_intervals_and_fit_values() {
 let before:u8=kani::any();let replacement:u8=kani::any();let writeback:u8=kani::any();
 let start:usize=kani::any();let end:usize=kani::any();
 kani::assume(start<8 && end<=8 && start<=end);
 let width=end-start;
 kani::assume(u16::from(replacement)<(1u16<<width));
 let mask=((1u16<<width)-1)<<start;
 let expected=(u16::from(before)&!mask)|(u16::from(replacement)<<start);
 let mut original=before;let mut caller=before;
 let original_ref=<u8 as BitField>::set_bits(&mut original,start..end,replacement);
 let caller_ref=bitfield_range_u8_inline::original_u8_set_range(&mut caller,start,end,replacement);
 let observed=*original_ref;let caller_observed=*caller_ref;
 *original_ref=writeback;*caller_ref=writeback;
 assert_eq!(u16::from(observed),expected);
 assert_eq!(caller_observed,observed);
 assert_eq!(u16::from(observed)&!mask,u16::from(before)&!mask);
 assert_eq!(original,writeback);assert_eq!(caller,writeback);
}
