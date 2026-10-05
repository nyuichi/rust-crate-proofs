#!/usr/bin/env python3
import hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[7]
EV=ROOT/'http/1.5.0/verification/uri/evidence'
CHECK=EV/'uri-eq-transport-proof-2026-10-05/carryover-task-stream-check-2026-10-05'
HARNESS=ROOT/'http/1.5.0/verification/uri'
LIB='/workspace/proof-tools/creusot-data/share/why3find/packages/creusot'
CASES=[
 ('from-authority-body','uri-conversion-proof-2026-10-05','from_authority_for_uri.coma','uri/impl_From_for_Uri/from.coma'),
 ('from-authority-refines','uri-conversion-proof-2026-10-05','from_authority_for_uri__refines.coma','uri/impl_From_for_Uri/from__refines.coma'),
 ('from-path-body','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri.coma','uri/impl_From_for_Uri_0/from.coma'),
 ('from-path-refines','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri__refines.coma','uri/impl_From_for_Uri_0/from__refines.coma'),
 ('debug-body','uri-debug-proof-2026-10-05','uri_debug_fmt.coma','uri/impl_Debug_for_Uri/fmt.coma'),
 ('debug-refines','uri-debug-proof-2026-10-05','uri_debug_fmt__refines.coma','uri/impl_Debug_for_Uri/fmt__refines.coma'),
]
def sha(b):return hashlib.sha256(b).hexdigest()
records=[]
for label,old_dir,old_name,fresh_tail in CASES:
 paths={'old':EV/old_dir/'comas'/old_name,'fresh':HARNESS/'verif/http_uri_proof_rlib'/fresh_tail}
 entry={'case':label}
 for ver,path in paths.items():
  outdir=CHECK/'full-stdout'/label/ver;outdir.mkdir(parents=True,exist_ok=True)
  cmd=['why3','prove','-L',LIB,'-a','split_vc','-D','why3',str(path)]
  p=subprocess.run(cmd,cwd=HARNESS,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  (outdir/'stdout.bin').write_bytes(p.stdout)
  (outdir/'stderr.bin').write_bytes(p.stderr)
  (outdir/'exit-status.txt').write_text(str(p.returncode)+'\n')
  entry[ver]={'input':str(path.relative_to(ROOT)),'command':cmd,'exit_code':p.returncode,'stdout_file':str((outdir/'stdout.bin').relative_to(CHECK)),'stdout_bytes':len(p.stdout),'stdout_sha256':sha(p.stdout),'stderr_file':str((outdir/'stderr.bin').relative_to(CHECK)),'stderr_bytes':len(p.stderr),'stderr_sha256':sha(p.stderr)}
 entry['complete_stdout_byte_identity']=entry['old']['exit_code']==entry['fresh']['exit_code']==0 and (CHECK/entry['old']['stdout_file']).read_bytes()==(CHECK/entry['fresh']['stdout_file']).read_bytes()
 records.append(entry)
obj={'schema_version':1,'method':'For each archived and fresh COMA separately, invoke `why3 prove -L <creusot-packages> -a split_vc -D why3 <COMA>` without `-o`. The entire task printer output is captured as raw stdout bytes; no normalization or task regrouping is applied.','command':'full-stdout-command.txt','cases':records,'all_six_full_stdout_streams_byte_identical':all(x['complete_stdout_byte_identity'] for x in records),'stderr_policy':'stderr is retained to diagnose warnings/source-span differences; it is not used as the VC stream because the full generated task bodies are stdout.'}
(CHECK/'full-stdout-comparison.json').write_text(json.dumps(obj,indent=2)+'\n')
print(json.dumps({'all_six_full_stdout_streams_byte_identical':obj['all_six_full_stdout_streams_byte_identical'],'bytes':{r['case']:r['old']['stdout_bytes'] for r in records}},indent=2))
if any(r['old']['exit_code'] or r['fresh']['exit_code'] for r in records):sys.exit(1)
