import os,pathlib,hashlib,json,re
root=pathlib.Path(__file__).resolve().parents[3]; out=pathlib.Path(os.environ['OUT_DIR']); gen=pathlib.Path('generated');gen.mkdir(exist_ok=True)
s=(root/'src/bytes.rs').read_text(); rows={}
def span(name):
 assert s.count('// ORIGINAL_SHARED_BEGIN '+name+'\n') == 1, ('duplicate/missing begin',name)
 assert s.count('// ORIGINAL_SHARED_END '+name+'\n') == 1, ('duplicate/missing end',name)
 a=s.index('// ORIGINAL_SHARED_BEGIN '+name);b=s.index('// ORIGINAL_SHARED_END '+name,a)+len('// ORIGINAL_SHARED_END '+name)
 t=s[a:b];rows[name]={'sha256':hashlib.sha256(t.encode()).hexdigest(),'line':s[:a].count('\n')+1};return t
records=''
for name in ['bytes_record.rs','vtable_record.rs']:
 p=root/'src/bytes'/name;t=p.read_text();rows[name]={'sha256':hashlib.sha256(t.encode()).hexdigest()};records+=t+'\n'
# Preserve the mutable Shared namespace as well as the BytesMut record.
m=(root/'src/bytes_mut.rs').read_text()
def mutable_record(start,name):
 a=m.index(start);b=m.index('\n}',a)+2;t=m[a:b]
 rows[name]={'sha256':hashlib.sha256(t.encode()).hexdigest(),'line':m[:a].count('\n')+1}
 return t
records+='mod mutable_record {\nuse alloc::vec::Vec;\nuse core::{ptr::NonNull,sync::atomic::AtomicUsize};\n'+mutable_record('struct Shared {','bytes_mut::Shared')+'\n'+mutable_record('pub struct BytesMut {','bytes_mut::BytesMut')+'\n}\nuse mutable_record::BytesMut;\n'
impls=span('bytes_clone_impl')+'\n'+span('bytes_from_vec_impl')+'\nimpl Bytes {\n'+span('bytes_cleanup')+'\n'+span('bytes_as_slice')+'\n}\n'+span('bytes_as_ref_impl')+'\n'
for name,t in [('public_records.rs',records),('public_traits.rs',impls)]:
 (out/name).write_text(t);(gen/name).write_text(t);rows['generated/'+name]={'sha256':hashlib.sha256(t.encode()).hexdigest()}
native_names=['shared_vtable','shared_table_native','shared_clone','shallow_clone_arc','shared_drop','release_shared','free_shared']
native={name:span(name) for name in native_names}
# Closed-source admission checks: a hash records the input; these checks also
# reject changing a selected table entry or severing its native helper chain.
entries=dict(re.findall(r'\b(clone|into_vec|into_mut|is_unique|drop)\s*:\s*(\w+)\s*,',native['shared_vtable']))
assert entries == {'clone':'shared_clone','into_vec':'shared_to_vec','into_mut':'shared_to_mut','is_unique':'shared_is_unique','drop':'shared_drop'},entries
assert re.search(r"fn original_shared_table_native\(\)\s*->\s*&'static Vtable\s*\{\s*&SHARED_VTABLE\s*\}",native['shared_table_native'])
assert 'shallow_clone_arc(shared as _, ptr, len)' in native['shared_clone']
assert 'crate::ref_count_ops::increment(&(*shared).ref_cnt)' in native['shallow_clone_arc']
assert 'release_shared(shared.cast())' in native['shared_drop']
assert 'ref_cnt.fetch_sub(1, Ordering::Release)' in native['release_shared']
assert 'ref_cnt.load(Ordering::Acquire)' in native['release_shared']
assert 'free_shared(ptr)' in native['release_shared']
helper=(root/'src/ref_count_ops.rs').read_text()
assert 'try_increment(counter).is_err()' in helper
assert re.search(r'fetch_update\(\s*Ordering::Relaxed,\s*Ordering::Relaxed,\s*crate::ref_count_limit::next_ref_count,\s*\)',helper)
# Exact native declarations are emitted for erasure review in their original
# production namespace. They are not standalone substitutes for that namespace.
(gen/'native_bindings.rs').write_text('\n\n'.join(native.values())+'\n')
rows['closed_native_binding']={'entries':entries,'getter':'original_shared_table_native',
 'clone_shim':'shared_clone_checked','cleanup_shim':'shared_drop_checked',
 'status':'checked identifier/helper chain; instrumented-body erasure remains generic TCB'}

(gen/'source-map.json').write_text(json.dumps(rows,indent=2)+'\n')

# Exact original public view bodies: string/comment-aware balanced extraction.
def view_body(signature):
 a=s.index(signature); opening=s.index('{',a); depth=0; state='code'; escaped=False; i=opening
 while i<len(s):
  ch=s[i]; nxt=s[i:i+2]
  if state=='line':
   if ch=='\n': state='code'
  elif state=='block':
   if nxt=='*/': state='code';i+=1
  elif state=='string':
   if escaped: escaped=False
   elif ch=='\\': escaped=True
   elif ch=='"': state='code'
  elif nxt=='//': state='line';i+=1
  elif nxt=='/*': state='block';i+=1
  elif ch=='"': state='string'
  elif ch=='{': depth+=1
  elif ch=='}':
   depth-=1
   if depth==0: return s[a:i+1]
  i+=1
 raise AssertionError('unterminated view body')
views={name:view_body(sig) for name,sig in [
 ('slice','pub fn slice(&self, range: impl RangeBounds<usize>) -> Self'),
 ('new_empty_with_ptr','fn new_empty_with_ptr(ptr: *const u8) -> Self'),
 ('static_clone','unsafe fn static_clone('),('static_drop','unsafe fn static_drop('),
 ('without_provenance','fn without_provenance(ptr: usize) -> *const u8')]}
(gen/'native_view_bindings.rs').write_text('\n\n'.join(views.values())+'\n')
rows['view_bodies']={name:hashlib.sha256(body.encode()).hexdigest() for name,body in views.items()}
(gen/'source-map.json').write_text(json.dumps(rows,indent=2)+'\n')

# Exact default-native cursor methods; selected cfg branch is independently MIR checked.
cursors={name:view_body(sig) for name,sig in [
 ('inc_start','unsafe fn inc_start(&mut self, by: usize)'),
 ('remaining','fn remaining(&self) -> usize'),('chunk','fn chunk(&self) -> &[u8]'),
 ('advance','fn advance(&mut self, cnt: usize)'),('len','pub const fn len(&self) -> usize')]}
(gen/'native_cursor_bindings.rs').write_text('\n\n'.join(cursors.values())+'\n')
rows['cursor_bodies']={name:hashlib.sha256(body.encode()).hexdigest() for name,body in cursors.items()}
(gen/'source-map.json').write_text(json.dumps(rows,indent=2)+'\n')
