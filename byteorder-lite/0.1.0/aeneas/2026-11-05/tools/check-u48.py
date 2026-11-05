from pathlib import Path
import json,hashlib,re
p=Path(__file__).resolve().parents[1]
src=p
s=(src/'src/lib.rs').read_text();f=(p/'proofs/ByteorderLiteU48Roots/Funs.lean').read_text()
meta=json.loads((p/'proofs/ByteorderLiteU48Roots/translation.json').read_text())
records=[]
for impl in ['BigEndian','LittleEndian']:
 for op,callee in [('read_u48','read_uint'),('write_u48','write_uint'),('read_i48','read_int'),('write_i48','write_int')]:
  m=re.search(r'fn '+op+r'\([^\n]+\) [^{]*\{([^}]+)\}',s)
  assert m and 'Self::'+callee+'(' in m[1] and ', 6)' in m[1]
  name='byteorder_lite.'+impl+'.Insts.Byteorder_liteByteOrder.'+op
  d=re.search(r'def '+re.escape(name)+r'\b(.*?)(?=\n/--|\nend |\Z)',f,re.S)
  assert d
  body=d[1].split(':=',1)[1]
  expected='do byteorder_lite.'+impl+'.Insts.Byteorder_liteByteOrder.'+callee+' buf'+(' n' if op.startswith('write') else '')+' 6#usize'
  assert ' '.join(body.split())==expected,(name,body)
  rows=[x for x in meta['functions'] if x['lean_name'].endswith('.'+name)]
  assert len(rows)==1 and not rows[0]['is_opaque'] and not rows[0]['is_local']
  assert (p/rows[0]['source']['file']).resolve()==(p/'src/lib.rs').resolve()
  records.append({'original_api':impl+'::'+op,'actual_generated_body':rows[0]['lean_name'],'static_callee':callee,'count':6,'source':rows[0]['source']})
assert not any(x['is_opaque'] for x in meta['functions'])
(p/'evidence/u48/body-guard.json').write_text(json.dumps({'records':records,'mapping_count':8,'opaque_count':0},indent=2)+'\n')
print('8 concrete original defaults: exact bodies, const6 static callees and original source traced')
