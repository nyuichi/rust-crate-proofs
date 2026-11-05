use serde::{Serialize,Serializer};
use serde::ser::{self,SerializeStruct,Impossible};

use rand_pcg::Lcg64Xsh32;
#[derive(Debug,PartialEq,Eq)] struct E(u32);
impl core::fmt::Display for E { fn fmt(&self,f:&mut core::fmt::Formatter<'_>)->core::fmt::Result {write!(f,"error")} }
impl core::error::Error for E {}
impl ser::Error for E {fn custom<T:core::fmt::Display>(_:T)->Self {E(u32::MAX)}}
struct Sink { bytes:[u8;16], fields:usize, fail:u8, code:u32, calls:usize, nested:bool }
impl Sink { fn stage(&mut self,n:u8)->Result<(),E>{self.calls+=1;if self.fail==n {Err(E(self.code))}else{Ok(())}} }
impl<'a> Serializer for &'a mut Sink {
 type Ok=();type Error=E;
 type SerializeSeq=Impossible<(),E>;type SerializeTuple=Impossible<(),E>;type SerializeTupleStruct=Impossible<(),E>;type SerializeTupleVariant=Impossible<(),E>;type SerializeMap=Impossible<(),E>;type SerializeStruct=Self;type SerializeStructVariant=Impossible<(),E>;
 fn serialize_struct(self,name:&'static str,len:usize)->Result<Self,E>{ assert_eq!(name,"Lcg64Xsh32");assert_eq!(len,2);self.stage(0)?;Ok(self) }
 fn serialize_u64(self,v:u64)->Result<(),E>{ assert!(self.nested);assert!(self.fields<2);self.stage((2+2*self.fields)as u8)?;let b=v.to_le_bytes();for i in 0..8{self.bytes[8*self.fields+i]=b[i];}Ok(()) }
fn serialize_bool(self,_:bool)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i8(self,_:i8)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i16(self,_:i16)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i32(self,_:i32)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i64(self,_:i64)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_i128(self,_:i128)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u8(self,_:u8)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u16(self,_:u16)->Result<(),E>{Err(E(u32::MAX))}
fn serialize_u32(self,_:u32)->Result<(),E>{Err(E(u32::MAX))}
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
 fn serialize_field<T:?Sized+Serialize>(&mut self,key:&'static str,v:&T)->Result<(),E>{assert!(self.fields<2);assert_eq!(key,["state","increment"][self.fields]);self.stage((1+2*self.fields)as u8)?;self.nested=true;v.serialize(&mut **self)?;self.nested=false;self.fields+=1;Ok(())}
 fn skip_field(&mut self,_:&'static str)->Result<(),E>{Err(E(u32::MAX))}
 fn end(self)->Result<(),E>{assert_eq!(self.fields,2);self.stage(5)}
}

use serde::{Deserialize,Deserializer};
use serde::de::{self,Visitor,SeqAccess,DeserializeSeed};
impl de::Error for E {
 fn custom<T:core::fmt::Display>(_:T)->Self{E(u32::MAX)}
 fn invalid_length(n:usize,_:&dyn de::Expected)->Self{E(0x100+n as u32)}
}
struct Input{bytes:[u8;16],length:usize,index:usize,fail:u8,code:u32,calls:usize}
impl Input {fn stage(&mut self,n:u8)->Result<(),E>{self.calls+=1;if n==self.fail{Err(E(self.code))}else{Ok(())}}}
struct Word<'a>(&'a mut Input);
impl<'de> Deserializer<'de> for Word<'_>{type Error=E;
 fn deserialize_any<V:Visitor<'de>>(self,_:V)->Result<V::Value,E>{Err(E(u32::MAX))}
 fn deserialize_u64<V:Visitor<'de>>(self,v:V)->Result<V::Value,E>{self.0.stage((2+2*self.0.index)as u8)?;let i=8*self.0.index;v.visit_u64(u64::from_le_bytes([self.0.bytes[i],self.0.bytes[i+1],self.0.bytes[i+2],self.0.bytes[i+3],self.0.bytes[i+4],self.0.bytes[i+5],self.0.bytes[i+6],self.0.bytes[i+7]]))}
 serde::forward_to_deserialize_any!{bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any}
}
impl<'de> SeqAccess<'de> for &mut Input{type Error=E;
 fn next_element_seed<T:DeserializeSeed<'de>>(&mut self,seed:T)->Result<Option<T::Value>,E>{self.stage((1+2*self.index)as u8)?;if self.index>=self.length{return Ok(None)}let val=seed.deserialize(Word(&mut **self))?;self.index+=1;Ok(Some(val))}
}
impl<'de> Deserializer<'de> for &mut Input{type Error=E;
 fn deserialize_any<V:Visitor<'de>>(self,_:V)->Result<V::Value,E>{Err(E(u32::MAX))}
 fn deserialize_struct<V:Visitor<'de>>(self,name:&'static str,fields:&'static[&'static str],v:V)->Result<V::Value,E>{assert_eq!(name,"Lcg64Xsh32");assert_eq!(fields,["state","increment"]);self.stage(0)?;v.visit_seq(self)}
 serde::forward_to_deserialize_any!{bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any}
}
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(20)]
fn original_deserialize_fixed_protocol(){
 let bytes:[u8;16]=kani::any();let length:usize=kani::any();kani::assume(length<=2);let fail:u8=kani::any();kani::assume(fail<=5);let code:u32=kani::any();
 let mut input=Input{bytes,length,index:0,fail,code,calls:0};let result=Lcg64Xsh32::deserialize(&mut input);
 let last=if length==2{4}else{1+2*length};
 if fail as usize<=last{assert_eq!(result,Err(E(code)));assert_eq!(input.calls,fail as usize+1);assert_eq!(input.index,if fail==0{0}else{(fail as usize-1)/2});}
 else if length<2 {assert_eq!(result,Err(E(0x100+length as u32)));assert_eq!(input.calls,2+2*length);assert_eq!(input.index,length);}
 else{let rng=result.unwrap();assert_eq!(input.calls,5);assert_eq!(input.index,2);let old=rng.clone();assert_eq!(rng.state(),u64::from_le_bytes(bytes[0..8].try_into().unwrap()));assert_eq!(rng.stream(),u64::from_le_bytes(bytes[8..16].try_into().unwrap())>>1);let mut sink=Sink{bytes:[0;16],fields:0,fail:6,code:0,calls:0,nested:false};assert_eq!(rng.serialize(&mut sink),Ok(()));assert_eq!(sink.bytes,bytes);assert_eq!(sink.fields,2);assert_eq!(sink.calls,6);assert_eq!(rng,old);}
}
