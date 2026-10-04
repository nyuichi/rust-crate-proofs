#!/usr/bin/env python3
"""Require intended unproved VCs, never syntax/setup failures, for proof negatives."""
import hashlib, json, pathlib, re, shutil, subprocess
root = pathlib.Path(__file__).resolve().parents[1]
artifacts = root/'verification/artifacts'
logs = artifacts/'logs'; logs.mkdir(exist_ok=True)
evidence = artifacts/'evidence'; evidence.mkdir(exist_ok=True)
cases = [('helpers','wrong_postcondition','wrong_postcondition'),
         ('helpers','reachable_false','reachable_false'),
         ('storage','wrong_byte','wrong_byte'),
         ('storage','missing_ownership','recover_without_ownership'),
         ('deallocation','wrong_layout','wrong_layout'),
         ('deallocation','wrong_capacity','wrong_capacity')]
results=[]
for target, feature, goal in cases:
    command=[str(root/'scripts/verify-bytes.sh'),target,'--features',feature]
    run=subprocess.run(command,cwd=root,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
    log=logs/f'{target}-{feature}.log';log.write_text(run.stdout)
    intended=(run.returncode!=0 and re.search(r'Goal Coma\.vc_'+re.escape(goal)+r'\b.*✘',run.stdout)
              and 'unproved file' in run.stdout and 'Compilation failed' not in run.stdout)
    if not intended:
        raise SystemExit(f'{target}:{feature}: expected VC rejection missing; see {log}')
    generated=root/'verification/probes'/target/'verif'
    hashes={}
    for p in generated.rglob('*'):
        if p.is_file() and p.suffix=='.coma':
            dest=evidence/f'{target}-{feature}'/p.relative_to(generated)
            dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest)
            hashes[str(dest.relative_to(root))]=hashlib.sha256(dest.read_bytes()).hexdigest()
    results.append(dict(target=target,feature=feature,expected_goal=goal,exit_code=run.returncode,
                        command=command,log=str(log.relative_to(root)),vc_hashes=hashes))
    print(f'{target}:{feature}: intended VC rejected',flush=True)
(artifacts/'negative-results.json').write_text(json.dumps(results,indent=2)+'\n')
