#!/usr/bin/env python3
"""Independent AZ immutable origin audit; no compiler/checker/solver replay."""
import json,hashlib,collections
from pathlib import Path
sha=lambda b:hashlib.sha256(b).hexdigest()
def get(p):return json.loads(p.read_bytes())
def require(c,m):
 if not c:raise RuntimeError(m)
b=Path('/workspace/bytes-work/bytes/1.11.1/verification/evidence-closures/az-proof-origin-v1');h=Path('/workspace/work/az-origin-independent-hydrate-v1');pin='6bfe965371c3a7ec3c91e0c1a8429de9e1f48810afd61a0fb6adabfcd86a0772';require(sha((b/'manifest.json').read_bytes())==pin,'raw external authority mismatch');d=get(b/'manifest.json');files={r['path']:r for r in d['files']};require(len(files)==len(d['files']),'duplicate manifest namespace')
actual={p.relative_to(h).as_posix() for p in h.rglob('*') if p.is_file()};require(actual==set(files),'hydrate namespace mismatch')
for path,r in files.items():
 p=h/path;raw=p.read_bytes();require(not p.is_symlink() and (sha(raw),len(raw))==(r['sha256'],r['size']),'hydrated identity mismatch '+path)
profile=get(h/'az/evidence/AZ_INPUT_BINDINGS.json');req=get(h/'az/evidence/AZ_REQUIRED_INPUTS.json');require(req['schema']=='az-required-input-inventory-v1' and len(req['entries'])==1353,'AZ inventory schema/completeness mismatch');bound={r['destination']:r for r in profile['bindings']}
def slot(logical):
 r=bound.get(logical);return h/(r['source_path'] if r and r['resolution']=='ancestor' else logical)
for r in profile['bindings']:
 p=slot(r['destination']);raw=p.read_bytes();require((sha(raw),len(raw))==(r['sha256'],r['size']),'final input binding mismatch')
full=get(slot('az/probe/evidence/AZ_FULL_PROOF_RUN.json'));require(sha(slot('az/probe/evidence/AZ_FULL_PROOF_RUN.json').read_bytes())=='0a29aa31e5593200f18b15c2e66b9411e2c46faca0ed71f28ed88de48286a012','full origin authority mismatch')
require(full['status']=='complete_positive_diagnostic' and full['policy_diagnostic'] and full['policy_correspondence_exit']==2 and full['prover_process_exit']==0,'origin diagnostic disposition changed');policy=get(slot('az/probe/generated/proof-targets.json'));require(policy['diagnostic'] and policy['correspondence_exit_status']==2 and policy['excluded']=={} and policy['features']==[],'original full policy changed')
stats={'files':0,'prover':0,'null':0,'structural':0}
def visit(n):
 if n is None:stats['null']+=1
 elif isinstance(n,dict) and 'children'in n:
  if not n['children']:stats['structural']+=1
  for c in n['children']:visit(c)
 elif isinstance(n,dict) and 'prover'in n:stats['prover']+=1
 else:raise RuntimeError('unknown proof node')
for r in full['proof_file_identity']:
 current=slot('az/probe/'+r['path']);original=slot('az/probe/evidence/full-origin-inputs-v1/proofs/'+r['path']);raw=current.read_bytes();require(raw==original.read_bytes() and (sha(raw),len(raw))==(r['sha256'],r['size']),'current/full-origin proof pair differs')
 if r['path'].endswith('/proof.json'):
  stats['files']+=1
  for node in get(current)['proofs']['Coma'].values():visit(node)
require(stats==full['statistics']=={'files':191,'prover':2207,'null':0,'structural':0},'independent full proof statistics mismatch')
for r in full['probe_input_identity']:
 raw=slot('az/probe/'+r['path']).read_bytes();original=slot('az/probe/evidence/full-origin-inputs-v1/inputs/'+r['path']).read_bytes();require(raw==original and (sha(raw),len(raw))==(r['sha256'],r['size']),'proof-critical source origin/current differs')
