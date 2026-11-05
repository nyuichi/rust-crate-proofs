include!(concat!(env!("OUT_DIR"),"/modules.rs"));
#[cfg(kani)]
mod checks {
 #[kani::proof]
 #[kani::unwind(20)]
 fn from_seed_exact() {
  let seed: [u8;16]=kani::any();
  let got=super::original::verification_from_seed(seed);
  let mut words=[0u32;4];
  for i in 0..4 {
   let j=4*i;
   words[i]=(seed[j] as u32) | ((seed[j+1] as u32)<<8)
       | ((seed[j+2] as u32)<<16) | ((seed[j+3] as u32)<<24);
  }
  let expected=if words==[0;4] {[0xBAD_5EED;4]} else {words};
  assert_eq!(got,expected);
  assert_ne!(got,[0;4]);
 }
 #[kani::proof]
 #[kani::unwind(20)]
 fn nonzero_state_is_preserved() {
  let words:[u32;4]=kani::any();
  kani::assume(words != [0;4]);
  let next=super::original::verification_next_state(words);
  assert_ne!(next,[0;4]);
 }
}
