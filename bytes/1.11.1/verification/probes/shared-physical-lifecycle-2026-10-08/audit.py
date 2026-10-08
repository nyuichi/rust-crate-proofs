"""Read-only archive/member/proof audit; runs no prover."""
import pathlib,tarfile,json,hashlib,re
p=pathlib.Path(__file__).resolve().parent
report={}
def nulls(x):
 if x is None:return 1
 if isinstance(x,dict):return sum(nulls(v) for v in x.values())
 if isinstance(x,list):return sum(map(nulls,x))
 return 0
for archive in sorted((p/'evidence').glob('*.tar.gz')):
 with tarfile.open(archive) as t:
  files={m.name:t.extractfile(m).read() for m in t.getmembers() if m.isfile()}
  manifests=[k for k in files if k.endswith('/hashes.json')]
  if not manifests:continue
  manifest=manifests[0];root=manifest[:-len('hashes.json')]
  hashes=json.loads(files[manifest])
  assert all(hashlib.sha256(files[root+k]).hexdigest()==h for k,h in hashes.items()),archive
  coma=[k for k in files if k.endswith('.coma')]
  proofs=[k for k in files if k.endswith('/proof.json')]
  n=sum(nulls(json.loads(files[k]).get('proofs',{})) for k in proofs)
  report[archive.name]={'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'members':len(files),'coma':len(coma),'proof_json':len(proofs),'nulls':n,'member_hashes_match':True}
  if archive.name.startswith('positive'):
   assert len(coma)==len(proofs)>0 and n==0,archive
   logs=b'\n'.join(v for k,v in files.items() if k.endswith('.log'))
   assert f'Proved ({len(coma)} files)'.encode() in logs,archive
  if archive.name=='positive-b-current-74.tar.gz':
   for f in (p/'src').glob('*.rs'):
    assert files[root+'src/'+f.name]==f.read_bytes(),f
   report[archive.name]['current_source_byte_identical']=True
(p/'audit.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
