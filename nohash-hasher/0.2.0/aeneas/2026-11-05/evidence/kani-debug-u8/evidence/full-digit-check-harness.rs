#[cfg(kani)]
mod verification {
use core::fmt::{self, Write};
use core::hash::Hasher;
use nohash_hasher::NoHashHasher;
struct Sink { bytes:[u8;64],len:usize,fail:bool,attempts:usize }
impl Write for Sink {
 fn write_str(&mut self,s:&str)->fmt::Result {
  if !s.is_empty() {self.attempts+=1;}
  if self.fail {return Err(fmt::Error)}
  let end=self.len+s.len(); assert!(end<=64);
  self.bytes[self.len..end].copy_from_slice(s.as_bytes());self.len=end;Ok(())
 }
}
fn check<T>(h:NoHashHasher<T>, value:u64,written:bool,fail:bool) {
 let mut sink=Sink{bytes:[0;64],len:0,fail,attempts:0};
 let result=write!(&mut sink,"{:?}",h);
 assert!(sink.attempts>0);
 if fail {assert!(result.is_err());assert_eq!(sink.len,0);for i in 0..64 {assert_eq!(sink.bytes[i],0);}return}
 assert!(result.is_ok());
 let prefix=b"NoHashHasher(";
 for i in 0..13 {assert_eq!(sink.bytes[i],prefix[i]);}
 let mut end=sink.len-1;
 assert_eq!(sink.bytes[end],b')');
 #[cfg(debug_assertions)] {
  let suffix:&[u8]=if written {b", true"} else {b", false"};
  end-=suffix.len();for i in 0..suffix.len() {assert_eq!(sink.bytes[end+i],suffix[i]);}
 }
 #[cfg(not(debug_assertions))] let _=written;
 assert!(end>13 && end<=33);
 // Independent decimal interpretation: digit syntax, canonicality, and value.
 if end>14 {assert_ne!(sink.bytes[13],b'0');}
 let mut parsed:u64=0;
 for i in 13..33 {if i<end {let d=sink.bytes[i];assert!(d>=b'0'&&d<=b'9');parsed=parsed.checked_mul(10).unwrap().checked_add((d-b'0') as u64).unwrap();}}
 assert_eq!(parsed,value);
 for i in 0..64 {if i>=sink.len {assert_eq!(sink.bytes[i],0);}}
}
#[kani::proof]
#[kani::unwind(80)]
fn original_u8_debug() {
 let value:u8=kani::any();let written:bool=kani::any();let fail:bool=kani::any();
 let mut h=NoHashHasher::<u8>::default();if written {h.write_u8(value);}
 check(h,if written {value as u64}else{0},written,fail);
}
}
