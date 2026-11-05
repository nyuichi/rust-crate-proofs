#[cfg(kani)]
mod checks {
 use rand_xorshift::XorShiftRng;
 use rand_core::{TryRng,SeedableRng};
 #[kani::proof]
 fn original_xorshift_public_two_call_composition() {
  let seed:[u8;16]=kani::any();
  let mut wide=XorShiftRng::from_seed(seed);
  let mut narrow=wide.clone();
  let result=match wide.try_next_u64(){Ok(x)=>x,Err(e)=>match e{}};
  let low=match narrow.try_next_u32(){Ok(x)=>x,Err(e)=>match e{}};
  let high=match narrow.try_next_u32(){Ok(x)=>x,Err(e)=>match e{}};
  assert_eq!(result,u64::from(low)|(u64::from(high)<<32));
  assert!(wide==narrow);
 }
}
