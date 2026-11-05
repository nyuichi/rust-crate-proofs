from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
d=json.loads((p/'proofs/ByteorderLiteApiRoots/translation.json').read_text())
functions=d['functions']
for endian in ['BigEndian','LittleEndian']:
 for sign in ['u','i']:
  for bits in [16,32,64,128]:
   name=f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{endian}}}::write_{sign}{bits}'
   rows=[f for f in functions if f['rust_name']==name]
   assert len(rows)==1,(name,rows)
   f=rows[0];assert not f['is_opaque'] and not f['is_local'],f
   assert (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
   assert f['lean_name'].startswith('byteorder_lite_api_roots.byteorder_lite.'),f
print('All sixteen original dependency bodies are present, non-opaque, and traced to unchanged src/lib.rs; wrappers only select roots')
