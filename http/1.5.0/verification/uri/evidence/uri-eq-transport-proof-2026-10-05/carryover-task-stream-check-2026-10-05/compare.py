#!/usr/bin/env python3
import hashlib, json, pathlib, subprocess, sys
ROOT=pathlib.Path(__file__).resolve().parents[7]
EVIDENCE=ROOT/'http/1.5.0/verification/uri/evidence/uri-eq-transport-proof-2026-10-05'
CHECK=EVIDENCE/'carryover-task-stream-check-2026-10-05'
HARNESS=ROOT/'http/1.5.0/verification/uri'
LIB=pathlib.Path('/workspace/proof-tools/creusot-data/share/why3find/packages/creusot')
CASES=[
 ('from-authority-body','uri-conversion-proof-2026-10-05','from_authority_for_uri.coma','uri/impl_From_for_Uri/from.coma'),
 ('from-authority-refines','uri-conversion-proof-2026-10-05','from_authority_for_uri__refines.coma','uri/impl_From_for_Uri/from__refines.coma'),
 ('from-path-body','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri.coma','uri/impl_From_for_Uri_0/from.coma'),
 ('from-path-refines','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri__refines.coma','uri/impl_From_for_Uri_0/from__refines.coma'),
 ('debug-body','uri-debug-proof-2026-10-05','uri_debug_fmt.coma','uri/impl_Debug_for_Uri/fmt.coma'),
 ('debug-refines','uri-debug-proof-2026-10-05','uri_debug_fmt__refines.coma','uri/impl_Debug_for_Uri/fmt__refines.coma'),
]
def sha(data): return hashlib.sha256(data).hexdigest()
results=[]
for label,old_evidence,coma_name,fresh_suffix in CASES:
 old_coma=EVIDENCE.parent/old_evidence/'comas'/coma_name
 fresh_coma=HARNESS/'verif/http_uri_proof_rlib'/fresh_suffix
 record={'case':label,'old_coma':str(old_coma.relative_to(ROOT)),'fresh_coma':str(fresh_coma.relative_to(ROOT))}
 task_maps=[]
 for version,coma in [('old',old_coma),('fresh',fresh_coma)]:
  out=CHECK/'outputs'/label/version/'tasks'
  out.mkdir(parents=True,exist_ok=True)
  proc=subprocess.run(['why3','prove','-L',str(LIB),'-a','split_vc','-D','why3','-o',str(out),str(coma)],cwd=HARNESS,text=True,capture_output=True)
  base=CHECK/'outputs'/label/version
  (base/'stdout.txt').write_text(proc.stdout)
  (base/'stderr.txt').write_text(proc.stderr)
  (base/'exit-status.txt').write_text(str(proc.returncode)+'\n')
  tasks=sorted(p for p in out.rglob('*.why') if p.is_file())
  task_hashes=sorted(sha(p.read_bytes()) for p in tasks)
  task_maps.append(task_hashes)
  record[version]={
   'exit_code':proc.returncode,
   'stdout_sha256':sha(proc.stdout.encode()),
   'stderr_sha256':sha(proc.stderr.encode()),
   'stdout_bytes':len(proc.stdout.encode()),
   'stderr_bytes':len(proc.stderr.encode()),
   'task_count':len(tasks),
   'tasks':[{'file':str(p.relative_to(CHECK)),'sha256':sha(p.read_bytes())} for p in tasks],
   'complete_task_hash_multiset':task_hashes,
  }
 record['exact_full_task_stream_identity']=record['old']['exit_code']==record['fresh']['exit_code']==0 and task_maps[0]==task_maps[1]
 record['stdout_identity']=record['old']['stdout_sha256']==record['fresh']['stdout_sha256']
 record['stderr_identity']=record['old']['stderr_sha256']==record['fresh']['stderr_sha256']
 results.append(record)
summary={
 'schema_version':1,
 'purpose':'Read-only full Why3 split-task comparison for the six conversion/Debug COMAs previously accepted under their own source freeze. No span normalization is applied to generated task files.',
 'command':'command.txt',
 'all_six_full_task_streams_identical':all(x['exact_full_task_stream_identity'] for x in results),
 'all_six_console_stdout_identical':all(x['stdout_identity'] for x in results),
 'all_six_stderr_identical':all(x['stderr_identity'] for x in results),
 'results':results,
 'reuse_rule':'Only a byte-identical complete generated task multiset can support transfer of a previous proof result. Console or span-normalized COMA identity alone is insufficient.'
}
(CHECK/'comparison.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'all_six_full_task_streams_identical':summary['all_six_full_task_streams_identical'],'all_stdout_identical':summary['all_six_console_stdout_identical'],'all_stderr_identical':summary['all_six_stderr_identical'],'counts':{r['case']:{'old':r['old']['task_count'],'fresh':r['fresh']['task_count']} for r in results}},indent=2))
if any(r['old']['exit_code'] or r['fresh']['exit_code'] for r in results): sys.exit(1)
