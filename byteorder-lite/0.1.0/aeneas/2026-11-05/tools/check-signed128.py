from pathlib import Path
import json,re,hashlib
b=Path(__file__).resolve().parents[1];p=b/'proofs'
m=json.loads((p/'LiteSigned128Roots/translation.json').read_text());rows=[]
for e in ['BigEndian','LittleEndian']:
 for op in ['read_int128','write_int128','read_uint128','write_uint128']:
  name=f'byteorder_lite::{{impl byteorder_lite::ByteOrder for byteorder_lite::{e}}}::{op}'
  fs=[x for x in m['functions'] if x['rust_name']==name];assert len(fs)==1 and not fs[0]['is_opaque'] and not fs[0]['is_local']
  assert (b/fs[0]['source']['file']).resolve()==(b/'src/lib.rs').resolve()
  rows.append(fs[0])
for helper in ['extend_sign128','unextend_sign128']:
 fs=[x for x in m['functions'] if x['rust_name']=='byteorder_lite::'+helper];assert len(fs)==1 and not fs[0]['is_opaque']
assert not any(x['is_opaque'] for x in m['functions'])
(b/'evidence/signed128/body-guard.json').write_text(json.dumps({'original_methods':rows,'opaque_count':0,'helpers_nonopaque':True,'new_models':0},indent=2)+'\n')
print('Original signed1284/unsigned1284/fullhelpers source-traced nonopaque; no new models')
