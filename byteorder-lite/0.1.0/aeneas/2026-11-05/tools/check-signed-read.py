from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
rows=json.loads((p/'proofs/SignedReadRoots/translation.json').read_text())['functions']
for e in ['BigEndian','LittleEndian']:
 for method in ['read_i24','read_int','read_uint','read_uint128']:
  name=f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{e}}}::{method}'
  fs=[f for f in rows if f['rust_name']==name]
  assert len(fs)==1,(name,fs)
  f=fs[0];assert not f['is_opaque'] and not f['is_local'],f
  assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
  assert f['lean_name'].startswith('signed_read_roots.byteorder_lite.'),f
assert not any(f['is_opaque'] for f in rows)
for helper in ['extend_sign']:
 fs=[f for f in rows if f['rust_name']=='byteorder_lite::'+helper]
 assert len(fs)==1 and not fs[0]['is_opaque'] and not fs[0]['is_local']
 assert (p/fs[0]['source']['file']).resolve()==(p/'src/lib.rs').resolve()
print('Original concrete signed read defaults and full concrete callees are present, nonopaque and source-traced; no callback assumption or wrapper substitution')
