import pathlib,os,hashlib,json
root=pathlib.Path(__file__).resolve().parents[3]; spans={}; out=pathlib.Path(os.environ['OUT_DIR']);gen=pathlib.Path('generated');gen.mkdir(exist_ok=True)
def source(name):
 p=root/'src'/name;s=p.read_text();spans['source/'+name]={'sha256':hashlib.sha256(p.read_bytes()).hexdigest()};return s
def marked(s,name,prefix='ORIGINAL_FREEZE'):
 a=s.index('// '+prefix+'_BEGIN '+name);b=s.index('// '+prefix+'_END '+name,a)+len('// '+prefix+'_END '+name);t=s[a:b];spans[name]={'sha256':hashlib.sha256(t.encode()).hexdigest(),'first_line':s[:a].count('\n')+1};return t
def struct(s,needle):
 a=s.index(needle);b=s.index('\n}',a)+2;return s[a:b]
def save(name,s):
 (out/name).write_text(s);(gen/name).write_text(s);spans['generated/'+name]={'sha256':hashlib.sha256(s.encode()).hexdigest()}
m=source('bytes_mut.rs');b=source('bytes.rs')
s='use core::{mem::{self,ManuallyDrop,MaybeUninit},ptr::{self,NonNull},cmp,slice};\nuse alloc::{vec::Vec,boxed::Box};\nuse crate::{Bytes,TryGetError};\nuse core::sync::atomic::AtomicUsize;\n'
s+=marked(m,'proof_imports','ORIGINAL_UNIQUE')+'\n'+struct(m,'pub struct BytesMut {')+'\n'+marked(m,'proof_model','ORIGINAL_UNIQUE')+'\n'+struct(m,'struct Shared {')+'\n'
lines=m.splitlines()
for i,line in enumerate(lines):
 if line.startswith('const ') and any(('const '+n) in line for n in ['KIND_','MAX_ORIGINAL_CAPACITY_WIDTH','MIN_ORIGINAL_CAPACITY_WIDTH','ORIGINAL_CAPACITY_MASK','ORIGINAL_CAPACITY_OFFSET','VEC_POS_OFFSET','MAX_VEC_POS','NOT_VEC_POS_MASK','PTR_WIDTH']):
  if lines[i-1].startswith('#[cfg('):s+=lines[i-1]+'\n'
  s+=line+'\n'
s+='impl BytesMut {\n'+'\n'.join(marked(m,n,'ORIGINAL_UNIQUE') for n in ['with_capacity','len','capacity','reserve','extend','from_vec','spare','advance_mut'])+'\n'
s+='\n'.join(marked(m,n) for n in ['kind','get_vec_pos','freeze'])+'\n'
s+='#[requires(false)] fn reserve_inner(&mut self,_n:usize,_a:bool)->bool {panic!("excluded growth")}\n}\n'
s+='\n'.join(marked(m,n,'ORIGINAL_UNIQUE') for n in ['capacity_repr','invalid_ptr','vptr'])+'\n'+marked(m,'rebuild_vec')+'\n';save('bytes_mut.rs',s)
s='use core::{mem::{self,ManuallyDrop},ptr,slice};\nuse alloc::{vec::Vec,boxed::Box};\nuse core::sync::atomic::{AtomicPtr,AtomicUsize};\nuse crate::BytesMut;\n'
s+=marked(b,'bytes_imports')+'\n'+struct(b,'pub struct Bytes {')+'\n'+struct(b,'pub(crate) struct Vtable {')+'\n'+struct(b,'struct Shared {')+'\n'+marked(b,'frozen_model')+'\n'
s+='const KIND_MASK:usize=0b1;\nimpl Bytes {\n'
for n in ['bytes_len','bytes_as_slice','bytes_inc_start','bytes_advance','bytes_from_vec']:
 t=marked(b,n)
 # Exact body preserved. Concrete method surface replaces open Buf/From dispatch.
 if n in ['bytes_advance','bytes_from_vec']:t=t.replace('    fn ', '    pub(crate) fn ',1)
 s+=t+'\n'
s+='}\n'
s+='''
#[requires(input@.len() < usize::MAX@)]
#[ensures(result.0.original_frozen_valid())]
#[ensures(result.0.original_frozen_bytes() == input@)]
#[ensures(result.1@ == input@.len())]
pub fn original_freeze_and_read(input:&[u8])->(Bytes,usize) {
 let mut value=BytesMut::with_capacity(input.len()+1);
 value.extend_from_slice(input);
 let frozen=value.freeze();
 let slice=frozen.as_slice();
 proof_assert!(slice@ == input@);
 let observed=slice.len();
 (frozen,observed)
}

#[requires(input@.len() < usize::MAX@)]
#[ensures(result.0.original_frozen_valid())]
#[ensures(result.0.original_frozen_bytes() == input@)]
#[ensures(result.1 == if input@.len() == 0 { None } else { Some(input@[0]) })]
pub fn original_freeze_and_load_byte(input:&[u8])->(Bytes,Option<u8>) {
 let mut value=BytesMut::with_capacity(input.len()+1);
 value.extend_from_slice(input);
 let frozen=value.freeze();
 let slice=frozen.as_slice();
 let observed=if slice.len()==0 { None } else { Some(slice[0]) };
 (frozen,observed)
}
#[cfg(feature="negative_retarget_read")]
#[requires(input@.len() < usize::MAX@)]
pub fn negative_retarget_read(input:&[u8])->Bytes {
 let mut value=BytesMut::with_capacity(input.len()+1);
 value.extend_from_slice(input);
 let mut frozen=value.freeze();
 // Deliberately retarget the native field without a matching physical capability.
 frozen.ptr=ptr::null();
 let _slice=frozen.as_slice();
 frozen
}
''';save('bytes.rs',s)
(out/'source-map.json').write_text(json.dumps(spans,indent=2));(gen/'source-map.json').write_text(json.dumps(spans,indent=2))
