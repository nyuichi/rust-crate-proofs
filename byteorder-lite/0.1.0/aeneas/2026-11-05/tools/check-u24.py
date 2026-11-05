from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
rows=json.loads((p/'proofs/ByteorderLiteU24Roots/translation.json').read_text())['functions']
for e in ['BigEndian','LittleEndian']:
 for method in ['read_u24','write_u24','read_uint','read_uint128','write_uint']:
  name=f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{e}}}::{method}'
  fs=[f for f in rows if f['rust_name']==name]
  assert len(fs)==1,(name,fs)
  f=fs[0];assert not f['is_opaque'] and not f['is_local'],f
  assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
  assert f['lean_name'].startswith('byteorder_lite_u24_roots.byteorder_lite.'),f
assert not any(f['is_opaque'] for f in rows)
print('Original concrete u24 defaults and full concrete callees are present, nonopaque and source-traced; no callback assumption or wrapper substitution')
