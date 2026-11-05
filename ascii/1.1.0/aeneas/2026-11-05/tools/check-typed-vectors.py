from pathlib import Path
import json
p=Path(__file__).resolve().parents[1]
packet=json.loads((p/'evidence/typed-vector-bindings/binding-packet.json').read_text())
m=json.loads((p/'proofs/Generated/translation.json').read_text())
actual=[]
for row in packet['rows']:
 rust=row['actual_extracted_original']['rust_name']
 fs=[x for x in m['functions'] if x['rust_name']==rust]
 assert len(fs)==1 and fs[0]['is_local'] and not fs[0]['is_opaque']
 assert (p/fs[0]['source']['file']).resolve()==(p/'src/ascii_string.rs').resolve()
 actual.append(fs[0])
assert not any(x['is_opaque'] for x in m['functions'])
assert not any('Vec<u8>' in x['rust_name'] and ('IntoAsciiString' in x['rust_name'] or 'From<ascii::ascii_string::AsciiString>' in x['rust_name']) for x in m['functions'])
(p/'evidence/typed-vector-body-guard.json').write_text(json.dumps({'original_manual_typed_methods':actual,'opaque_count':0,'wrong_vec_u8_impls':0,'new_models':0,'publication_gate':'exact typed source binding still required'},indent=2)+'\n')
print('Original typedvector4 exact source identities only; opaque0/wrongVecu8body0/newmodels0')
