from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
rows=json.loads((p/'proofs/ByteorderU24Roots/translation.json').read_text())['functions']
for e in ['BigEndian','LittleEndian']:
 for method in ['read_u24','read_uint','read_uint128']:
  name=f'byteorder::{{impl byteorder::ByteOrder for byteorder::{e}}}::{method}'
  fs=[f for f in rows if f['rust_name']==name]
  assert len(fs)==1,(name,fs)
  f=fs[0];assert not f['is_opaque'] and not f['is_local'],f
  assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
  assert f['lean_name'].startswith('byteorder_u24_roots.byteorder.'),f
assert not any(f['is_opaque'] for f in rows)
print('Original concrete u24 read defaults and full concrete callees are present, nonopaque and source-traced; no callback assumption or wrapper substitution')

import re
root=(p/'proofs/ReadVariableGenerated/Byteorder.lean').read_text()
selected=(p/'proofs/ByteorderU24Roots/Funs.lean').read_text()
for e in ['BigEndian','LittleEndian']:
 a=re.search(r'def '+e+r'\.Insts\.ByteorderByteOrder\.read_uint\b',root)
 b=re.search(r'def byteorder\.'+e+r'\.Insts\.ByteorderByteOrder\.read_uint\b',selected)
 assert a and b
 def body(text,m):
  start=text.index(':= do',m.start());end=text.find('\n/--',start)
  return text[start:end]
 assert body(root,a)==body(selected,b),e
print('Both actual original read_uint bodies are text-identical; Lean U24Bodies proves their definitional equality, not an opaque alias')
