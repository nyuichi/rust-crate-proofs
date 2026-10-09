#!/usr/bin/env python3
"""Narrow AY admission reuse audit; no old checker/prover or ancestor scan."""
import json,hashlib,collections,argparse
from pathlib import Path
sha=lambda b:hashlib.sha256(b).hexdigest()
def get(p):return json.loads(p.read_bytes())
def require(c,m):
 if not c:raise RuntimeError(m)
repo=Path('/workspace/bytes-work');verify=repo/'bytes/1.11.1/verification';origin=verify/'evidence-closures/ay-proof-origin-v1';probe=verify/'probes/original-root-phase-view-2026-10-09';stage=Path('/workspace/work/ay-admission-source-v1');recipepath=Path('/workspace/work/ay-admission-recipe-v1.json')
require(sha(recipepath.read_bytes())=='45e5206d6f5552dd538cc02fb75bcff655a7adf4ae4f0b59c166d51d037d65df','outer recipe pin')
recipe=get(recipepath);full=get(probe/'evidence/AY_FULL_PROOF_RUN.json');record=get(stage/'metadata/AY_ADMISSION_REUSE.json');profile=get(stage/'metadata/AY_ADMISSION_REUSE_PROFILE.json');od=get(origin/'manifest.json')
require(sha((origin/'manifest.json').read_bytes())=='fa26279c75516fb66d4d7b419f2a54cbac4bb85ab164c6de376f515e6a622c3e','origin pin')
for n,pin in [('AY_ADMISSION_REUSE.json','fb491e83c98d021f736e49b2229b537584e8d25a667859db76d5049c96c1938a'),('AY_ADMISSION_REUSE_PROFILE.json','43913a7595a825252402fc1b1b1e585b9c96648cda2bc4afa36d2d3f264d7905')]:require(sha((stage/'metadata'/n).read_bytes())==pin,'frozen outer metadata changed')
critical=full['proof_file_identity']+full['probe_input_identity'];require(len(critical)==408 and profile['current_proof_critical_identity_count']==408 and profile['current_proof_critical_identity_sha256']==sha((json.dumps(critical,sort_keys=True,indent=2)+'\n').encode()),'critical inventory mismatch')
for r in critical:
 q=probe/r['path'];require(q.is_file() and not q.is_symlink() and (sha(q.read_bytes()),q.stat().st_size)==(r['sha256'],r['size']),'actual proof-critical source/output changed')
require(record['status']=='admitted_reuse' and record['prover_reexecuted'] is False and record['original_policy_diagnostic'] and record['original_policy_correspondence_exit']==2 and record['current_correspondence_exit']==0 and record['full_original_admitted'] is False and profile['full_original_admitted'] is False,'reuse scope conflates origin/current/full disposition')
require(record['statistics']==full['statistics']=={'files':186,'prover':1955,'null':0,'structural':0},'proof statistics changed')
prov={e['id']:e for e in od['provenance']};om={m['id']:m for m in od['mounts']};imports=recipe['imports'];require(len(imports)==1 and imports[0]['manifest_sha256']==record['origin_raw_manifest_sha256'] and len(imports[0]['mounts'])==len(om)==1265,'origin import completeness')
for m in imports[0]['mounts']:
 old=om[m['source_mount_id']];require(m['path_prefix']=='ancestors/ay-origin/'+old['path_prefix'] and m['role']==prov[old['provenance_id']]['role'],'origin role/namespace changed')
require(len({m['source_mount_id'] for m in imports[0]['mounts']})==1265,'duplicate origin selection')
sources={s['id']:s for s in recipe['sources']};local=[s for s in sources.values() if s['kind']=='local_file'];published=[s for s in sources.values() if s['kind']=='published_parts'];require(len(local)==3 and len(published)==7,'outer introduces unexpected producer payloads')
for s in published:require(len(s['parts'])==1 and (repo/s['parts'][0]).is_file(),'existing report reference missing')
files={r['path']:r for r in od['files']}
for ref in [profile['complete_frozen_input_binding_profile'],profile['frozen_required_inventory'],*profile['current_gates'].values()]:
 path=ref['path'];require(path.startswith('ancestors/ay-origin/'),'foreign admission input');r=files[path[len('ancestors/ay-origin/'):]];require((r['sha256'],r['size'])==(ref['sha256'],ref['size']),'admission source pin mismatch')
