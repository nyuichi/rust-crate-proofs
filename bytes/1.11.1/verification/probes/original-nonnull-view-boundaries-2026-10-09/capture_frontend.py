#!/usr/bin/env python3
"""Snapshot a failed frontend input without treating stale solver files as evidence."""
import argparse,hashlib,json,pathlib,tarfile
R=pathlib.Path(__file__).resolve().parent
P=R.parents[4]
sha=lambda b:hashlib.sha256(b).hexdigest()
a=argparse.ArgumentParser();a.add_argument('label');a.add_argument('log');args=a.parse_args()
archive=R/'evidence'/(args.label+'.tar.gz');receipt=archive.with_suffix('').with_suffix('.json')
assert not archive.exists() and not receipt.exists(),'immutable label exists'
files={}
def tree(base,prefix):
 for p in base.rglob('*'):
  if p.is_file() and not any(x in {'evidence','verif','target','.git','__pycache__','.proof-config','.why3findcache','.why3find','native-mir'} for x in p.relative_to(base).parts):
   files[prefix+'/'+p.relative_to(base).as_posix()]=p
# All proof/support sources are captured; proof artifacts are deliberately excluded.
tree(R,'probe')
tree(P/'bytes/1.11.1/src','inputs/repository/bytes/1.11.1/src')
for name in ['shared-physical-lifecycle-2026-10-08','original-public-shared-gate-2026-10-08','scoped-issuance-cursor-2026-10-09','original-shared-lifecycle-2026-10-08','guarded-shared-protocol-2026-10-08','original-shared-scoped-client-2026-10-09','original-public-constructor-gate-2026-10-09','original-boxed-automatic-drop-2026-10-09','original-promotable-first-clone-2026-10-09','original-shared-automatic-drop-2026-10-09','original-promotable-automatic-drop-2026-10-09','original-promotable-reclone-2026-10-09','original-promotable-surviving-child-2026-10-09','original-shared-finite-owners-2026-10-09','original-shared-slice-views-2026-10-09']:
 tree(P/'bytes/1.11.1/verification/probes'/name,'inputs/repository/bytes/1.11.1/verification/probes/'+name)
ar=P/'bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09'
tree(ar,'inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09')
for name in ['AR_CANONICAL_AUDIT.json','AR_CANONICAL_AUDIT.md','ar-positive-canonical-v2.tar.gz','ar-positive-canonical-v2.json']:
 files['inputs/repository/bytes/1.11.1/verification/probes/original-bytes-cursor-closure-2026-10-09/evidence/'+name]=ar/'evidence'/name
tree(pathlib.Path('/workspace/bytes-proof-tools/bytes-proof-std'),'inputs/private-std')
files['frontend.log']=pathlib.Path(args.log).resolve()
rows=[]
archive.parent.mkdir(exist_ok=True)
with tarfile.open(archive,'w:gz') as tar:
 for name,path in sorted(files.items()):
  tar.add(path,arcname=name,recursive=False);rows.append(dict(path=name,sha256=sha(path.read_bytes())))
result=dict(status='frontend_diagnostic_no_proof_claim',archive=archive.name,archive_sha256=sha(archive.read_bytes()),members=rows,proof_artifacts='excluded deliberately; no stale solver evidence',scope='AS nonnull-boundary frontend development capture with published AR ancestry; no proof admission.')
receipt.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['status','archive','archive_sha256']}))
