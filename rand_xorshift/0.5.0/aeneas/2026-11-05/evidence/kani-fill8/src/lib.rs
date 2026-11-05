#[cfg(kani)]
mod checks {
 use rand_xorshift::XorShiftRng;
 use rand_core::{TryRng,SeedableRng};
 #[kani::proof]
 #[kani::unwind(20)]
 fn original_xorshift_fill_up_to8() {
  let seed:[u8;16]=kani::any();
  let len:usize=kani::any(); kani::assume(len<=8);
  let initial:[u8;8]=kani::any();let mut actual=initial;
  let mut fill=XorShiftRng::from_seed(seed);let mut words=fill.clone();
  let result=fill.try_fill_bytes(&mut actual[..len]); assert!(result.is_ok());
  for word_index in 0..2 {
   if word_index*4<len {
   let word=match words.try_next_u32(){Ok(x)=>x,Err(e)=>match e{}};
   for byte_index in 0..4 {
    let i=word_index*4+byte_index;
    if i<len {assert_eq!(actual[i],((word>>(byte_index*8))&255)as u8);}
   }
  }
  }
  for i in 0..8 {if i>=len {assert_eq!(actual[i],initial[i]);}}
  assert!(fill==words);
 }
}
