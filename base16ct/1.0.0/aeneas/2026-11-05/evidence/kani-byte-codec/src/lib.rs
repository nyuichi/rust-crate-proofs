#[cfg(kani)]
mod checks {
fn lower_nibble(c:u8)->Option<u8>{
 if c>=b'0'&&c<=b'9'{Some(c-b'0')}else if c>=b'a'&&c<=b'f'{Some(c-b'a'+10)}else{None}
}
#[kani::proof]
#[kani::unwind(8)]
fn lower_pair_decode_all_bytes(){
 let input:[u8;2]=kani::any();let initial:[u8;3]=kani::any();let mut dst=initial;
 let expected=match (lower_nibble(input[0]),lower_nibble(input[1])){(Some(h),Some(l))=>Some(h*16+l),_=>None};
 let result=base16ct::lower::decode(&input[..],&mut dst);
 match (result,expected){
  (Ok(output),Some(value))=>{assert_eq!(output.len(),1);assert_eq!(output[0],value);}
  (Err(base16ct::Error::InvalidEncoding),None)=>{}
  _=>assert!(false,"exact success/error classification"),
 }
 if let Some(value)=expected {assert_eq!(dst[0],value);}
 assert_eq!(dst[1],initial[1]);assert_eq!(dst[2],initial[2]);
}
fn upper_nibble(c:u8)->Option<u8>{
 if c>=b'0'&&c<=b'9'{Some(c-b'0')}else if c>=b'A'&&c<=b'F'{Some(c-b'A'+10)}else{None}
}
fn mixed_nibble(c:u8)->Option<u8>{match lower_nibble(c){Some(n)=>Some(n),None=>upper_nibble(c)}}

#[kani::proof]
#[kani::unwind(8)]
fn upper_pair_decode_all_bytes(){
 let input:[u8;2]=kani::any();let initial:[u8;3]=kani::any();let mut dst=initial;
 let expected=match (upper_nibble(input[0]),upper_nibble(input[1])){(Some(h),Some(l))=>Some(h*16+l),_=>None};
 let result=base16ct::upper::decode(&input[..],&mut dst);
 match (result,expected){
  (Ok(output),Some(value))=>{assert_eq!(output.len(),1);assert_eq!(output[0],value);}
  (Err(base16ct::Error::InvalidEncoding),None)=>{}
  _=>assert!(false,"exact success/error classification"),
 }
 if let Some(value)=expected {assert_eq!(dst[0],value);}
 assert_eq!(dst[1],initial[1]);assert_eq!(dst[2],initial[2]);
}

#[kani::proof]
#[kani::unwind(8)]
fn mixed_pair_decode_all_bytes(){
 let input:[u8;2]=kani::any();let initial:[u8;3]=kani::any();let mut dst=initial;
 let expected=match (mixed_nibble(input[0]),mixed_nibble(input[1])){(Some(h),Some(l))=>Some(h*16+l),_=>None};
 let result=base16ct::mixed::decode(&input[..],&mut dst);
 match (result,expected){
  (Ok(output),Some(value))=>{assert_eq!(output.len(),1);assert_eq!(output[0],value);}
  (Err(base16ct::Error::InvalidEncoding),None)=>{}
  _=>assert!(false,"exact success/error classification"),
 }
 if let Some(value)=expected {assert_eq!(dst[0],value);}
 assert_eq!(dst[1],initial[1]);assert_eq!(dst[2],initial[2]);
}

#[kani::proof]
#[kani::unwind(8)]
fn lower_byte_encode_and_roundtrip(){
 let value:u8=kani::any();let initial:[u8;4]=kani::any();let mut dst=initial;
 let digit=|n:u8|if n<10 {b'0'+n}else{b'a'+(n-10)};
 let expected=[digit(value/16),digit(value%16)];
 let input=[value];
 let output=base16ct::lower::encode(&input,&mut dst).expect("sufficient destination");
 assert_eq!(output.len(),2);assert_eq!(output[0],expected[0]);assert_eq!(output[1],expected[1]);
 let mut back=[0u8;1];
 let decoded=base16ct::lower::decode(output,&mut back).expect("generated canonical hex");
 assert_eq!(decoded.len(),1);assert_eq!(decoded[0],value);
 assert_eq!(dst[0],expected[0]);assert_eq!(dst[1],expected[1]);assert_eq!(dst[2],initial[2]);assert_eq!(dst[3],initial[3]);
}

#[kani::proof]
#[kani::unwind(8)]
fn upper_byte_encode_and_roundtrip(){
 let value:u8=kani::any();let initial:[u8;4]=kani::any();let mut dst=initial;
 let digit=|n:u8|if n<10 {b'0'+n}else{b'A'+(n-10)};
 let expected=[digit(value/16),digit(value%16)];
 let input=[value];
 let output=base16ct::upper::encode(&input,&mut dst).expect("sufficient destination");
 assert_eq!(output.len(),2);assert_eq!(output[0],expected[0]);assert_eq!(output[1],expected[1]);
 let mut back=[0u8;1];
 let decoded=base16ct::upper::decode(output,&mut back).expect("generated canonical hex");
 assert_eq!(decoded.len(),1);assert_eq!(decoded[0],value);
 assert_eq!(dst[0],expected[0]);assert_eq!(dst[1],expected[1]);assert_eq!(dst[2],initial[2]);assert_eq!(dst[3],initial[3]);
}

}
