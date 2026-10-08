#!/usr/bin/env python3
"""Immutable complete local/public-gate diagnostic snapshot; no success inference."""
import argparse,hashlib,json,pathlib,tarfile
p=argparse.ArgumentParser();p.add_argument('label');p.add_argument('log');p.add_argument('--positive',action='store_true');a=p.parse_args()
r=pathlib.Path(__file__).resolve().parent;c=r.parents[2];out=r/'evidence'/(a.label+'.tar.gz');assert not out.exists()
if a.positive:
 summary=json.loads((r/'generated/public-proof-summary.json').read_text())
 for name,expected in summary['source_sha256'].items():
  assert hashlib.sha256((r/name).read_bytes()).hexdigest()==expected,('source mismatch',name)
 for row in summary['targets']:
  coma=r/row['coma'];proof=coma.with_suffix('')/'proof.json'
  assert hashlib.sha256(coma.read_bytes()).hexdigest()==row['coma_sha256']
  assert hashlib.sha256(proof.read_bytes()).hexdigest()==row['proof_sha256']
 assert summary['statistics']['null']==0
files={}
def addtree(base,prefix,exclude=()):
 for f in base.rglob('*'):
  if f.is_file() and not any(x in f.relative_to(base).parts for x in exclude):files[prefix+'/'+str(f.relative_to(base))]=f
addtree(r,'probe',('target','evidence','.proof-config','.git'))
addtree(c/'src','inputs/production/src')
for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
 f=c/name
 if f.exists():files['inputs/production/'+name]=f
files['inputs/tool-config/creusot_why3.conf']=pathlib.Path('/workspace/bytes-proof-tools/creusot-data/creusot_why3.conf')
for name in ['guarded-shared-protocol-2026-10-08','shared-physical-lifecycle-2026-10-08','original-shared-lifecycle-2026-10-08']:
 addtree(r.parent/name/'src','inputs/'+name+'/src')
addtree(pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std/src'),'inputs/bytes-proof-std/src')
rows=[]
with tarfile.open(out,'w:gz') as t:
 for name,f in sorted(files.items()):
  t.add(f,arcname=name);rows.append({'path':name,'source':str(f),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
d={'archive':out.name,'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'log':a.log,'positive_source_coherence_checked':a.positive,'members':rows}
(r/'evidence'/(a.label+'.json')).write_text(json.dumps(d,indent=2)+'\n');print(d['archive'],d['sha256'],len(rows))
