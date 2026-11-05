#[cfg(kani)]
mod checks {
 use crate_under_test::{BigEndian,LittleEndian,ByteOrder};
 #[kani::proof]
 fn f32_bit_pattern_identity() {
  let bits: u32 = kani::any();
  let value=f32::from_bits(bits);
  assert_eq!(value.to_bits(),bits);
  assert_eq!(f32::from_bits(value.to_bits()).to_bits(),bits);
 }
 #[kani::proof]
 fn f64_bit_pattern_identity() {
  let bits: u64 = kani::any();
  let value=f64::from_bits(bits);
  assert_eq!(value.to_bits(),bits);
  assert_eq!(f64::from_bits(value.to_bits()).to_bits(),bits);
 }
 #[kani::proof]
 #[kani::unwind(12)]
 fn be_write_f32_bits_and_suffix() {
  let bits: u32 = kani::any();
  let value=f32::from_bits(bits);
  let mut buffer: [u8; 6] = kani::any();
  let before=buffer;
  BigEndian::write_f32(&mut buffer,value);
  let expected=bits.to_be_bytes();
  for i in 0..4 { assert_eq!(buffer[i],expected[i]); }
  for i in 4..6 { assert_eq!(buffer[i],before[i]); }
 }
 #[kani::proof]
 #[kani::unwind(12)]
 fn le_write_f32_bits_and_suffix() {
  let bits: u32 = kani::any();
  let value=f32::from_bits(bits);
  let mut buffer: [u8; 6] = kani::any();
  let before=buffer;
  LittleEndian::write_f32(&mut buffer,value);
  let expected=bits.to_le_bytes();
  for i in 0..4 { assert_eq!(buffer[i],expected[i]); }
  for i in 4..6 { assert_eq!(buffer[i],before[i]); }
 }
 #[kani::proof]
 #[kani::unwind(12)]
 fn be_write_f64_bits_and_suffix() {
  let bits: u64 = kani::any();
  let value=f64::from_bits(bits);
  let mut buffer: [u8; 10] = kani::any();
  let before=buffer;
  BigEndian::write_f64(&mut buffer,value);
  let expected=bits.to_be_bytes();
  for i in 0..8 { assert_eq!(buffer[i],expected[i]); }
  for i in 8..10 { assert_eq!(buffer[i],before[i]); }
 }
 #[kani::proof]
 #[kani::unwind(12)]
 fn le_write_f64_bits_and_suffix() {
  let bits: u64 = kani::any();
  let value=f64::from_bits(bits);
  let mut buffer: [u8; 10] = kani::any();
  let before=buffer;
  LittleEndian::write_f64(&mut buffer,value);
  let expected=bits.to_le_bytes();
  for i in 0..8 { assert_eq!(buffer[i],expected[i]); }
  for i in 8..10 { assert_eq!(buffer[i],before[i]); }
 }
}
