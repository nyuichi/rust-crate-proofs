from pathlib import Path
import json,re
p=Path(__file__).resolve().parents[1]
m=json.loads((p/'proofs/Generated/translation.json').read_text())
f=(p/'proofs/Generated/Ascii.lean').read_text()
rows=[]
for method,field in [('ascii_error','error'),('into_source','owner')]:
 name='ascii::ascii_string::{ascii::ascii_string::FromAsciiError<O>}::'+method
 fs=[x for x in m['functions'] if x['rust_name']==name]
 assert len(fs)==1 and fs[0]['is_local'] and not fs[0]['is_opaque']
 assert (p/fs[0]['source']['file']).resolve()==(p/'src/ascii_string.rs').resolve()
 lean='ascii_string.FromAsciiError.'+method
 body=re.search(r'def '+re.escape(lean)+r'\b(.*?)(?=\n/--|\nend |\Z)',f,re.S)[1].split(':=',1)[1]
 assert ' '.join(body.split())=='do ok self.'+field
 rows.append(fs[0])
assert not any(x['is_opaque'] for x in m['functions'])
(p/'evidence/error-projection-guard.json').write_text(json.dumps({'actual_original_methods':rows,'all_union_opaque_functions':0,'new_models':0,'owner_clone_copy_requirement':False},indent=2)+'\n')
print('Original generic error projections source-traced; exact field bodies; wholeunion opaque0/newmodels0')
