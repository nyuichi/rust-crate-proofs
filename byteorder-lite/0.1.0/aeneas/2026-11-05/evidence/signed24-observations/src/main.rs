use byteorder_lite::{ByteOrder,BigEndian,LittleEndian};
use std::panic::{catch_unwind,AssertUnwindSafe};
fn run<E:ByteOrder>(name:&str){
 let vals:[i64;12]=[i32::MAX as i64,i32::MIN as i64,(1<<23)-1,1<<23,(1<<23)+1,-(1<<23),-(1<<23)-1,0,1,-1,i64::MAX,i64::MIN];
 for n in vals { for len in [0usize,2,3,5] { for api in ["write_i24","write_int"] {
 if api=="write_i24" && (n<i32::MIN as i64 || n>i32::MAX as i64) {continue}
 let mut buf=vec![0xa5u8;len];
 let ok=catch_unwind(AssertUnwindSafe(||if api=="write_i24" {E::write_i24(&mut buf,n as i32)} else {E::write_int(&mut buf,n,3)})).is_ok();
 let low=(n as u64 & 0x00ff_ffff) as u32;
 let exp=if name=="BE" {[(low>>16)as u8,(low>>8)as u8,low as u8]} else {[low as u8,(low>>8)as u8,(low>>16)as u8]};
 assert_eq!(ok,len>=3);
 if ok {assert_eq!(&buf[..3],&exp);assert!(buf[3..].iter().all(|x|*x==0xa5));}
 else {assert!(buf.iter().all(|x|*x==0xa5));}
 println!("{{\"endian\":\"{}\",\"api\":\"{}\",\"value\":{},\"length\":{},\"returned\":{},\"bytes\":{:?}}}",name,api,n,len,ok,buf);
 }}}
}
fn main(){std::panic::set_hook(Box::new(|_|{}));run::<BigEndian>("BE");run::<LittleEndian>("LE");}
