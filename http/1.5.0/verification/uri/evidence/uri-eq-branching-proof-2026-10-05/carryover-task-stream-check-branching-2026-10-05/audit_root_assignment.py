#!/usr/bin/env python3
import hashlib,json,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[7]
EV=ROOT/'http/1.5.0/verification/uri/evidence'
CHECK=EV/'uri-eq-branching-proof-2026-10-05/carryover-task-stream-check-branching-2026-10-05'
comp=json.loads((CHECK/'comparison.json').read_text())
conv=json.loads((EV/'uri-conversion-proof-2026-10-05/arity/audit.json').read_text())
debug=json.loads((EV/'uri-debug-proof-2026-10-05/arity/audit.json').read_text())
conv_targets={x['target']:x for x in conv['targets']}
map_conv={'from-authority-body':'From<Authority> for Uri body','from-authority-refines':'From<Authority> for Uri refinement','from-path-body':'From<PathAndQuery> for Uri body','from-path-refines':'From<PathAndQuery> for Uri refinement'}

def root_id_name(name):
 name=pathlib.Path(name).name.removesuffix('.why')
 if '-Coma-' in name: return name.split('-Coma-',1)[1]
 if '-vc_' in name: return 'vc_'+name.split('-vc_',1)[1]
 if '-refines-' in name: return name.split('-refines-',1)[1]
 raise ValueError('cannot identify Why3 root from task name: '+name)
def root_id(x): return root_id_name(x['file'])
records=[]
for case in comp['results']:
 label=case['case']
 old={root_id(x):x['sha256'] for x in case['old']['tasks']}
 fresh={root_id(x):x['sha256'] for x in case['fresh']['tasks']}
 if label in map_conv:
  audited=conv_targets[map_conv[label]]['independent_task_hashes']
  archived={root_id_name(k):v for k,v in audited.items()}
 else:
  if label=='debug-body': filt=lambda k:'fmt-Coma-vc_fmt_Uri' in k
  else: filt=lambda k:'fmt__refines-Coma-refines' in k
  archived={root_id_name(k):v for k,v in debug['task_sha256'].items() if filt(k)}
 # VC suffixes are Why3's root assignments. Sorting these stable labels gives the split task order;
 # the full stdout comparison independently verifies concatenation order without parsing/normalizing.
 order=sorted(old)
 rows=[{'ordinal':i,'why3_vc_suffix':k,'archived_arity_sha256':archived.get(k),'old_reprint_sha256':old.get(k),'fresh_current_sha256':fresh.get(k),'all_three_exact':archived.get(k)==old.get(k)==fresh.get(k)} for i,k in enumerate(order)]
 full=json.loads((CHECK/'full-stdout-comparison.json').read_text())
 full_case=next(x for x in full['cases'] if x['case']==label)
 records.append({'case':label,'old_task_roots':order,'fresh_task_roots':sorted(fresh),'root_assignment_order_same':order==sorted(fresh),'per_root_archived_old_fresh_bytes_same':all(x['all_three_exact'] for x in rows),'full_stdout_old_fresh_bytes_same':full_case['complete_stdout_byte_identity'],'roots':rows})
obj={'schema_version':1,'method':'Map each Why3 split task by its generated root suffix, preserving per-COMA identity. For every root compare archived independent-arity SHA, old COMA reprint SHA, and fresh COMA SHA. Independently compare the complete no-`-o` Why3 stdout byte stream for each individual COMA, which preserves root order.','all_six_per_root_assignments_and_order_identical':all(x['root_assignment_order_same'] and x['per_root_archived_old_fresh_bytes_same'] and x['full_stdout_old_fresh_bytes_same'] for x in records),'cases':records,'no_multiset_only_claim':True}
(CHECK/'root-assignment-order.json').write_text(json.dumps(obj,indent=2)+'\n')
# Attach stronger evidence references to the overall comparison summary.
comp['all_six_per_root_assignments_and_order_identical']=obj['all_six_per_root_assignments_and_order_identical']
comp['per_root_assignment_order_audit']='root-assignment-order.json'
comp['complete_full_stdout_comparison']='full-stdout-comparison.json'
(CHECK/'comparison.json').write_text(json.dumps(comp,indent=2)+'\n')
print(json.dumps({'all_six_per_root_assignments_and_order_identical':obj['all_six_per_root_assignments_and_order_identical'],'by_case':{x['case']:x['per_root_archived_old_fresh_bytes_same'] and x['full_stdout_old_fresh_bytes_same'] for x in records}},indent=2))
