from pathlib import Path
import json,hashlib,re
b=Path(__file__).resolve().parents[2];p=b/'byteorder';l=b/'byteorder-lite'
for source,frozen in [(p/'source',p/'candidate-native-identity'),(l/'source',l/'candidate-u48')]:
 for name in ['Cargo.toml','src/lib.rs']:assert (source/name).read_bytes()==(frozen/name).read_bytes()
new=json.loads((p/'read-pure-proofs/ByteorderReadPureRoots/translation.json').read_text())
old48=json.loads((l/'candidate-u48/proofs/ByteorderLiteU48Roots/translation.json').read_text())
oldsign=json.loads((l/'candidate-u48/proofs/SignedReadRoots/translation.json').read_text())
rows=[]
for e in ['BigEndian','LittleEndian']:
 for op in ['read_u48','read_int','read_i24','read_i48','read_uint']:
  name=f'byteorder::{{impl byteorder::ByteOrder for byteorder::{e}}}::{op}'
  fs=[x for x in new['functions'] if x['rust_name']==name];assert len(fs)==1
  f=fs[0];assert not f['is_opaque'] and not f['is_local']
  assert Path(f['source']['file']).resolve()==(p/'source/src/lib.rs').resolve()
  old=old48 if op in ['read_u48','read_i48'] else oldsign
  oname=name.replace('byteorder::','byteorder_lite::')
  os=[x for x in old['functions'] if x['rust_name']==oname];assert len(os)==1 and not os[0]['is_opaque']
  assert (l/'candidate-u48'/os[0]['source']['file']).resolve()==(l/'candidate-u48/src/lib.rs').resolve()
  rows.append({'original':f,'verified_lite_reference':os[0]})
helper=[x for x in new['functions'] if x['rust_name']=='byteorder::extend_sign'];assert len(helper)==1 and not helper[0]['is_opaque']
assert not any(x['is_opaque'] for x in new['functions'])
log=(p/'read-pure-evidence/proof.log').read_text();assert 'error:' not in log
for end in ['be','le']:
 for op in ['read_u48','read_int','read_i24','read_i48']:
  assert f"'ReadPureProofs.{end}_{op}_contract' depends on axioms: [propext, Classical.choice, Quot.sound]" in log
files=[p/'source/src/lib.rs',p/'source/Cargo.toml',l/'source/src/lib.rs',l/'source/Cargo.toml',p/'read-pure-proofs/ByteorderReadPureRoots/Funs.lean',p/'read-pure-proofs/ReadPureProofs.lean',p/'read-pure-evidence/proof.log']
r={'status':'small-proof-success-not-frozen','new_mapping_count':8,'all8_standard3':True,'both_original_sources_manifests_match_frozen':True,'actual_original_and_reference_bodies':rows,'original_extend_sign_helper':helper[0],'actual_original_read_uint_helpers':2,'new_model_count':0,'source_adaptations':0,'frozen_candidates_untouched':True,'coimport_with_existing_byte32':'not yet tested; module naming separation required','file_sha256':{str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in files}}
(p/'read-pure-evidence/results.json').write_text(json.dumps(r,indent=2)+'\n')
print('8 actual original readers/helper dependencies + both source guards + rfl contracts standard3 checked')
