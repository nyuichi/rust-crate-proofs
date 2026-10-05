#!/usr/bin/env python3
"""Print frozen phase-two Why3 tasks with the non-prover Why3 driver."""
import gzip, hashlib, json, os, re, subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
sha=lambda b: hashlib.sha256(b).hexdigest()
freeze=json.loads((HERE/'source-freeze.json').read_text())
for row in freeze['sources']:
    data=(HERE/'sources'/row['path']).read_bytes()
    assert sha(data)==row['sha256'] and len(data)==row['bytes']
selection=json.loads((HERE/'target-selection.json').read_text())
base=['/workspace/proof-tools/creusot-data/bin/why3','prove','-C','/workspace/proof-tools/config/creusot/why3.conf','-L','/workspace/proof-tools/creusot-data/share/why3find/packages/creusot','-D','why3']
results=[]
for i,row in enumerate(selection['comas']):
    coma=HERE/row['archive']; assert sha(coma.read_bytes())==row['sha256']
    out=HERE/f'printed/target-{i:02}'; out.mkdir(parents=True,exist_ok=True)
    command=base+[str(coma)]
    run=subprocess.run(command,capture_output=True)
    assert run.returncode==0,run.stderr.decode(errors='replace')
    goals=re.findall(rb'^goal ([^ ]+) :',run.stdout,re.M)
    packed=[]
    for name,data in [('stdout',run.stdout),('stderr',run.stderr)]:
        p=out/f'{name}.gz'; p.write_bytes(gzip.compress(data,mtime=0)); packed.append({'path':str(p.relative_to(HERE)),'raw_sha256':sha(data),'raw_bytes':len(data),'gzip_sha256':sha(p.read_bytes())})
    taskdir=out/'tasks'; taskdir.mkdir(exist_ok=True)
    files_command=base+['-o',str(taskdir),str(coma)]
    files=subprocess.run(files_command,capture_output=True)
    assert files.returncode==0,files.stderr.decode(errors='replace')
    tasks=[]
    for p in sorted(taskdir.glob('*.why')):
        d=p.read_bytes(); tasks.append({'path':str(p.relative_to(HERE)),'sha256':sha(d),'bytes':len(d)})
    results.append({'target':row['target'],'coma':{'path':row['archive'],'sha256':row['sha256']},'command':command,'returncode':run.returncode,'goals':[x.decode() for x in goals],'direct_root_count':len(goals),'streams':packed,'files_command':files_command,'files_returncode':files.returncode,'task_count':len(tasks),'tasks':tasks})
report={'scope':'Solver-free frozen-COMA Why3 task printing and direct-root inventory only; no prover results are claimed.','solver_invoked':False,'frontend_invoked':False,'source_modified':False,'source_freeze_sha256':sha((HERE/'source-freeze.json').read_bytes()),'target_selection_sha256':sha((HERE/'target-selection.json').read_bytes()),'source_file_count':len(freeze['sources']),'selected_coma_count':len(results),'results':results}
(HERE/'direct-task-print.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'solver_invoked':False,'source_file_count':report['source_file_count'],'selected_coma_count':len(results),'targets':[{'target':x['target'],'direct_root_count':x['direct_root_count'],'goals':x['goals']} for x in results]},indent=2))
