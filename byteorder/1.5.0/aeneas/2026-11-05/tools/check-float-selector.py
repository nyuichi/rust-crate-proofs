from pathlib import Path
import json,re
p=Path(__file__).resolve().parents[1]
fs=json.loads((p/'proofs/ByteorderFloatRoots/translation.json').read_text())['functions']
std={'core::f32::{f32}::to_bits','core::f64::{f64}::to_bits'}
assert {f['rust_name'] for f in fs if f['is_opaque']}==std
text=(p/'proofs/ByteorderFloatRoots/Funs.lean').read_text()
for endian in ['BigEndian','LittleEndian']:
 for sign in ['u','f']:
  for bits in [32,64]:
   name=f'byteorder::{{impl byteorder::ByteOrder for byteorder::{endian}}}::write_{sign}{bits}'
   rows=[f for f in fs if f['rust_name']==name];assert len(rows)==1,(name,rows)
   f=rows[0];assert not f['is_opaque'] and not f['is_local'],f
   assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
   if sign=='f':
    lean=f['lean_name'].removeprefix('byteorder_float_roots.')
    body=text.split('def '+lean+'\n')[1].split('\n/--')[0].split('\nend ')[0]
    body=body.split(':= do')[1]
    expected=f'let i ← core.f{bits}.F{bits}.to_bits n byteorder.{endian}.Insts.ByteorderByteOrder.write_u{bits} buf i'
    assert ' '.join(body.split())==expected,(name,body)
print('Four original float and four unsigned leaf bodies are non-opaque/source-traced; each float body only converts bits then calls its unsigned writer')
