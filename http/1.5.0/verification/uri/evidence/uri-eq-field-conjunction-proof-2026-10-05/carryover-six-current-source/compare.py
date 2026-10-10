#!/usr/bin/env python3
import hashlib, json, pathlib, subprocess, sys
ROOT=pathlib.Path(__file__).resolve().parents[7]
EV=ROOT/'http/1.5.0/verification/uri/evidence'
CHECK=EV/'uri-eq-field-conjunction-proof-2026-10-05/carryover-six-current-source'
HARNESS=ROOT/'http/1.5.0/verification/uri'
LIB='/workspace/proof-tools/creusot-data/share/why3find/packages/creusot'
CASES=[
 ('from-authority-body','uri-conversion-proof-2026-10-05','from_authority_for_uri.coma','from_authority_body.coma'),
 ('from-authority-refines','uri-conversion-proof-2026-10-05','from_authority_for_uri__refines.coma','from_authority_refines.coma'),
 ('from-path-body','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri.coma','from_path_body.coma'),
 ('from-path-refines','uri-conversion-proof-2026-10-05','from_path_and_query_for_uri__refines.coma','from_path_refines.coma'),
 ('debug-body','uri-debug-proof-2026-10-05','uri_debug_fmt.coma','debug_body.coma'),
 ('debug-refines','uri-debug-proof-2026-10-05','uri_debug_fmt__refines.coma','debug_refines.coma'),
]
rows=[]
for label,old_dir,old_name,fresh_name in CASES:
    entry={'target':label}; raw={}
    paths={'old':EV/old_dir/'comas'/old_name,'fresh':CHECK/'comas/current'/fresh_name}
    for kind,path in paths.items():
        outdir=CHECK/'full-stdout'/label/kind;outdir.mkdir(parents=True,exist_ok=True)
        cmd=['why3','prove','-L',LIB,'-a','split_vc','-D','why3',str(path)]
        p=subprocess.run(cmd,cwd=HARNESS,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        (outdir/'stdout.bin').write_bytes(p.stdout);(outdir/'stderr.bin').write_bytes(p.stderr);(outdir/'exit-status.txt').write_text(str(p.returncode)+'\n')
        entry[kind]={'input':str(path.relative_to(ROOT)),'input_bytes':path.stat().st_size,'input_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'command':cmd,'exit_code':p.returncode,'stdout_file':str((outdir/'stdout.bin').relative_to(CHECK)),'stdout_bytes':len(p.stdout),'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_file':str((outdir/'stderr.bin').relative_to(CHECK)),'stderr_bytes':len(p.stderr),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()}
        raw[kind]=p.stdout
    entry['full_stdout_byte_identical']=entry['old']['exit_code']==entry['fresh']['exit_code']==0 and raw['old']==raw['fresh']
    rows.append(entry)
obj={'schema_version':1,'method':'For each archived and fresh COMA separately, invoke Why3 split_vc with -D why3 and without -o. Capture the complete raw task-printer stdout byte stream; compare bytes without normalization or regrouping.','cases':rows,'all_six_full_stdout_streams_byte_identical':all(x['full_stdout_byte_identical'] for x in rows),'stderr_policy':'Retain stderr for diagnostics; only raw stdout is used to establish task stream identity.'}
(CHECK/'full-stdout-comparison.json').write_text(json.dumps(obj,indent=2)+'\n')
print(json.dumps({'all_six_full_stdout_streams_byte_identical':obj['all_six_full_stdout_streams_byte_identical'],'bytes':{r['target']:r['old']['stdout_bytes'] for r in rows}},indent=2))
if any(r['old']['exit_code'] or r['fresh']['exit_code'] for r in rows):sys.exit(1)
if not obj['all_six_full_stdout_streams_byte_identical']:sys.exit(2)