require(profile['source_checker_sha256']=='502470d6589adc437d80512e9cd87244e8cdf2998cf5306f501901c93ccb7803' and sha((probe/'check_correspondence.py').read_bytes())==profile['source_checker_sha256'],'current source checker changed')
require(record['origin_independent_audit']['json_sha256']==sha((origin/'reports/AY_ORIGIN_CLOSURE_AUDIT.json').read_bytes()) and record['origin_independent_audit']['markdown_sha256']==sha((origin/'reports/AY_ORIGIN_CLOSURE_AUDIT.md').read_bytes()),'independent origin audit authority changed')
ap=argparse.ArgumentParser();ap.add_argument('--manifest',type=Path);ap.add_argument('--expected-root-sha256');a=ap.parse_args()
report={'status':'PASS','scope':'narrow immutable admission reuse audit','proof_critical_identities_checked':408,'exact_origin_mounts_preserved':1265,'local_metadata_files':3,'existing_report_transports':7,'frozen_origin_required_slots':1394,'statistics':record['statistics'],'prover_reexecuted':False,'normal_checker_reexecuted':False,'origin_policy_diagnostic':True,'origin_correspondence_exit':2,'separate_current_correspondence_exit':0,'full_original_admitted':False}
if a.manifest:
 require(a.expected_root_sha256 and sha(a.manifest.read_bytes())==a.expected_root_sha256,'external outer pin mismatch');d=get(a.manifest);df={r['path']:r for r in d['files']};expected={}
 for path,r in files.items():expected['ancestors/ay-origin/'+path]=(r['sha256'],r['size'])
 for m in recipe['mounts']:
  s=sources[m['source_id']];q=stage/s['path'] if s['kind']=='local_file' else repo/s['parts'][0];raw=q.read_bytes();expected[m['path_prefix']]=(sha(raw),len(raw))
 for path,value in expected.items():require(path in df and (df[path]['sha256'],df[path]['size'])==value,'outer logical identity changed '+path)
 require(set(df)-set(expected)=={'lineage/capture-recipe.json','lineage/imports/ay-origin/manifest.json','lineage/imports/ay-origin/selection.json'},'undeclared outer namespace')
 require(df['lineage/capture-recipe.json']['sha256']==sha(recipepath.read_bytes()) and df['lineage/imports/ay-origin/manifest.json']['sha256']==sha((origin/'manifest.json').read_bytes()),'lineage authority mismatch')
 outerprov={e['id']:e for e in d['provenance']};outermounts={m['path_prefix']:m for m in d['mounts']}
 for old in od['mounts']:
  m=outermounts['ancestors/ay-origin/'+old['path_prefix']];require(m['object_sha256']==old['object_sha256'] and m['format']==old['format'] and m['member_prefix']==old['member_prefix'] and m['strip_prefix']==old['strip_prefix'] and outerprov[m['provenance_id']]['role']==prov[old['provenance_id']]['role'],'final imported mount identity/role changed')
 oldobjects={o['sha256']:o for o in od['objects']};outerobjects={o['sha256']:o for o in d['objects']};require(set(oldobjects)<=set(outerobjects),'imported object missing')
 for digest,o in oldobjects.items():require((outerobjects[digest]['size'],outerobjects[digest]['media'])==(o['size'],o['media']),'immutable object size/media changed')
 casfiles=list((a.manifest.parent/'objects').rglob('*'));casfiles=[q for q in casfiles if q.is_file()];localdigests={sha((stage/s['path']).read_bytes()) for s in local}
 require(all(q.name in localdigests|{sha(recipepath.read_bytes()),df['lineage/imports/ay-origin/selection.json']['sha256']} for q in casfiles),'duplicated ancestor/report object in new CAS')
 report.update(external_outer_manifest_sha256=a.expected_root_sha256,outer_file_count=len(df),outer_object_count=len(outerobjects),new_CAS_file_count=len(casfiles),new_CAS_bytes=sum(q.stat().st_size for q in casfiles),closed_graph_status='root composer validated; exact namespace/import/provenance identities independently checked')
 output=a.manifest.parent/'reports';output.mkdir(exist_ok=True);(output/'AY_ADMISSION_AUDIT.json').write_text(json.dumps(report,indent=2)+'\n')
 (output/'AY_ADMISSION_AUDIT.md').write_text('# AY admission audit\n\nPASS / GO for bounded admitted reuse at externally pinned raw manifest `'+a.expected_root_sha256+'`. Full original bytes 1.11.1 remains NOT ADMITTED.\n\nAll 1,265 audited-origin mounts and their typed roles/identities are preserved. The complete frozen 1,394-slot profile and independent origin audit remain linked. All 408 actual current proof-critical source/output identities match the single completed 186/1,955/0/0 proof origin. Original diagnostic correspondence exit 2 is preserved; current correspondence exit 0 and fresh Cargo four-artifact capture remain separate evidence. Exactly three small metadata files are new; seven existing reports are referenced in place. No ancestor object or report is republished into new CAS.\n\nNo ordinary checker, compiler, ancestor audit or solver was rerun. Historical captured documentation retains its frozen identity; current editorial files are separate. External executables remain hash-pinned but unbundled; Cargo paths retain historical location dependence.\n')
 print(json.dumps(report))
else:print(json.dumps(report))
