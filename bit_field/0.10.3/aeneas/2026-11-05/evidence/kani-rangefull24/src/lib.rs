use bit_field::BitField;
#[kani::proof]
fn get_full_u8(){let value:u8=kani::any();let before=value;assert_eq!(<u8 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_u8(){let mut value:u8=kani::any();let replacement:u8=kani::any();let writeback:u8=kani::any();let returned=<u8 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_u16(){let value:u16=kani::any();let before=value;assert_eq!(<u16 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_u16(){let mut value:u16=kani::any();let replacement:u16=kani::any();let writeback:u16=kani::any();let returned=<u16 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_u32(){let value:u32=kani::any();let before=value;assert_eq!(<u32 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_u32(){let mut value:u32=kani::any();let replacement:u32=kani::any();let writeback:u32=kani::any();let returned=<u32 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_u64(){let value:u64=kani::any();let before=value;assert_eq!(<u64 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_u64(){let mut value:u64=kani::any();let replacement:u64=kani::any();let writeback:u64=kani::any();let returned=<u64 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_u128(){let value:u128=kani::any();let before=value;assert_eq!(<u128 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_u128(){let mut value:u128=kani::any();let replacement:u128=kani::any();let writeback:u128=kani::any();let returned=<u128 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_usize(){let value:usize=kani::any();let before=value;assert_eq!(<usize as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_usize(){let mut value:usize=kani::any();let replacement:usize=kani::any();let writeback:usize=kani::any();let returned=<usize as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_i8(){let value:i8=kani::any();let before=value;assert_eq!(<i8 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_i8(){let mut value:i8=kani::any();let replacement:i8=kani::any();let writeback:i8=kani::any();let returned=<i8 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_i16(){let value:i16=kani::any();let before=value;assert_eq!(<i16 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_i16(){let mut value:i16=kani::any();let replacement:i16=kani::any();let writeback:i16=kani::any();let returned=<i16 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_i32(){let value:i32=kani::any();let before=value;assert_eq!(<i32 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_i32(){let mut value:i32=kani::any();let replacement:i32=kani::any();let writeback:i32=kani::any();let returned=<i32 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_i64(){let value:i64=kani::any();let before=value;assert_eq!(<i64 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_i64(){let mut value:i64=kani::any();let replacement:i64=kani::any();let writeback:i64=kani::any();let returned=<i64 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_i128(){let value:i128=kani::any();let before=value;assert_eq!(<i128 as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_i128(){let mut value:i128=kani::any();let replacement:i128=kani::any();let writeback:i128=kani::any();let returned=<i128 as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
#[kani::proof]
fn get_full_isize(){let value:isize=kani::any();let before=value;assert_eq!(<isize as BitField>::get_bits(&value,..),before);assert_eq!(value,before);}
#[kani::proof]
fn set_full_isize(){let mut value:isize=kani::any();let replacement:isize=kani::any();let writeback:isize=kani::any();let returned=<isize as BitField>::set_bits(&mut value,..,replacement);let observed=*returned;*returned=writeback;assert_eq!(observed,replacement);assert_eq!(value,writeback);}
