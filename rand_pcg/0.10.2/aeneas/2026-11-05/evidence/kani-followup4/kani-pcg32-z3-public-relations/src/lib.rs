#[cfg(kani)]
mod checks {
    use rand_pcg::{Lcg64Xsh32,rand_core::TryRng};
    #[kani::proof]
    fn original_pcg32_public_two_call_composition() {
        let state:u64=kani::any();
        let stream:u64=kani::any();
        let mut wide=Lcg64Xsh32::from_state(state,stream);
        let mut narrow=wide.clone();
        let result=match wide.try_next_u64(){Ok(x)=>x,Err(e)=>match e{}};
        let low=match narrow.try_next_u32(){Ok(x)=>x,Err(e)=>match e{}};
        let high=match narrow.try_next_u32(){Ok(x)=>x,Err(e)=>match e{}};
        assert_eq!(result,u64::from(low)|(u64::from(high)<<32));
        assert!(wide==narrow);
    }
}
#[cfg(kani)]
mod seed_checks {
 use rand_pcg::{Lcg64Xsh32,rand_core::SeedableRng};
 fn le64(b:[u8;8])->u64 {
  u64::from(b[0]) | (u64::from(b[1])<<8) | (u64::from(b[2])<<16) |
  (u64::from(b[3])<<24) | (u64::from(b[4])<<32) | (u64::from(b[5])<<40) |
  (u64::from(b[6])<<48) | (u64::from(b[7])<<56)
 }
 #[kani::proof]
 fn original_lcg64_seed_equals_public_new() {
  let seed:[u8;16]=kani::any();
  let state=le64([seed[0],seed[1],seed[2],seed[3],seed[4],seed[5],seed[6],seed[7]]);
  let raw_increment=le64([seed[8],seed[9],seed[10],seed[11],seed[12],seed[13],seed[14],seed[15]]);
  let actual=Lcg64Xsh32::from_seed(seed);
  let expected=Lcg64Xsh32::new(state,raw_increment>>1);
  assert!(actual==expected);
 }
}
