from pathlib import Path
import json,re
p=Path(__file__).resolve().parents[1]
m=json.loads((p/'proofs/ByteorderReadPureRoots/translation.json').read_text())
f=(p/'proofs/ByteorderReadPureRoots/Funs.lean').read_text()
rows=[]
for e in ['BigEndian','LittleEndian']:
 for method in ['read_u48','read_int','read_i24','read_i48','read_uint']:
  name=f'byteorder::{{impl byteorder::ByteOrder for byteorder::{e}}}::{method}'
  fs=[x for x in m['functions'] if x['rust_name']==name]
  assert len(fs)==1 and not fs[0]['is_opaque'] and not fs[0]['is_local']
  assert (p/fs[0]['source']['file']).resolve()==(p/'src/lib.rs').resolve()
  rows.append(fs[0])
helper=[x for x in m['functions'] if x['rust_name']=='byteorder::extend_sign']
assert len(helper)==1 and not helper[0]['is_opaque']
assert not any(x['is_opaque'] for x in m['functions'])
for e in ['BigEndian','LittleEndian']:
 for method,call,count in [('read_u48','read_uint',6),('read_i48','read_int',6)]:
  name=f'byteorder.{e}.Insts.ByteorderByteOrder.{method}'
  body=re.search(r'def '+re.escape(name)+r'\b(.*?)(?=\n/--|\nend |\Z)',f,re.S)[1].split(':=',1)[1]
  assert ' '.join(body.split())==f'do byteorder.{e}.Insts.ByteorderByteOrder.{call} buf {count}#usize'
(p/'evidence/read-pure/body-guard.json').write_text(json.dumps({'original_methods':rows,'helper':helper,'opaque_functions':0,'new_models':0},indent=2)+'\n')
print('Original 8 default reader bodies/2 full read_uint/extend_sign source-traced and nonopaque; count6 body guards passed')
