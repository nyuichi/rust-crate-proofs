use serde::{Serialize,Serializer};
use serde::ser::{self,SerializeStruct,Impossible};
use rand_core::SeedableRng;
use rand_xorshift::XorShiftRng;
#[derive(Debug,PartialEq,Eq)] struct E(u32);
impl core::fmt::Display for E { fn fmt(&self,f:&mut core::fmt::Formatter<'_>)->core::fmt::Result {write!(f,"error")} }
impl core::error::Error for E {}
impl ser::Error for E {fn custom<T:core::fmt::Display>(_:T)->Self {E(u32::MAX)}}
struct Sink { bytes:[u8;16], fields:usize, fail:u8, code:u32, calls:usize, nested:bool }
impl Sink { fn stage(&mut self,n:u8)->Result<(),E>{self.calls+=1;if self.fail==n {Err(E(self.code))}else{Ok(())}} }
impl<'a> Serializer for &'a mut Sink {
 type Ok=();type Error=E;
 type SerializeSeq=Impossible<(),E>;type SerializeTuple=Impossible<(),E>;type SerializeTupleStruct=Impossible<(),E>;type SerializeTupleVariant=Impossible<(),E>;type SerializeMap=Impossible<(),E>;type SerializeStruct=Self;type SerializeStructVariant=Impossible<(),E>;
 fn serialize_struct(self,name:&'static str,len:usize)->Result<Self,E>{ assert_eq!(name,"XorShiftRng");assert_eq!(len,4);self.stage(0)?;Ok(self) }
 fn serialize_u32(self,v:u32)->Result<(),E>{ assert!(self.nested);assert!(self.fields<4);self.stage((2+2*self.fields)as u8)?;let b=v.to_le_bytes();for i in 0..4{self.bytes[4*self.fields+i]=b[i];}Ok(()) }
fn serialize_bool(self,_:bool)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i8(self,_:i8)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i16(self,_:i16)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i32(self,_:i32)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i64(self,_:i64)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i128(self,_:i128)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u8(self,_:u8)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u16(self,_:u16)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u64(self,_:u64)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u128(self,_:u128)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_f32(self,_:f32)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_f64(self,_:f64)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_char(self,_:char)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_str(self,_:&str)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_bytes(self,_:&[u8])->Result<(),E>{Err(E(u32::MAX))}
fn serialize_none(self)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_unit(self)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_unit_struct(self,_:&'static str)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_unit_variant(self,_:&'static str,_:u32,_:&'static str)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_seq(self,_:Option<usize>)->Result<Self::SerializeSeq,E>{Err(E(u32::MAX))}
fn serialize_tuple(self,_:usize)->Result<Self::SerializeTuple,E>{Err(E(u32::MAX))}
fn serialize_tuple_struct(self,_:&'static str,_:usize)->Result<Self::SerializeTupleStruct,E>{Err(E(u32::MAX))}
fn serialize_tuple_variant(self,_:&'static str,_:u32,_:&'static str,_:usize)->Result<Self::SerializeTupleVariant,E>{Err(E(u32::MAX))}
fn serialize_map(self,_:Option<usize>)->Result<Self::SerializeMap,E>{Err(E(u32::MAX))}
fn serialize_struct_variant(self,_:&'static str,_:u32,_:&'static str,_:usize)->Result<Self::SerializeStructVariant,E>{Err(E(u32::MAX))}
fn serialize_some<T:?Sized+Serialize>(self,_:&T)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_newtype_struct<T:?Sized+Serialize>(self,_:&'static str,_:&T)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_newtype_variant<T:?Sized+Serialize>(self,_:&'static str,_:u32,_:&'static str,_:&T)->Result<(),E>{Err(E(u32::MAX))}
fn collect_str<T:?Sized+core::fmt::Display>(self,_:&T)->Result<(),E>{Err(E(u32::MAX))}
}
impl SerializeStruct for &mut Sink {type Ok=();type Error=E;
 fn serialize_field<T:?Sized+Serialize>(&mut self,key:&'static str,v:&T)->Result<(),E>{assert!(self.fields<4);assert_eq!(key,["x","y","z","w"][self.fields]);self.stage((1+2*self.fields)as u8)?;self.nested=true;v.serialize(&mut **self)?;self.nested=false;self.fields+=1;Ok(())}
 fn skip_field(&mut self,_:&'static str)->Result<(),E>{Err(E(u32::MAX))}
 fn end(self)->Result<(),E>{assert_eq!(self.fields,4);self.stage(9)}
}
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(20)]
fn original_serialize_fixed_protocol(){
 let seed:[u8;16]=kani::any();let fail:u8=kani::any();kani::assume(fail<=10);let code:u32=kani::any();
 let rng=XorShiftRng::from_seed(seed);let old=rng.clone();let mut sink=Sink{bytes:[0;16],fields:0,fail,code,calls:0,nested:false};
 let result=rng.serialize(&mut sink);assert_eq!(rng,old);
 let complete=if fail<=9 {assert_eq!(result,Err(E(code)));assert_eq!(sink.calls,fail as usize+1);if fail==0{0}else{(fail as usize-1)/2}}else{assert_eq!(result,Ok(()));assert_eq!(sink.calls,10);4};
 assert_eq!(sink.fields,complete);
 for i in 0..16 {let expected=if seed==[0;16]{[0xed,0x5e,0xad,0x0b][i%4]}else{seed[i]};assert_eq!(sink.bytes[i],if i<4*complete{expected}else{0});}
}
