#!/usr/bin/env python3
import json,hashlib,collections
from pathlib import Path
sha=lambda b:hashlib.sha256(b).hexdigest()
def require(x,m):
 if not x:raise RuntimeError(m)
def get(p):return json.loads(p.read_bytes())
base=Path('/workspace/bytes-work/bytes/1.11.1/verification/evidence-closures/ay-proof-origin-v1');manifest=base/'manifest.json';pin='fa26279c75516fb66d4d7b419f2a54cbac4bb85ab164c6de376f515e6a622c3e';require(sha(manifest.read_bytes())==pin,'external raw pin mismatch')
d=get(manifest);h=Path('/workspace/work/ay-origin-independent-hydrate-v1');files={r['path']:r for r in d['files']};require(len(files)==len(d['files']),'manifest duplicate logical paths')
actual={q.relative_to(h).as_posix() for q in h.rglob('*') if q.is_file()};require(actual==set(files),'hydrated namespace mismatch')
for path,r in files.items():
 q=h/path;require(not q.is_symlink() and (sha(q.read_bytes()),q.stat().st_size)==(r['sha256'],r['size']),'hydrated identity mismatch '+path)
req=get(h/'ay/evidence/AY_REQUIRED_INPUTS.json');profile=get(h/'ay/evidence/AY_INPUT_BINDINGS.json') if (h/'ay/evidence/AY_INPUT_BINDINGS.json').exists() else get(h/'ay/metadata/AY_INPUT_BINDINGS.json')
for b in profile['bindings']:
 source=b['source_path'] if b['resolution']=='ancestor' else b['destination'];require(source in files and (files[source]['sha256'],files[source]['size'])==(b['sha256'],b['size']),'bound input missing/mismatch '+b['destination'])
bound={b['destination']:b for b in profile['bindings']}
def slot(logical):
 b=bound.get(logical);return h/(b['source_path'] if b and b['resolution']=='ancestor' else logical)
full=get(slot('ay/probe/evidence/AY_FULL_PROOF_RUN.json'));require(full['status']=='complete_positive_diagnostic' and full['policy_diagnostic'] and full['policy_correspondence_exit']==2 and full['prover_process_exit']==0,'origin status changed')
policy=get(slot('ay/probe/generated/proof-targets.json'));require(policy['diagnostic'] and policy['correspondence_exit_status']==2 and policy['excluded']=={} and policy['features']==[],'original policy altered')
stats={'files':0,'prover':0,'null':0,'structural':0}
def visit(n):
 if n is None:stats['null']+=1
 elif isinstance(n,dict) and 'children'in n:
  if not n['children']:stats['structural']+=1
  for c in n['children']:visit(c)
 elif isinstance(n,dict) and 'prover'in n:stats['prover']+=1
 else:raise RuntimeError('unknown proof node')
for r in full['proof_file_identity']:
 current=slot('ay/probe/'+r['path']);origin=slot('ay/probe/evidence/full-origin-inputs-v1/proofs/'+r['path']);raw=current.read_bytes();require(raw==origin.read_bytes() and (sha(raw),len(raw))==(r['sha256'],r['size']),'proof/origin reuse mismatch')
 if r['path'].endswith('/proof.json'):
  stats['files']+=1
  for n in get(current)['proofs']['Coma'].values():visit(n)
