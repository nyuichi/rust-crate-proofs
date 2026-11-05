#[cfg(kani)]
mod checks {
 use core::{convert::Infallible,fmt};
 use rand_xorshift::XorShiftRng;
 use rand_core::{TryRng,SeedableRng};
 #[derive(Debug,PartialEq,Eq)]
 struct Failure(u32);
 impl fmt::Display for Failure {
  fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str("mock-source-failure")}
 }
 impl core::error::Error for Failure {}
 struct Fixed {seed:[u8;16],zero_first:bool,calls:usize}
 impl TryRng for Fixed {
  type Error=Infallible;
  fn try_next_u32(&mut self)->Result<u32,Infallible>{panic!("unexpected next32")}
  fn try_next_u64(&mut self)->Result<u64,Infallible>{panic!("unexpected next64")}
  fn try_fill_bytes(&mut self,dst:&mut[u8])->Result<(),Infallible>{
   assert_eq!(dst.len(),16); self.calls+=1;
   if self.zero_first && self.calls==1 {dst.copy_from_slice(&[0;16]);}
   else {dst.copy_from_slice(&self.seed);}
   Ok(())
  }
 }
 struct FallibleFixed(Fixed);
 impl TryRng for FallibleFixed {
  type Error=Failure;
  fn try_next_u32(&mut self)->Result<u32,Failure>{panic!("unexpected next32")}
  fn try_next_u64(&mut self)->Result<u64,Failure>{panic!("unexpected next64")}
  fn try_fill_bytes(&mut self,dst:&mut[u8])->Result<(),Failure>{
   match self.0.try_fill_bytes(dst){Ok(())=>Ok(()),Err(e)=>match e{}}
  }
 }
 struct Failing {payload:u32,bytes:[u8;16],zero_first:bool,calls:usize}
 impl TryRng for Failing {
  type Error=Failure;
  fn try_next_u32(&mut self)->Result<u32,Failure>{panic!("unexpected next32")}
  fn try_next_u64(&mut self)->Result<u64,Failure>{panic!("unexpected next64")}
  fn try_fill_bytes(&mut self,dst:&mut[u8])->Result<(),Failure>{
   assert_eq!(dst.len(),16);self.calls+=1;
   if self.zero_first && self.calls==1 {dst.copy_from_slice(&[0;16]);Ok(())}
   else {dst.copy_from_slice(&self.bytes);Err(Failure(self.payload))}
  }
 }
 #[kani::proof]
 #[kani::unwind(20)]
 fn original_from_rng_fixed_nonzero_seed(){
  let seed:[u8;16]=kani::any();kani::assume(seed!=[0;16]);let zero_first:bool=kani::any();
  let mut source=Fixed{seed,zero_first,calls:0};
  let actual=XorShiftRng::from_rng(&mut source);
  assert_eq!(source.calls,if zero_first {2}else{1});
  assert!(actual==XorShiftRng::from_seed(seed));
 }
 #[kani::proof]
 #[kani::unwind(20)]
 fn original_try_from_rng_fixed_nonzero_seed(){
  let seed:[u8;16]=kani::any();kani::assume(seed!=[0;16]);let zero_first:bool=kani::any();
  let mut source=FallibleFixed(Fixed{seed,zero_first,calls:0});
  let actual=match XorShiftRng::try_from_rng(&mut source){Ok(x)=>x,Err(_)=>panic!("unexpected error")};
  assert_eq!(source.0.calls,if zero_first {2}else{1});
  assert!(actual==XorShiftRng::from_seed(seed));
 }
 #[kani::proof]
 #[kani::unwind(20)]
 fn original_try_from_rng_exact_source_error(){
  let payload:u32=kani::any();let zero_first:bool=kani::any();let bytes:[u8;16]=kani::any();
  let mut source=Failing{payload,bytes,zero_first,calls:0};
  let error=match XorShiftRng::try_from_rng(&mut source){Err(e)=>e,Ok(_)=>panic!("unexpected seed")};
  assert_eq!(source.calls,if zero_first {2}else{1});assert_eq!(error,Failure(payload));
 }
}
