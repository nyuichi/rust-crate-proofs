from pathlib import Path
import json,re,hashlib
b=Path(__file__).resolve().parents[1];p=b/'proofs'
m=json.loads((p/'ByteorderRead128Roots/translation.json').read_text());records=[]
for e in ['BigEndian','LittleEndian']:
 for op in ['read_int128','read_uint128']:
  name=f'byteorder::{{impl byteorder::ByteOrder for byteorder::{e}}}::{op}'
  rows=[x for x in m['functions'] if x['rust_name']==name];assert len(rows)==1 and not rows[0]['is_opaque'] and not rows[0]['is_local']
  assert (b/rows[0]['source']['file']).resolve()==(b/'src/lib.rs').resolve();records+=rows
h=[x for x in m['functions'] if x['rust_name']=='byteorder::extend_sign128'];assert len(h)==1 and not h[0]['is_opaque'];assert not any(x['is_opaque'] for x in m['functions'])
(b/'evidence/read128/body-guard.json').write_text(json.dumps({'actual_methods':records,'actual_extend_sign128':h,'opaque_count':0,'new_models':0},indent=2)+'\n')
print('Actual originalread1282/uint1282/extend_sign128 nonopaque source-traced; no newmodels')