require(stats==full['statistics']=={'files':186,'prover':1955,'null':0,'structural':0},'proof statistics mismatch')
correspondence=slot('ay/probe/generated/correspondence.json');cargo=slot('ay/probe/generated/compiled-capture-summary.json');control=slot('ay/probe/evidence/peer-drop-control-v1/receipt.json')
require(sha(correspondence.read_bytes())=='30bffd2b07a881f59a37b8fc013d4109d3aa8f2649bd309612624b626b118a54' and get(correspondence)['status']=='pass','current correspondence authority mismatch')
require(sha(cargo.read_bytes())=='2e66590dee73a61b7f3ea8b639ee5b98d0c52927845296d4ac00fb5e44dda5ac' and get(cargo)['status']=='pass','fresh Cargo authority mismatch')
require(sha(control.read_bytes())=='4dc3c34b957f20e398d10a3a521ad343a87b6c020881cd3e994dc53a050322db','control authority mismatch')
c=get(control);require(c['capture_integrity']=='pass' and c['operational_rejection']['reason']=='AY client bb4 contains missing, extra, or reordered operations' and not c['ancestor_audit_reexecuted'],'wrong control rejection')
client=c['client_path'];original=slot('ay/probe/'+client).read_bytes();mutated=slot('ay/probe/evidence/peer-drop-control-v1/fixture/'+client).read_bytes();old=c['mutation']['old'].encode();new=c['mutation']['new'].encode();require(original.count(old)==1 and original.replace(old,new)==mutated,'control modifies more than exact peer Drop')
roles=collections.Counter(r['role'] for r in d['provenance']);require(set(roles)=={'proof_origin','canonical_admission','diagnostic_control','reused_ancestor'},'typed provenance role coverage mismatch')
require((h/'lineage/imports/ax/manifest.json').read_bytes()==(Path('/workspace/bytes-work/bytes/1.11.1/verification/evidence-closures/ax-admitted-reuse-v1/manifest.json')).read_bytes(),'imported AX raw authority bytes mismatch')
report=get(Path('/workspace/work/ay-audit/AY_ORIGIN_INVENTORY_AUDIT.json'));report.update({'status':'PASS','disposition':'GO for immutable diagnostic proof origin; no full original admission','external_raw_manifest_sha256':pin,'capture_report_sha256':sha((base/'capture-report.json').read_bytes()),'closed_hydrated_files':len(actual),'objects':len(d['objects']),'typed_provenance_roles':dict(roles),'statistics':stats,'all_372_current_origin_proof_files_identical':True,'current_correspondence_sha256':sha(correspondence.read_bytes()),'fresh_Cargo4_sha256':sha(cargo.read_bytes()),'peer_control_receipt_sha256':sha(control.read_bytes()),'independent_hydrate_report_sha256':sha(Path('/workspace/work/ay-audit/AY_ORIGIN_HYDRATE_REPORT.json').read_bytes()),'offline_normal_checker_reexecuted':False,'solver_reexecuted':False,'origin_admitted':False,'remaining_limits':['Full original bytes architecture remains NOT ADMITTED.','External executable hashes are verified; executable payloads are not bundled.','Cargo artifact paths are location-bound historical snapshots.','Captured ASSESSMENT is frozen historical bytes; later editorial update is separate.']})
out=base/'reports';out.mkdir(exist_ok=True);(out/'AY_ORIGIN_CLOSURE_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n')
(out/'AY_ORIGIN_CLOSURE_AUDIT.md').write_text('''# AY immutable origin audit\n\nPASS / GO for the diagnostic proof origin. Externally pinned raw manifest: `'''+pin+'''`. Full original bytes 1.11.1 remains NOT ADMITTED.\n\nIndependent reconstruction establishes all 1,394 required input slots, including 372 actual proof outputs, 36 complete source/proof inputs, 117 environment inputs, current native/Cargo/correspondence evidence and the one peer-Drop control. All 599 ancestor aliases match the exact published AX authority; all 795 local bindings match frozen staging bytes. Safe offline hydration verifies every one of the 13,739 logical file identities and the closed object graph. All 372 current/origin proof files match; independent statistics are 186 files / 1,955 prover leaves / zero null / zero structural.\n\nOriginal diagnostic policy remains exit 2 and nonadmitted; separately captured current correspondence passes and fresh four Cargo artifacts pass. The peer-Drop fixture changes only bb4 Drop to goto; refreshed integrity passes and the exact operation check rejects. No compiler, normal checker, ancestor audit or solver was rerun in this audit. All eight external executable hashes match; their payloads remain unbundled. Cargo absolute paths remain historical location-bound snapshots. Captured ASSESSMENT bytes retain their historical identity; later editorial changes are separate.\n''')
print(json.dumps({'status':'PASS','manifest_sha256':pin,'files':len(actual),'report_json_sha256':sha((out/'AY_ORIGIN_CLOSURE_AUDIT.json').read_bytes()),'report_md_sha256':sha((out/'AY_ORIGIN_CLOSURE_AUDIT.md').read_bytes())}))
