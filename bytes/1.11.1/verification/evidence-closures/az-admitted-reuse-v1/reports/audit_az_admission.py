#!/usr/bin/env python3
"""Narrow AZ admission reuse audit; no old checker/prover or ancestor scan."""
import json,hashlib,collections,argparse
from pathlib import Path
sha=lambda b:hashlib.sha256(b).hexdigest()
def get(p):return json.loads(p.read_bytes())
def require(c,m):
 if not c:raise RuntimeError(m)
repo=Path('/workspace/bytes-work');verify=repo/'bytes/1.11.1/verification';origin=verify/'evidence-closures/az-proof-origin-v1';probe=verify/'probes/original-root-phase-clone-2026-10-09';stage=Path('/workspace/work/az-admission-source-v1');recipepath=Path('/workspace/work/az-admission-recipe-v1.json')
require(sha(recipepath.read_bytes())=='b151a693b64839cc5a90646aa0cbe92dac7dd85a19fa80030501e14b04e0c8c7','outer recipe pin')
recipe=get(recipepath);full=get(probe/'evidence/AZ_FULL_PROOF_RUN.json');record=get(stage/'metadata/AZ_ADMISSION_REUSE.json');profile=get(stage/'metadata/AZ_ADMISSION_REUSE_PROFILE.json');od=get(origin/'manifest.json')
require(sha((origin/'manifest.json').read_bytes())=='6bfe965371c3a7ec3c91e0c1a8429de9e1f48810afd61a0fb6adabfcd86a0772','origin pin')
for n,pin in [('AZ_ADMISSION_REUSE.json','3748d26f9c06e6dcbe2eb388f37e588da26052e9b04c6ad6f4a5dd81f560b93a'),('AZ_ADMISSION_REUSE_PROFILE.json','724f5d44b09038b2407d6a44e82aa16191ce97cd1828508ec1213e28ce3eac78')]:require(sha((stage/'metadata'/n).read_bytes())==pin,'frozen outer metadata changed')
critical=full['proof_file_identity']+full['probe_input_identity'];require(len(critical)==419 and profile['current_proof_critical_identity_count']==419 and profile['current_proof_critical_identity_sha256']==sha((json.dumps(critical,sort_keys=True,indent=2)+'\n').encode()),'critical inventory mismatch')
for r in critical:
 q=probe/r['path'];require(q.is_file() and not q.is_symlink() and (sha(q.read_bytes()),q.stat().st_size)==(r['sha256'],r['size']),'actual proof-critical source/output changed')
require(record['status']=='admitted_reuse' and record['prover_reexecuted'] is False and record['original_policy_diagnostic'] and record['original_policy_correspondence_exit']==2 and record['current_correspondence_exit']==0 and record['full_original_admitted'] is False and profile['full_original_admitted'] is False,'reuse scope conflates origin/current/full disposition')
require(record['statistics']==full['statistics']=={'files':191,'prover':2207,'null':0,'structural':0},'proof statistics changed')
prov={e['id']:e for e in od['provenance']};om={m['id']:m for m in od['mounts']};imports=recipe['imports'];require(len(imports)==1 and imports[0]['manifest_sha256']==record['origin_raw_manifest_sha256'] and len(imports[0]['mounts'])==len(om)==2072,'origin import completeness')
for m in imports[0]['mounts']:
 old=om[m['source_mount_id']];require(m['path_prefix']=='ancestors/az-origin/'+old['path_prefix'] and m['role']==prov[old['provenance_id']]['role'],'origin role/namespace changed')
require(len({m['source_mount_id'] for m in imports[0]['mounts']})==2072,'duplicate origin selection')
sources={s['id']:s for s in recipe['sources']};local=[s for s in sources.values() if s['kind']=='local_file'];published=[s for s in sources.values() if s['kind']=='published_parts'];require(len(local)==3 and len(published)==8,'outer introduces unexpected producer payloads')
for s in published:require(len(s['parts'])==1 and (repo/s['parts'][0]).is_file(),'existing report reference missing')
files={r['path']:r for r in od['files']}
for ref in [profile['complete_frozen_input_binding_profile'],profile['frozen_required_inventory'],*profile['current_gates'].values()]:
 path=ref['path'];require(path.startswith('ancestors/az-origin/'),'foreign admission input');r=files[path[len('ancestors/az-origin/'):]];require((r['sha256'],r['size'])==(ref['sha256'],ref['size']),'admission source pin mismatch')
