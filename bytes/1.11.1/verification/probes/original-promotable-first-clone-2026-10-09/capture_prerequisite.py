#!/usr/bin/env python3
"""Immutable all-target capture for the isolated owned-pointer diagnostic crate."""
import argparse,hashlib,importlib.util,json,pathlib,tarfile
R=pathlib.Path(__file__).resolve().parent; M=R/'prerequisites/owned-pointer';T=pathlib.Path('/workspace/bytes-proof-tools')
sha=lambda b:hashlib.sha256(b).hexdigest()
a=argparse.ArgumentParser();a.add_argument('label');a.add_argument('log');args=a.parse_args()
archive=R/'evidence'/(args.label+'.tar.gz');receipt=R/'evidence'/(args.label+'.json')
assert not archive.exists() and not receipt.exists()
files={}
def tree(base,prefix,skip=()):
 for p in base.rglob('*'):
  if p.is_file() and not any(x in skip for x in p.relative_to(base).parts):files[prefix+'/'+p.relative_to(base).as_posix()]=p
# Capture only the mini crate's complete dependency closure, not concurrently edited AL integration.
tree(M,'probe/prerequisites/owned-pointer',('target','__pycache__','.why3findcache','.why3find'))
files['probe/src/owned_pointer.rs']=R/'src/owned_pointer.rs'
files['probe/capture_prerequisite.py']=R/'capture_prerequisite.py'
files['inputs/repository/bytes/1.11.1/verification/probes/original-shared-lifecycle-2026-10-08/src/pointer_event.rs']=R.parent/'original-shared-lifecycle-2026-10-08/src/pointer_event.rs'
tree(T/'bytes-proof-std','inputs/private-std',('target','.git','__pycache__'))
for name,path in [('installation-manifest.json',T/'installation-manifest.json'),('activate.sh',T/'activate.sh'),('cargo-config.toml',T/'cargo/config.toml'),('creusot_why3.conf',T/'creusot-data/creusot_why3.conf'),('base-why3.conf',T/'config/creusot/why3.conf')]:files['inputs/tools/'+name]=path
files['inputs/tools/base-activate.sh']=pathlib.Path('/workspace/proof-tools/activate.sh')
files['run.log']=pathlib.Path(args.log).resolve()
policy=json.loads((M/'generated/proof-targets.json').read_text());actual=sorted(p.relative_to(M).as_posix() for p in (M/'verif').rglob('*.coma'))
assert actual==policy['included'] and policy['excluded']=={}
statistics=dict(files=0,prover=0,null=0,structural=0)
def visit(x):
 if x is None:statistics['null']+=1
 elif 'children' in x:
  if not x['children']:statistics['structural']+=1
  for y in x['children']:visit(y)
 elif 'prover' in x:statistics['prover']+=1
 else:raise ValueError(x)
for c in actual:
 p=M/pathlib.Path(c).with_suffix('')/'proof.json';d=json.loads(p.read_text());statistics['files']+=1
 for v in d['proofs']['Coma'].values():visit(v)
assert statistics['null']>0,'negative diagnostic must actually fail'
rows=[]
with tarfile.open(archive,'w:gz') as tar:
 for name,path in sorted(files.items()):tar.add(path,arcname=name,recursive=False);rows.append(dict(path=name,sha256=sha(path.read_bytes())))
r=dict(status='deliberate_semantic_rejection',archive=archive.name,archive_sha256=sha(archive.read_bytes()),members=rows,statistics=statistics,target_policy=policy,scope='Generic owned-pointer prerequisite only; no native Bytes lifecycle/fullcrate admission')
receipt.write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:r[k] for k in ['status','archive','archive_sha256','statistics']}))
