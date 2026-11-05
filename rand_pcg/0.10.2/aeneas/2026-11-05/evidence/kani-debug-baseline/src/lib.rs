#[cfg(kani)]
mod checks {
use core::fmt::{self,Write};
use rand_core::SeedableRng;
struct Sink{bytes:[u8;64],len:usize,fail:bool,attempt:bool}
impl Write for Sink {
 fn write_str(&mut self,s:&str)->fmt::Result {
  if !s.is_empty(){self.attempt=true;}
  if self.fail{return Err(fmt::Error)}
  let end=self.len+s.len();assert!(end<=64);
  self.bytes[self.len..end].copy_from_slice(s.as_bytes());self.len=end;Ok(())
 }
}
fn check<T:fmt::Debug>(rng:T,literal:&[u8],fail:bool){
 let mut sink=Sink{bytes:[0;64],len:0,fail,attempt:false};
 let result=write!(&mut sink,"{:?}",rng);
 assert!(sink.attempt);
 assert_eq!(result.is_err(),fail);
 assert_eq!(sink.len,if fail {0}else{literal.len()});
 for i in 0..64 {let expected=if !fail && i<literal.len(){literal[i]}else{0};assert_eq!(sink.bytes[i],expected);}
}
#[derive(kani::Arbitrary)]
enum Constructor {New,State,Seed,SeedU64}
#[kani::proof]
#[kani::unwind(80)]
fn lcg64_debug_constant(){
 let ctor:Constructor=kani::any();let fail:bool=kani::any();
 let state:u64=kani::any();let stream:u64=kani::any();
 let rng=match ctor {
  Constructor::New=>rand_pcg::Lcg64Xsh32::new(state,stream),
  Constructor::State=>rand_pcg::Lcg64Xsh32::from_state(state,stream),
  Constructor::Seed=>rand_pcg::Lcg64Xsh32::from_seed(kani::any()),
  Constructor::SeedU64=>rand_pcg::Lcg64Xsh32::seed_from_u64(kani::any()),
 };
 check(rng,b"Lcg64Xsh32 {}",fail);
}
#[kani::proof]
#[kani::unwind(80)]
fn lcg128_debug_constant(){
 let ctor:Constructor=kani::any();let fail:bool=kani::any();
 let state:u128=kani::any();let stream:u128=kani::any();
 let rng=match ctor {
  Constructor::New=>rand_pcg::Lcg128Xsl64::new(state,stream),
  Constructor::State=>rand_pcg::Lcg128Xsl64::from_state(state,stream),
  Constructor::Seed=>rand_pcg::Lcg128Xsl64::from_seed(kani::any()),
  Constructor::SeedU64=>rand_pcg::Lcg128Xsl64::seed_from_u64(kani::any()),
 };
 check(rng,b"Lcg128Xsl64 {}",fail);
}

#[kani::proof]
#[kani::unwind(80)]
fn lcg128cm_debug_constant(){
 let ctor:Constructor=kani::any();let fail:bool=kani::any();
 let state:u128=kani::any();let stream:u128=kani::any();
 let rng=match ctor {
  Constructor::New=>rand_pcg::Lcg128CmDxsm64::new(state,stream),
  Constructor::State=>rand_pcg::Lcg128CmDxsm64::from_state(state,stream),
  Constructor::Seed=>rand_pcg::Lcg128CmDxsm64::from_seed(kani::any()),
  Constructor::SeedU64=>rand_pcg::Lcg128CmDxsm64::seed_from_u64(kani::any()),
 };
 check(rng,b"Lcg128CmDxsm64 {}",fail);
}

#[derive(kani::Arbitrary)]
enum McgConstructor{New,Seed,SeedU64}
#[kani::proof]
#[kani::unwind(80)]
fn mcg128_debug_constant(){
 let ctor:McgConstructor=kani::any();let fail:bool=kani::any();
 let rng=match ctor {
  McgConstructor::New=>rand_pcg::Mcg128Xsl64::new(kani::any()),
  McgConstructor::Seed=>rand_pcg::Mcg128Xsl64::from_seed(kani::any()),
  McgConstructor::SeedU64=>rand_pcg::Mcg128Xsl64::seed_from_u64(kani::any()),
 };
 check(rng,b"Mcg128Xsl64 {}",fail);
}

}
