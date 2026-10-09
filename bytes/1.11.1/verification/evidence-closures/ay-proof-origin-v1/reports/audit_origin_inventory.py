#!/usr/bin/env python3
"""Independent AY input-slot reconstruction and recipe/profile byte audit."""
import json,hashlib,re,collections
from pathlib import Path
repo=Path('/workspace/bytes-work');v=repo/'bytes/1.11.1/verification';p=v/'probes/original-root-phase-view-2026-10-09';stage=Path('/workspace/work/ay-origin-source-v1')
sha=lambda b:hashlib.sha256(b).hexdigest()
def get(path):return json.loads(path.read_bytes())
def require(cond,msg):
 if not cond:raise RuntimeError(msg)
reqpath=Path('/workspace/work/AY_REQUIRED_INPUTS_v1.json');profilepath=stage/'metadata/AY_INPUT_BINDINGS.json';recipepath=Path('/workspace/work/ay-origin-recipe-v1.json')
for path,pin in [(reqpath,'897ac71f0b1447ab96871a852afd162294a9b3c849edf6861ac5e87d915e755b'),(profilepath,'3a7f130358beac0e374a4093a4b52996675ae94beecf7cdf7ae6c4f80b3b6e21'),(recipepath,'a0318df3420d964dc398e2606015bd68c9e7b79e9ae01191ad55ebd3d2f121aa')]:require(sha(path.read_bytes())==pin,'frozen authority changed '+str(path))
req=get(reqpath);profile=get(profilepath);recipe=get(recipepath);expected={}
def add(destination,path):
 require(path.is_file() and not path.is_symlink(),'missing regular source '+str(path));raw=path.read_bytes();row=(sha(raw),len(raw),str(path.resolve()))
 require(destination not in expected or expected[destination]==row,'colliding independent slot '+destination);expected[destination]=row

def probe(relative):add('ay/probe/'+relative,p/relative)
full=get(p/'evidence/AY_FULL_PROOF_RUN.json')
require(sha((p/'evidence/AY_FULL_PROOF_RUN.json').read_bytes())=='c10232a3db94caa520eeabf25b9be1387c71fb1e41cd17cab5aecf27d09b3dc4','full origin authority changed')
# Reconstruct translated bodies/proofs from actual current directory, not profile claims.
actual=sorted((p/'verif').rglob('*.coma'));require(len(actual)==186,'translation inventory count')
proofslots=set()
for coma in actual:
 for path in [coma,coma.with_suffix('')/'proof.json']:
  rel=path.relative_to(p).as_posix();proofslots.add(rel);probe(rel);probe('evidence/full-origin-inputs-v1/proofs/'+rel)
require(proofslots=={r['path'] for r in full['proof_file_identity']},'full receipt omits actual proof bodies')
inputs={q.relative_to(p).as_posix() for q in (p/'src').glob('*.rs')}|{'Cargo.toml','Cargo.lock','build.rs','elaborate.py','run-proof.sh','extract_public.py','why3find.json','inherited-targets.json','AY_REVIEWED_SOURCE.json','generated/mapping.json','generated/proof-targets.json'}
require(len(inputs)==36 and inputs=={r['path'] for r in full['probe_input_identity']},'full source/input inventory not exact independently required set')
for rel in inputs:probe(rel);probe('evidence/full-origin-inputs-v1/inputs/'+rel)
for q in (p/'evidence/full-origin-inputs-v1').rglob('*'):
 if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['AY_FULL_PROOF_RUN.json','full-diagnostic-v1.log']:probe('evidence/'+name)
for directory in ['new-bodies-diagnostic-v1b','affected-bodies-diagnostic-v2','peer-drop-control-v1']:
 for q in (p/'evidence'/directory).rglob('*'):
  if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['AY_SCAFFOLD.json','AY_REVIEWED_SOURCE.json','check_correspondence.py','check_native.py','prepare_capture.py','native.rs','capture-native.sh','native-field-profile.rs','native-field-profile.log','reviewed-production-inputs.json','TASK_PRINTING.md','print_proof_task.ml']:probe(name)
for name in ['compiled-capture-summary.json','correspondence.json','mapping.json','proof-targets.json','public_records.rs','public_traits.rs','source-map.json','native_bindings.rs','native_view_bindings.rs','native_cursor_bindings.rs']:probe('generated/'+name)
for q in (p/'generated/compiled-inputs').iterdir():
 if q.is_file():probe(q.relative_to(p).as_posix())
for q in (p/'native-mir').iterdir():
 if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['Cargo.toml','Cargo.lock','native-run.log','tests/root_phase_witness.rs']:probe('native-test/'+name)
review=get(p/'reviewed-production-inputs.json')
for rel,digest in review['files'].items():
 q=repo/'bytes/1.11.1'/rel;require(sha(q.read_bytes())==digest,'reviewed production byte mismatch');add('ay/repository/bytes/1.11.1/'+rel,q)
