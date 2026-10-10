#!/usr/bin/env python3
import hashlib, json, pathlib, collections
ROOT=pathlib.Path(__file__).resolve().parents[7]
EVIDENCE=ROOT/'http/1.5.0/verification/uri/evidence'
CHECK=EVIDENCE/'uri-eq-transport-proof-2026-10-05/carryover-task-stream-check-2026-10-05'
comparison=json.loads((CHECK/'comparison.json').read_text())
conv=json.loads((EVIDENCE/'uri-conversion-proof-2026-10-05/arity/audit.json').read_text())
debug=json.loads((EVIDENCE/'uri-debug-proof-2026-10-05/arity/audit.json').read_text())
conv_manifest=json.loads((EVIDENCE/'uri-conversion-proof-2026-10-05/manifest.json').read_text())
debug_manifest=json.loads((EVIDENCE/'uri-debug-proof-2026-10-05/manifest.json').read_text())
conv_targets={x['target']:x for x in conv['targets']}
map_conv={
 'from-authority-body':'From<Authority> for Uri body',
 'from-authority-refines':'From<Authority> for Uri refinement',
 'from-path-body':'From<PathAndQuery> for Uri body',
 'from-path-refines':'From<PathAndQuery> for Uri refinement',
}
map_proof={
 'from-authority-body':'proofs/from_authority_for_uri.json',
 'from-authority-refines':'proofs/from_authority_for_uri__refines.json',
 'from-path-body':'proofs/from_path_and_query_for_uri.json',
 'from-path-refines':'proofs/from_path_and_query_for_uri__refines.json',
 'debug-body':'proofs/uri_debug_fmt.json',
 'debug-refines':'proofs/uri_debug_fmt__refines.json',
}

def sha_file(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def count_provers(n):
 if isinstance(n,dict):
  if 'prover' in n: return 1
  return sum(count_provers(v) for v in n.values())
 if isinstance(n,list): return sum(count_provers(x) for x in n)
 return 0
records=[]
for r in comparison['results']:
 label=r['case']; old_tasks=r['old']['tasks']; fresh_tasks=r['fresh']['tasks']
 if label in map_conv:
  t=conv_targets[map_conv[label]]
  archived_hashes=sorted(t['independent_task_hashes'].values())
  proof_path=EVIDENCE/'uri-conversion-proof-2026-10-05'/map_proof[label]
  proof_root=t['own_root']
  proof_json=json.loads(proof_path.read_text())
  root_nodes=proof_json['proofs']['Coma']
  own_leaf_count=count_provers(root_nodes[proof_root])
  supports=t.get('support_roots',[])
  reported_count=t['own_terminal_prover_leaves']
  expected_task_count=t['own_children']
 elif label=='debug-body':
  da=debug['task_sha256']
  archived_hashes=sorted(v for k,v in da.items() if '/fmt-Coma-vc_fmt_Uri' in k)
  proof_path=EVIDENCE/'uri-debug-proof-2026-10-05'/map_proof[label]
  proof_json=json.loads(proof_path.read_text())
  root_nodes=proof_json['proofs']['Coma']
  proof_root="vc_fmt_Uri'0"
  own_leaf_count=count_provers(root_nodes[proof_root])
  supports=debug['support_roots']
  reported_count=debug['body_owned_leaves']; expected_task_count=reported_count
 else:
  da=debug['task_sha256']
  archived_hashes=sorted(v for k,v in da.items() if '/fmt__refines-Coma-refines' in k)
  proof_path=EVIDENCE/'uri-debug-proof-2026-10-05'/map_proof[label]
  proof_json=json.loads(proof_path.read_text())
  root_nodes=proof_json['proofs']['Coma']; proof_root='refines'
  own_leaf_count=count_provers(root_nodes[proof_root]); supports=[]
  reported_count=debug['refinement_owned_leaves']; expected_task_count=reported_count
 old_hashes=sorted(x['sha256'] for x in old_tasks)
 fresh_hashes=sorted(x['sha256'] for x in fresh_tasks)
 records.append({
  'case':label,
  'old_proof_json':str(proof_path.relative_to(ROOT)),
  'old_proof_json_sha256':sha_file(proof_path),
  'old_arity_audit':('uri-conversion-proof-2026-10-05/arity/audit.json' if label in map_conv else 'uri-debug-proof-2026-10-05/arity/audit.json'),
  'owned_root':proof_root,
  'old_tree_owned_prover_leaves':own_leaf_count,
  'old_tree_arity_matches_old_audit':own_leaf_count==reported_count==expected_task_count,
  'zero_child_support_roots':supports,
  'archived_old_arity_task_hashes_match_freshly_reprinted_old_coma':old_hashes==archived_hashes,
  'fresh_current_task_hashes_match_archived_old_arity_task_hashes':fresh_hashes==archived_hashes,
  'fresh_current_task_hashes_match_reprinted_old_coma_hashes':fresh_hashes==old_hashes,
  'old_task_count':len(old_tasks),
  'fresh_task_count':len(fresh_tasks),
 })
summary={
 'schema_version':1,
 'method':'Independent old-tree audit combines archived proof.json tree counts, archived split_vc arity audit hashes, and this package\'s separate full Why3 task prints. Generated task files are compared byte-for-byte as hash multisets; no normalization is applied.',
 'all_old_proof_trees_agree_with_archived_arity':all(x['old_tree_arity_matches_old_audit'] for x in records),
 'all_archived_task_hashes_match_old_coma_reprints':all(x['archived_old_arity_task_hashes_match_freshly_reprinted_old_coma'] for x in records),
 'all_fresh_task_streams_match_archived_old_tree_tasks':all(x['fresh_current_task_hashes_match_archived_old_arity_task_hashes'] for x in records),
 'all_cases_supported_for_exact_obligation_carryover':all(x['old_tree_arity_matches_old_audit'] and x['archived_old_arity_task_hashes_match_freshly_reprinted_old_coma'] and x['fresh_current_task_hashes_match_archived_old_arity_task_hashes'] for x in records),
 'cases':records,
 'scope_note':'This establishes exact identity of the six generated Why3 task streams; proof reuse applies only to those same obligations and does not establish any other target under the new source snapshot.'
}
(CHECK/'old-tree-independent-arity.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({k:summary[k] for k in ['all_old_proof_trees_agree_with_archived_arity','all_archived_task_hashes_match_old_coma_reprints','all_fresh_task_streams_match_archived_old_tree_tasks','all_cases_supported_for_exact_obligation_carryover']},indent=2))