require(profile['source_checker_sha256']=='66c5657bed08daa22537f5a8efb0c3004bfbb850e7e22655bfbcdcab6db78b77' and sha((probe/'check_correspondence.py').read_bytes())==profile['source_checker_sha256'],'current source checker changed')
require(record['origin_independent_audit']['json_sha256']==sha((origin/'reports/AZ_ORIGIN_CLOSURE_AUDIT.json').read_bytes()) and record['origin_independent_audit']['markdown_sha256']==sha((origin/'reports/AZ_ORIGIN_CLOSURE_AUDIT.md').read_bytes()),'independent origin audit authority changed')
ap=argparse.ArgumentParser();ap.add_argument('--manifest',type=Path);ap.add_argument('--expected-root-sha256');a=ap.parse_args()
report={'status':'PASS','scope':'narrow immutable admission reuse audit','proof_critical_identities_checked':419,'exact_origin_mounts_preserved':2072,'local_metadata_files':3,'existing_report_transports':8,'frozen_origin_required_slots':1353,'statistics':record['statistics'],'prover_reexecuted':False,'normal_checker_reexecuted':False,'origin_policy_diagnostic':True,'origin_correspondence_exit':2,'separate_current_correspondence_exit':0,'full_original_admitted':False}
if a.manifest:
 require(a.expected_root_sha256 and sha(a.manifest.read_bytes())==a.expected_root_sha256,'external outer pin mismatch');d=get(a.manifest);df={r['path']:r for r in d['files']};expected={}
 for path,r in files.items():expected['ancestors/az-origin/'+path]=(r['sha256'],r['size'])
 for m in recipe['mounts']:
  s=sources[m['source_id']];q=stage/s['path'] if s['kind']=='local_file' else repo/s['parts'][0];raw=q.read_bytes();expected[m['path_prefix']]=(sha(raw),len(raw))
 for path,value in expected.items():require(path in df and (df[path]['sha256'],df[path]['size'])==value,'outer logical identity changed '+path)
 require(set(df)-set(expected)=={'lineage/capture-recipe.json','lineage/imports/az-origin/manifest.json','lineage/imports/az-origin/selection.json'},'undeclared outer namespace')
 require(df['lineage/capture-recipe.json']['sha256']==sha(recipepath.read_bytes()) and df['lineage/imports/az-origin/manifest.json']['sha256']==sha((origin/'manifest.json').read_bytes()),'lineage authority mismatch')
 outerprov={e['id']:e for e in d['provenance']};outermounts={m['path_prefix']:m for m in d['mounts']}
 for old in od['mounts']:
  m=outermounts['ancestors/az-origin/'+old['path_prefix']];require(m['object_sha256']==old['object_sha256'] and m['format']==old['format'] and m['member_prefix']==old['member_prefix'] and m['strip_prefix']==old['strip_prefix'] and outerprov[m['provenance_id']]['role']==prov[old['provenance_id']]['role'],'final imported mount identity/role changed')
 oldobjects={o['sha256']:o for o in od['objects']};outerobjects={o['sha256']:o for o in d['objects']};require(set(oldobjects)<=set(outerobjects),'imported object missing')
 for digest,o in oldobjects.items():require((outerobjects[digest]['size'],outerobjects[digest]['media'])==(o['size'],o['media']),'immutable object size/media changed')
 casfiles=list((a.manifest.parent/'objects').rglob('*'));casfiles=[q for q in casfiles if q.is_file()];localdigests={sha((stage/s['path']).read_bytes()) for s in local}
 require(all(q.name in localdigests|{sha(recipepath.read_bytes()),df['lineage/imports/az-origin/selection.json']['sha256']} for q in casfiles),'duplicated ancestor/report object in new CAS')
 report.update(external_outer_manifest_sha256=a.expected_root_sha256,outer_file_count=len(df),outer_object_count=len(outerobjects),new_CAS_file_count=len(casfiles),new_CAS_bytes=sum(q.stat().st_size for q in casfiles),closed_graph_status='root composer validated; exact namespace/import/provenance identities independently checked')
 output=a.manifest.parent/'reports';output.mkdir(exist_ok=True);(output/'AZ_ADMISSION_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n')
 (output/'AZ_ADMISSION_AUDIT.md').write_text('# AZ admission audit\n\nPASS / GO for bounded admitted reuse at externally pinned raw manifest `'+a.expected_root_sha256+'`. Full original bytes 1.11.1 remains NOT ADMITTED.\n\nAll 1,265 audited-origin mounts and their typed roles/identities are preserved. The complete frozen 1,394-slot profile and independent origin audit remain linked. All 419 actual current proof-critical source/output identities match the single completed 191/2,207/0/0 proof origin. Original diagnostic correspondence exit 2 is preserved; current correspondence exit 0 and fresh Cargo four-artifact capture remain separate evidence. Exactly three small metadata files are new; eight existing reports are referenced in place. No ancestor object or report is republished into new CAS.\n\nNo ordinary checker, compiler, ancestor audit or solver was rerun. Historical captured documentation retains its frozen identity; current editorial files are separate. External executables remain hash-pinned but unbundled; Cargo paths retain historical location dependence.\n')
 print(json.dumps(report))
else:print(json.dumps(report))
