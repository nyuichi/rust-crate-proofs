#!/usr/bin/env python3
import argparse, hashlib, json, pathlib, shutil, tarfile, re
p=argparse.ArgumentParser(); p.add_argument('label'); p.add_argument('--proof-log',required=True); p.add_argument('--exit',type=int,required=True); p.add_argument('--kind',choices=['positive','negative','frontend'],required=True); p.add_argument('--features',default=''); a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent
out=root/'evidence'/a.label
if out.exists(): raise SystemExit('refuse overwrite')
out.mkdir(parents=True)
for name in ['src','Cargo.toml','Cargo.lock','run-proof.sh','why3find.json','capture.py','README.md']:
 s=root/name; d=out/name
 if s.is_dir(): shutil.copytree(s,d)
 else: shutil.copy2(s,d)
shutil.copy2(a.proof_log,out/'proof.log')
for name in ['/tmp/unsafe-affine-native-final.log','/tmp/unsafe-affine-native.log']:
 if pathlib.Path(name).exists(): shutil.copy2(name,out/pathlib.Path(name).name)
if a.kind!='frontend' and (root/'verif').exists(): shutil.copytree(root/'verif',out/'verif')
std=pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std')
for name in ['src/std/ops.rs','src/ghost.rs','src/ghost/fn_ghost.rs','src/ghost/resource.rs','src/logic/ra/excl.rs','Cargo.toml']:
 s=std/name; d=out/'std-analogue'/name; d.parent.mkdir(parents=True,exist_ok=True); shutil.copy2(s,d)
conf=root/'.proof-config/creusot/why3.conf'
if conf.exists(): shutil.copy2(conf,out/'why3.conf')
comas=list(out.glob('verif/**/*.coma')); proofs=list(out.glob('verif/**/proof.json'))
def nulls(x):
 if x is None:return 1
 if isinstance(x,dict):return sum(nulls(v) for v in x.values())
 if isinstance(x,list):return sum(nulls(v) for v in x)
 return 0
n=sum(nulls(json.loads(f.read_text())) for f in proofs)
log=pathlib.Path(a.proof_log).read_text()
if a.kind=='positive':
 assert a.exit==0 and len(comas)==len(proofs)>0 and n==0
 assert f'Proved ({len(comas)} files)' in log
 assert all(f.with_suffix('').joinpath('proof.json').exists() for f in comas)
if a.kind=='negative':assert a.exit!=0 and n>0
files={str(f.relative_to(out)):hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(out.rglob('*')) if f.is_file()}
receipt={'kind':a.kind,'exit':a.exit,'features':a.features,'effective_proof_added_features':['creusot-std/creusot','creusot-std/nightly'],'coma':len(comas),'proof_json':len(proofs),'recursive_nulls':n,'sha256':files,'scope':'Restricted scalar unsafe dispatch; resource is moved unchanged by checked wrapper. Not general ghost-transforming callback/public Bytes integration.'}
(out/'manifest.json').write_text(json.dumps(receipt,indent=2)+'\n')
archive=out.with_suffix('.tar.gz')
with tarfile.open(archive,'w:gz') as t:t.add(out,arcname=a.label)
print(json.dumps({'archive':str(archive),'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'coma':len(comas),'proof_json':len(proofs),'nulls':n}))
