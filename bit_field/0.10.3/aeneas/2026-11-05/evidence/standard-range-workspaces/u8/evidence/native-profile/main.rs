use bit_field::BitField;
fn main() {
 let mut value=0u8;
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
  let _=<u8 as BitField>::set_bits(&mut value,0usize..0usize,1u8);
 }));
 println!("outcome={} state={}",if result.is_ok(){"normal"}else{"caught_panic"},value);
}
