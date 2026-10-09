#!/usr/bin/env python3
"""Independent AY input-slot reconstruction and recipe/profile byte audit."""
import json,hashlib,re,collections
from pathlib import Path
repo=Path('/workspace/bytes-work');v=repo/'bytes/1.11.1/verification';p=v/'probes/original-root-phase-clone-2026-10-09';stage=Path('/workspace/work/az-origin-source-v1')
sha=lambda b:hashlib.sha256(b).hexdigest()
def get(path):return json.loads(path.read_bytes())
def require(cond,msg):
 if not cond:raise RuntimeError(msg)
reqpath=Path('/workspace/work/AZ_REQUIRED_INPUTS_v1.json');profilepath=stage/'metadata/AZ_INPUT_BINDINGS.json';recipepath=Path('/workspace/work/az-origin-recipe-v1.json')
require(sha(reqpath.read_bytes())=='aa02d915e100a72baaf4583f701d738ff1e2c8cc88621b0471b2db9d5325493d','required inventory pin');req=get(reqpath);expected={}

def add(destination,path):
 require(path.is_file() and not path.is_symlink(),'missing regular source '+str(path));raw=path.read_bytes();row=(sha(raw),len(raw),str(path.resolve()))
 require(destination not in expected or expected[destination]==row,'colliding independent slot '+destination);expected[destination]=row

def probe(relative):add('az/probe/'+relative,p/relative)
full=get(p/'evidence/AZ_FULL_PROOF_RUN.json')
require(sha((p/'evidence/AZ_FULL_PROOF_RUN.json').read_bytes())=='0a29aa31e5593200f18b15c2e66b9411e2c46faca0ed71f28ed88de48286a012','full origin authority changed')
# Reconstruct translated bodies/proofs from actual current directory, not profile claims.
actual=sorted((p/'verif').rglob('*.coma'));require(len(actual)==191,'translation inventory count')
proofslots=set()
for coma in actual:
 for path in [coma,coma.with_suffix('')/'proof.json']:
  rel=path.relative_to(p).as_posix();proofslots.add(rel);probe(rel);probe('evidence/full-origin-inputs-v1/proofs/'+rel)
require(proofslots=={r['path'] for r in full['proof_file_identity']},'full receipt omits actual proof bodies')
inputs={q.relative_to(p).as_posix() for q in (p/'src').glob('*.rs')}|{'Cargo.toml','Cargo.lock','build.rs','elaborate.py','run-proof.sh','extract_public.py','why3find.json','inherited-targets.json','AZ_REVIEWED_SOURCE.json','generated/mapping.json','generated/proof-targets.json'}
require(len(inputs)==37 and inputs=={r['path'] for r in full['probe_input_identity']},'full source/input inventory not exact independently required set')
for rel in inputs:probe(rel);probe('evidence/full-origin-inputs-v1/inputs/'+rel)
for q in (p/'evidence/full-origin-inputs-v1').rglob('*'):
 if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['AZ_FULL_PROOF_RUN.json','full-diagnostic-v1.log']:probe('evidence/'+name)
for directory in ['new-bodies-diagnostic-v2','frontend-diagnostic-v1']:
 for q in (p/'evidence'/directory).rglob('*'):
  if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['AZ_SCAFFOLD.json','AZ_REVIEWED_SOURCE.json','check_correspondence.py','check_native.py','prepare_capture.py','native.rs','capture-native.sh','native-field-profile.rs','native-field-profile.log','reviewed-production-inputs.json','TASK_PRINTING.md','print_proof_task.ml']:probe(name)
for name in ['compiled-capture-summary.json','correspondence.json','mapping.json','proof-targets.json','public_records.rs','public_traits.rs','source-map.json','native_bindings.rs','native_view_bindings.rs','native_cursor_bindings.rs']:probe('generated/'+name)
for q in (p/'generated/compiled-inputs').iterdir():
 if q.is_file():probe(q.relative_to(p).as_posix())
for q in (p/'native-mir').iterdir():
 if q.is_file():probe(q.relative_to(p).as_posix())
for name in ['Cargo.toml','Cargo.lock','native-run.log','tests/root_clone_witness.rs']:probe('native-test/'+name)
review=get(p/'reviewed-production-inputs.json')
for rel,digest in review['files'].items():
 q=repo/'bytes/1.11.1'/rel;require(sha(q.read_bytes())==digest,'reviewed production byte mismatch');add('az/repository/bytes/1.11.1/'+rel,q)
add('az/repository/bytes/1.11.1/Cargo.toml.orig',repo/'bytes/1.11.1/Cargo.toml.orig')
for r in full['external_input_identity']:add('az/'+r['snapshot_path'],Path(r['path']))
# Resolve real source module graph, independently from profile destinations.
seen=set()
def imports(q):
 q=q.resolve();require(q.is_relative_to(repo),'module source escapes repository')
 if q in seen:return
 seen.add(q);add('az/repository/'+q.relative_to(repo).as_posix(),q)
 for literal in re.findall(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+\w+\s*;',q.read_text()):imports(q.parent/literal)
for q in (p/'src').glob('*.rs'):imports(q)
for rel in ['tools/evidence_capture.py','tools/evidence_closure.py','tools/test_evidence_capture.py','tools/test_evidence_closure.py','ARCHITECTURE_DECISIONS.md','ARCHITECTURE_ASSESSMENT.md']:add('az/verification/'+rel,v/rel)
for name in ['az_capture_full.py','az_capture_selected.py','az_derive_capture_inventory.py']:add('az/evidence/producers/'+name,Path('/workspace/work')/name)
rows={r['destination']:r for r in req['entries']}
require(len(rows)==len(req['entries']) and set(expected)==set(rows),'independent completeness mismatch '+str({'missing':sorted(set(expected)-set(rows)),'extra':sorted(set(rows)-set(expected))}))
for dest,(digest,size,source) in expected.items():
 r=rows[dest];require((r['sha256'],r['size'],r['source'])==(digest,size,source),'current frozen input byte mismatch '+dest)
for r in req['external_binaries']:
 q=Path(r['path']);require(q.is_file() and not q.is_symlink() and (sha(q.read_bytes()),q.stat().st_size)==(r['sha256'],r['size']) and r['payload_bundled'] is False,'external executable mismatch')
report={'status':'PASS','scope':'complete AZ frozen current input inventory; profile/closed graph pending','required_slots':len(expected),'proof_outputs':len(proofslots),'full_source_inputs':len(inputs),'environment_inputs':len(full['external_input_identity']),'distinct_module_graph_nodes':len(seen),'external_executable_hashes_verified':len(req['external_binaries']),'required_inventory_sha256':sha(reqpath.read_bytes()),'schema':'az-required-input-inventory-v1','normal_checker_reexecuted':False,'prover_reexecuted':False,'full_original_admitted':False}
Path('/workspace/work/AZ_ORIGIN_INVENTORY_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
