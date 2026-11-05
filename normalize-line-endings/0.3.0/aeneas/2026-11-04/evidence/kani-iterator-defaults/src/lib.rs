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
#[kani::proof]
#[kani::unwind(8)]
fn inherited_nth_all_indices(){
 let data:[char;4]=kani::any();let len:usize=kani::any();kani::assume(len<=4);
 let index:usize=kani::any();let (expected,count)=reference(data,len);
 let mut iter=normalized(Input{data,len,pos:0});
 let result=iter.nth(index);
 if index<count {
  assert_eq!(result,Some(expected[index]));
  let start=index+1;let remaining=count-start;let mut j=0;
  while j<remaining {assert_eq!(iter.next(),Some(expected[start+j]));j+=1;}
 } else {assert_eq!(result,None);}
 assert_eq!(iter.next(),None);assert_eq!(iter.next(),None);
}
#[kani::proof]
#[kani::unwind(8)]
fn inherited_count(){
 let data:[char;4]=kani::any();let len:usize=kani::any();kani::assume(len<=4);
 let (_,count)=reference(data,len);
 assert_eq!(normalized(Input{data,len,pos:0}).count(),count);
}
#[kani::proof]
#[kani::unwind(8)]
fn inherited_last(){
 let data:[char;4]=kani::any();let len:usize=kani::any();kani::assume(len<=4);
 let (expected,count)=reference(data,len);
 let actual=normalized(Input{data,len,pos:0}).last();
 if count==0 {assert_eq!(actual,None);}else{assert_eq!(actual,Some(expected[count-1]));}
}

}