add('ay/repository/bytes/1.11.1/Cargo.toml.orig',repo/'bytes/1.11.1/Cargo.toml.orig')
for r in full['external_input_identity']:add('ay/'+r['snapshot_path'],Path(r['path']))
# Resolve real source module graph, independently from profile destinations.
seen=set()
def imports(q):
 q=q.resolve();require(q.is_relative_to(repo),'module source escapes repository')
 if q in seen:return
 seen.add(q);add('ay/repository/'+q.relative_to(repo).as_posix(),q)
 for literal in re.findall(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+\w+\s*;',q.read_text()):imports(q.parent/literal)
for q in (p/'src').glob('*.rs'):imports(q)
for rel in ['tools/evidence_capture.py','tools/evidence_closure.py','tools/test_evidence_capture.py','tools/test_evidence_closure.py','ARCHITECTURE_DECISIONS.md','ARCHITECTURE_ASSESSMENT.md']:add('ay/verification/'+rel,v/rel)
for name in ['ay_capture_full.py','ay_capture_selected.py','ay_capture_selected_focused.py','ay_derive_capture_inventory.py']:add('ay/evidence/producers/'+name,Path('/workspace/work')/name)
rows={r['destination']:r for r in req['entries']};bindings={r['destination']:r for r in profile['bindings']}
require(len(rows)==len(req['entries']) and len(bindings)==len(profile['bindings']),'duplicate profile destination')
require(set(expected)==set(rows)==set(bindings),'independent completeness mismatch: '+str({'missing':sorted(set(expected)-set(rows)),'extra':sorted(set(rows)-set(expected))}))
axpath=v/'evidence-closures/ax-admitted-reuse-v1/manifest.json';require(sha(axpath.read_bytes())==profile['ancestor_raw_manifest_sha256']=='05cca18b78627f28340e01f86abc52c7e75669734f1143389b832d7a1db5e8a5','AX ancestor pin mismatch')
ax=get(axpath);axfiles={r['path']:r for r in ax['files']};sources={r['id']:r for r in recipe['sources']};mounts={r['path_prefix']:r for r in recipe['mounts']}
documentation_drift=[]
for dest,(digest,size,source) in expected.items():
 if dest in ('ay/verification/ARCHITECTURE_ASSESSMENT.md','ay/verification/ARCHITECTURE_DECISIONS.md') and (rows[dest]['sha256'],rows[dest]['size'])!=(digest,size):
  documentation_drift.append({'path':dest,'frozen_sha256':rows[dest]['sha256'],'current_sha256':digest});digest,size=rows[dest]['sha256'],rows[dest]['size']
 r=rows[dest];b=bindings[dest];require((r['sha256'],r['size'],r['source'])==(digest,size,source),'required source identity mismatch '+dest);require((b['sha256'],b['size'])==(digest,size),'binding identity mismatch '+dest)
 if b['resolution']=='local':
  rel=Path(b['source_path']);require(not rel.is_absolute() and '..' not in rel.parts,'unsafe stage path');q=stage/rel;require(q.is_file() and not q.is_symlink() and (sha(q.read_bytes()),q.stat().st_size)==(digest,size),'local staging mismatch '+dest)
  require(dest in mounts and sources[mounts[dest]['source_id']]['path']==b['source_path'],'local recipe omission '+dest)
 else:
  require(b['resolution']=='ancestor' and b['source_path'].startswith('ancestors/ax/'),'unknown binding source');orig=b['source_path'][len('ancestors/ax/'):];require(orig in axfiles and (axfiles[orig]['sha256'],axfiles[orig]['size'])==(digest,size),'ancestor alias does not match exact slot '+dest)
require(profile['required_inventory_sha256']==sha(reqpath.read_bytes()),'required inventory not bound')
imports_decl=recipe['imports'];require(len(imports_decl)==1 and imports_decl[0]['manifest_sha256']==profile['ancestor_raw_manifest_sha256'],'wrong import authority')
external=[]
for r in req['external_binaries']:
 q=Path(r['path']);require(q.is_file() and not q.is_symlink() and sha(q.read_bytes())==r['sha256'] and q.stat().st_size==r['size'] and r['payload_bundled'] is False,'external executable identity mismatch');external.append(r['name'])
report={'status':'pass','scope':'independent current-input completeness and frozen recipe/profile staging; closed object graph audit pending external root pin','required_slots':len(expected),'proof_outputs':len(proofslots),'full_source_inputs':len(inputs),'environment_inputs':len(full['external_input_identity']),'module_sources':len(seen),'resolutions':dict(collections.Counter(r['resolution'] for r in bindings.values())),'required_inventory_sha256':sha(reqpath.read_bytes()),'binding_profile_sha256':sha(profilepath.read_bytes()),'recipe_sha256':sha(recipepath.read_bytes()),'external_binaries_verified':external,'external_payloads_bundled':False,'postfreeze_documentation_drift':documentation_drift,'normal_checker_reexecuted':False,'prover_reexecuted':False,'diagnostic_policy_exit':2,'full_original_admitted':False}
Path('/workspace/work/ay-audit/AY_ORIGIN_INVENTORY_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