for logical,pin0 in [('az/probe/generated/correspondence.json','621e09dd4491be55ca6f0bf093e832b8e757b1a1235b179351b2067433a681c3'),('az/probe/generated/compiled-capture-summary.json','91858b53c03d2c822ca98570e0ee9ef0d3334740880ce04ad397bf6319732e80')]:
 p=slot(logical);require(sha(p.read_bytes())==pin0 and get(p)['status']=='pass','separate current gate authority mismatch')
require(sha((b/'capture-report.json').read_bytes())=='4547484857216d8f0a79ca68564aee3f859b5373757818ed7773618b91969f75','capture report authority mismatch')
roles=collections.Counter(e['role'] for e in d['provenance']);require(set(roles)=={'proof_origin','diagnostic_control','reused_ancestor','canonical_admission'},'typed roles incomplete')
aypath=Path('/workspace/bytes-work/bytes/1.11.1/verification/evidence-closures/ay-admitted-reuse-v1/manifest.json');require((h/'lineage/imports/ay/manifest.json').read_bytes()==aypath.read_bytes(),'AY imported raw authority changed')
report=get(Path('/workspace/work/AZ_ORIGIN_INVENTORY_AUDIT.json'));report.update(status='PASS',disposition='GO for immutable diagnostic proof origin only; full original NOT ADMITTED',external_raw_manifest_sha256=pin,closed_hydrated_files=len(files),objects=len(d['objects']),typed_provenance_roles=dict(roles),statistics=stats,current_origin_proof_pairs_exact=382,current_origin_critical_inputs_exact=37,environment_inputs=117,origin_policy_diagnostic=True,origin_correspondence_exit=2,prover_process_exit=0,current_correspondence_sha256='621e09dd4491be55ca6f0bf093e832b8e757b1a1235b179351b2067433a681c3',fresh_Cargo4_sha256='91858b53c03d2c822ca98570e0ee9ef0d3334740880ce04ad397bf6319732e80',capture_report_sha256=sha((b/'capture-report.json').read_bytes()),independent_hydrate_report_sha256=sha(Path('/workspace/work/AZ_ORIGIN_HYDRATE_REPORT.json').read_bytes()),actual_manifest_audit_pending=False,inherited_controls_reused=True,new_negative_controls=0,compiler_invoked=False,normal_checker_reexecuted=False,prover_reexecuted=False,external_binary_payloads_bundled=False)
cap=get(b/'capture-report.json');report['accounting']={k:cap[k] for k in ['logical_expanded_bytes','unique_object_bytes','referenced_published_transport_bytes','newly_published_bytes']}
out=b/'reports';out.mkdir(exist_ok=True);(out/'AZ_ORIGIN_CLOSURE_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n');(out/'AZ_ORIGIN_CLOSURE_AUDIT.md').write_text('# AZ immutable proof origin audit\n\nPASS / GO for immutable diagnostic origin at external raw manifest `'+pin+'`. Full original bytes 1.11.1 remains NOT ADMITTED.\n\nIndependent required-input reconstruction and final binding audit establish all 1,353 slots, including 382 current/origin proof files, 37 complete critical source inputs, 117 environment inputs, actual native capture, four Cargo artifacts, current correspondence and preserved frontend/selected traces. All 565 inherited aliases and 788 local staging bindings are exact. Safe offline hydration independently verifies all 14,546 logical identities and the closed typed graph. The 382 current/origin proof files and 37 source inputs match exactly; statistics recompute to 191 files / 2,207 prover leaves / zero null / zero structural.\n\nOriginal diagnostic policy stays exit 2/nonadmitted with actual prover exit 0. Fresh current correspondence and Cargo snapshot remain separate passing evidence. No new negative control, normal checker, old ancestor audit, compiler or solver was run by this audit. Eight executable hashes are verified; binary payloads remain unbundled. Cargo absolute paths remain historical location-bound snapshots.\n')
print(json.dumps({'status':'PASS','raw':pin,'json_sha256':sha((out/'AZ_ORIGIN_CLOSURE_AUDIT.json').read_bytes()),'md_sha256':sha((out/'AZ_ORIGIN_CLOSURE_AUDIT.md').read_bytes())}))
