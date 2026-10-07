import pathlib,re,hashlib,json,os
root=pathlib.Path(__file__).resolve().parents[3]
source=root/'src/bytes_mut.rs'; text=source.read_text(); spans={}
def marked(name):
 a=text.index('// ORIGINAL_UNIQUE_BEGIN '+name); b=text.index('// ORIGINAL_UNIQUE_END '+name,a)+len('// ORIGINAL_UNIQUE_END '+name)
 out=text[a:b]; spans[name]={'sha256':hashlib.sha256(out.encode()).hexdigest(),'first_line':text[:a].count('\n')+1};return out
s='use crate::TryGetError;\nuse core::{mem::{ManuallyDrop,MaybeUninit},ptr::{self,NonNull},cmp,slice};\nuse alloc::vec::Vec;\nuse core::sync::atomic::AtomicUsize;\n'+marked('proof_imports')+'\n'
a=text.index('pub struct BytesMut {'); b=text.index('// Thread-safe reference-counted container',a)
s+=text[a:b]
a=text.index('struct Shared {'); b=text.index('\n}',a)+2;s+=text[a:b]+'\n'
lines=text.splitlines()
for i,line in enumerate(lines):
 if line.startswith('const ') and any(('const '+n) in line for n in ['KIND_','MAX_ORIGINAL_CAPACITY_WIDTH','MIN_ORIGINAL_CAPACITY_WIDTH','ORIGINAL_CAPACITY_MASK','ORIGINAL_CAPACITY_OFFSET','VEC_POS_OFFSET','MAX_VEC_POS','NOT_VEC_POS_MASK','PTR_WIDTH']):
  if lines[i-1].startswith('#[cfg('):s+=lines[i-1]+'\n'
  s+=line+'\n'
s+='impl BytesMut {\n'+ '\n'.join(marked(n) for n in ['with_capacity','len','capacity','reserve','extend','from_vec','spare','advance_mut'])+'\n'
s+='''#[cfg_attr(creusot, requires(false))]
fn reserve_inner(&mut self,_additional:usize,_allocate:bool)->bool { panic!("excluded growth") }
}\n'''
s+='\n'.join(marked(n) for n in ['capacity_repr','invalid_ptr'])+'\n'
s+=marked('vptr')+'\n'
# Actual selected native type returned live; no destructor is extracted.
s+='''
#[cfg(creusot)]
#[ensures(result.unique_valid())]
#[ensures(result.len@ == input@.len())]
#[ensures(forall<i:Int> 0 <= i && i < input@.len() ==> result.unique_slot(i) == Some(Some(input@[i])))]
pub fn allocate_and_append(input:&[u8]) -> BytesMut {
 let mut value=BytesMut::with_capacity(input.len());
 value.extend_from_slice(input);
 value
}
#[cfg(creusot)]
#[requires(input@.len() + suffix@.len() <= creusot_std::std::vec::capacity_model(input))]
#[ensures(result.unique_valid())]
#[ensures(result.len@ == input@.len() + suffix@.len())]
#[ensures(forall<i:Int> 0 <= i && i < input@.len() ==> result.unique_slot(i) == Some(Some(input@[i])))]
#[ensures(forall<i:Int> 0 <= i && i < suffix@.len() ==> result.unique_slot(input@.len()+i) == Some(Some(suffix@[i])))]
#[ensures(forall<i:Int> result.len@ <= i && i < result.cap@ ==> result.unique_slot(i) == Some(None))]
pub fn append_to_existing(input:Vec<u8>, suffix:&[u8]) -> BytesMut {
 let mut value=BytesMut::from_vec(input);
 value.extend_from_slice(suffix);
 value
}
#[cfg(all(creusot,feature="negative_uninitialized_publish"))]
#[ensures(result.unique_valid())]
pub fn negative_publish_unknown() -> BytesMut {
 let mut value=BytesMut::from_vec(Vec::with_capacity(1));
 // Intentionally omit every write; B1 says this slot is Unknown.
 unsafe { value.advance_mut(1); }
 value
}
#[cfg(all(test,not(creusot)))]
mod tests { use super::*;
#[test] fn exact_native_empty_and_nonempty() {
 for input in [&b""[..],&b"abc"[..],&b"longer contents"[..]] {
 let mut value=BytesMut::with_capacity(input.len()+8); value.extend_from_slice(input);
 assert_eq!(value.len,input.len());
 unsafe { assert_eq!(slice::from_raw_parts(value.ptr.as_ptr(),value.len),input);
 drop(Vec::from_raw_parts(value.ptr.as_ptr(),value.len,value.cap)); }
 }
}
}
'''
out=pathlib.Path(os.environ['OUT_DIR']);(out/'actual.rs').write_text(s)
spans['source']={'sha256':hashlib.sha256(source.read_bytes()).hexdigest()};spans['generated']={'sha256':hashlib.sha256(s.encode()).hexdigest()}
(out/'source-map.json').write_text(json.dumps(spans,indent=2))
pathlib.Path('generated').mkdir(exist_ok=True);pathlib.Path('generated/actual.rs').write_text(s);pathlib.Path('generated/source-map.json').write_text(json.dumps(spans,indent=2))
