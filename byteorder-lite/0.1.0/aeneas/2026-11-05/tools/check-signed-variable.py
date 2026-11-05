from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
rows=json.loads((p/'proofs/SignedVariableRoots/translation.json').read_text())['functions']
for e in ['BigEndian','LittleEndian']:
 for method in ['write_i24','write_int','write_uint']:
  name=f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{e}}}::{method}'
  fs=[f for f in rows if f['rust_name']==name]
  assert len(fs)==1,(name,fs)
  f=fs[0];assert not f['is_opaque'] and not f['is_local'],f
  assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
  assert f['lean_name'].startswith('signed_variable_roots.byteorder_lite.'),f
assert not any(f['is_opaque'] for f in rows)
for helper in ['unextend_sign','pack_size']:
 fs=[f for f in rows if f['rust_name']=='byteorder_lite::'+helper]
 assert len(fs)==1 and not fs[0]['is_opaque'] and not fs[0]['is_local']
 assert (p/fs[0]['source']['file']).resolve()==(p/'src/lib.rs').resolve()
print('Original concrete signed variable defaults and full concrete callees are present, nonopaque and source-traced; no callback assumption or wrapper substitution')
