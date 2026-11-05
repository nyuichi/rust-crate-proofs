#[cfg(kani)]
mod checks {
use normalize_line_endings::normalized;
struct Input{data:[char;4],len:usize,pos:usize}
impl Iterator for Input{
 type Item=char;
 fn next(&mut self)->Option<char>{if self.pos<self.len {let c=self.data[self.pos];self.pos+=1;Some(c)}else{None}}
}
fn reference(data:[char;4],len:usize)->([char;4],usize){
 let mut out=['\0';4];let mut count=0;let mut i=0;
 while i<len {
  let c=data[i];
  if c!='\n'||i==0||data[i-1]!='\r'{out[count]=if c=='\r'{'\n'}else{c};count+=1;}
  i+=1;
 }
 (out,count)
}
fn locate(expected:[char;4],count:usize,target:char)->Option<usize>{
 let mut i=0;while i<count {if expected[i]==target{return Some(i)}i+=1;}None
}
#[kani::proof]
#[kani::unwind(8)]
fn position_character_and_remainder(){
 let data:[char;4]=kani::any();let len:usize=kani::any();kani::assume(len<=4);
 let target:char=kani::any();let (expected,count)=reference(data,len);let found=locate(expected,count,target);
 let mut iter=normalized(Input{data,len,pos:0});let actual=iter.position(|c|c==target);
 assert_eq!(actual,found);
 if let Some(index)=found {let start=index+1;let mut j=start;while j<count {assert_eq!(iter.next(),Some(expected[j]));j+=1;}}
 assert_eq!(iter.next(),None);assert_eq!(iter.next(),None);
}
#[kani::proof]
#[kani::unwind(8)]
fn find_character_and_remainder(){
 let data:[char;4]=kani::any();let len:usize=kani::any();kani::assume(len<=4);
 let target:char=kani::any();let (expected,count)=reference(data,len);let found=locate(expected,count,target);
 let mut iter=normalized(Input{data,len,pos:0});let actual=iter.find(|c|*c==target);
 if let Some(index)=found {
  assert_eq!(actual,Some(expected[index]));
  let start=index+1;let mut j=start;while j<count {assert_eq!(iter.next(),Some(expected[j]));j+=1;}
 }else{assert_eq!(actual,None);}
 assert_eq!(iter.next(),None);assert_eq!(iter.next(),None);
}

}
