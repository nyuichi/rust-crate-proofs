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
enum Constructor {Seed,SeedU64}
#[kani::proof]
#[kani::unwind(80)]
fn original_xorshift_debug_constant(){
 let ctor:Constructor=kani::any();let fail:bool=kani::any();
 let rng=match ctor {
  Constructor::Seed=>rand_xorshift::XorShiftRng::from_seed(kani::any()),
  Constructor::SeedU64=>rand_xorshift::XorShiftRng::seed_from_u64(kani::any()),
 };
 check(rng,b"XorShiftRng {}",fail);
}
}
