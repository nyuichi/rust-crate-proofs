#!/usr/bin/env python3
"""Capture this downstream crate, its exact proved bytes dependency, and a completed run."""
from pathlib import Path
import hashlib, io, json, re, sys, tarfile, os
root=Path(__file__).resolve().parent
crate=root.parents[2]
name,kind,code,prooflog,nativelog=sys.argv[1:]
code=int(code)
sha=lambda b:hashlib.sha256(b).hexdigest()
files={}
baseline=os.environ.get('BYTES_PROVEN_RUN','adapters-public-spec-255')
base=crate/'verification/modified-variant-evidence/runs/positive'/baseline/'evidence.tar.gz'
with tarfile.open(base) as tar:
 for member in tar.getmembers():
  if member.isfile() and member.name.startswith('configuration/'):
   files['dependency-'+member.name]=tar.extractfile(member).read()
  if member.isfile() and member.name.startswith('production-source/'):
   rel=member.name.removeprefix('production-source/')
   data=tar.extractfile(member).read()
   if rel.startswith('src/') or rel in ('Cargo.toml','Cargo.lock'):
    assert (crate/rel).read_bytes()==data, 'dependency differs from proved255: '+rel
   files['dependency/'+rel]=data
for rel in ['Cargo.toml','Cargo.lock','src/lib.rs','verify.sh','capture.py','why3find.json']:
 files['consumer/'+rel]=(root/rel).read_bytes()
for label,path in [('proof',prooflog),('native',nativelog)]:files['logs/'+label+'.log']=Path(path).read_bytes()
for label in ['native','proof']:
 path=Path('/tmp/bytes-external-'+label+'-features.log')
 if path.exists(): files['logs/'+label+'-effective-features.log']=path.read_bytes()
assert Path('/workspace/bytes-proof-tools/creusot-source/creusot-std/src/std/vec.rs').read_bytes()==files['dependency-configuration/creusot-std-vec-contracts.rs'], 'std TCB changed'
for path in (root/'verif').rglob('*'):
 if path.is_file():files['verif/'+str(path.relative_to(root/'verif'))]=path.read_bytes()
def nulls(x):
 if x is None:return 1
 if isinstance(x,dict):return sum(nulls(v) for v in x.values())
 if isinstance(x,list):return sum(nulls(v) for v in x)
 return 0
comas=[n for n in files if n.endswith('.coma')]
proofs=[n for n in files if n.endswith('/proof.json')]
null_count=sum(nulls(json.loads(files[n]).get('proofs',{})) for n in proofs)
if kind=='positive':
 assert code==0 and comas and len(comas)==len(proofs) and null_count==0
 assert all(n[:-5]+'/proof.json' in files for n in comas)
 assert re.search(r'Proved\s*\('+str(len(comas))+r' files\)',files['logs/proof.log'].decode())
record={'kind':kind,'exit_code':code,'scope':'Actual downstream crate calling modified bytes public contracts; bytes dependency body proof is archived255. No full-coverage claim.',
 'dependency_positive_run':baseline,'dependency_positive_archive_sha256':sha(base.read_bytes()),'dependency_source_correspondence':'Every captured src/Cargo dependency file equals the proved255 bytes.',
 'command':'./verify.sh','native_command':'cargo test --locked','bytes_features':['verified','std'],'implicit_proof_features':['creusot-std/creusot','creusot-std/nightly'],
 'counts':{'coma':len(comas),'proof_json':len(proofs),'null_leaves':null_count},'full_coverage':False}
files['run.json']=json.dumps(record,indent=2).encode()
manifest={n:{'sha256':sha(b),'bytes':len(b)} for n,b in sorted(files.items())}
files['members.json']=json.dumps(manifest,indent=2).encode()
out=root/'evidence'/name
out.mkdir(parents=True,exist_ok=False)
archive=out/'evidence.tar.gz'
with tarfile.open(archive,'w:gz') as tar:
 for n,b in sorted(files.items()):
  info=tarfile.TarInfo(n);info.size=len(b);info.mode=0o644;tar.addfile(info,io.BytesIO(b))
with tarfile.open(archive) as tar:
 for n,meta in manifest.items():assert sha(tar.extractfile(n).read())==meta['sha256']
receipt={**record,'archive_sha256':sha(archive.read_bytes()),'members_verified':len(manifest),'members':manifest}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:v for k,v in receipt.items() if k!='members'},indent=2))
