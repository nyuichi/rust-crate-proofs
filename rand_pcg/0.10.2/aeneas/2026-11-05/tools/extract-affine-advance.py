from pathlib import Path
import os,json,hashlib,tarfile,tomllib,subprocess,re
root=Path(__file__).resolve().parents[1];archive=root/'evidence/upstream.crate'
def source_hashes():
 with tarfile.open(archive) as t:
  result={}
  for m in t.getmembers():
   if m.isfile() and (m.name.endswith('.rs') or m.name.endswith('/Cargo.toml')):
    rel=Path(m.name).relative_to('rand_pcg-0.10.2');data=t.extractfile(m).read();assert (root/rel).read_bytes()==data,str(rel);result[str(rel)]=hashlib.sha256(data).hexdigest()
 return result
before=source_hashes();assert next(x['version'] for x in tomllib.loads((root/'Cargo.lock').read_text())['package'] if x['name']=='rand_core')=='0.10.0'
roots=['crate::pcg64::Lcg64Xsh32::advance','crate::pcg128::Lcg128Xsl64::advance','crate::pcg128cm::Lcg128CmDxsm64::advance'];commands=[]
for mode in ['default','serde']:
 out=root/'evidence'/('affine-advance-'+mode);out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset','aeneas','--sysroot','default','--dest-file',str(out/'pcg_affine_advance.llbc')]
 for selected in roots:args+=['--start-from',selected]
 if mode=='serde':args+=['--','--features','serde']
 lean=['aeneas','-backend','lean','-emit-json','-split-files','-namespace','PcgAffineAdvanceModel','-abort-on-error','-dest',str(out/'generated'),str(out/'pcg_affine_advance.llbc')]
 for command,log in [(args,'charon.log'),(lean,'aeneas.log')]:
  with (out/log).open('w') as f:subprocess.run(command,cwd=root,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=f,stderr=subprocess.STDOUT,check=True)
 j=json.loads((out/'pcg_affine_advance.llbc').read_text());assert not j['has_errors'];assert j['translated']['target_information'][0]['key']=='x86_64-unknown-linux-gnu'
 assert not re.search(r'\b(sorry|axiom)\b','\n'.join(p.read_text() for p in (out/'generated').glob('*.lean')))
 commands.append({'configuration':mode,'charon':args,'aeneas':lean,'roots':roots,'apis':['rand_pcg::'+x+'::advance' for x in ['Lcg64Xsh32','Lcg128Xsl64','Lcg128CmDxsm64']],'namespace':'PcgAffineAdvanceModel'})
rows=[]
for name in ['Types.lean','Funs.lean']:
 a=(root/'evidence/affine-advance-default/generated'/name).read_bytes();b=(root/'evidence/affine-advance-serde/generated'/name).read_bytes();assert a==b,name;(root/'proofs/PcgAffineAdvance'/name).write_bytes(a);rows.append({'file':name,'sha256':hashlib.sha256(a).hexdigest(),'default_serde_byte_identical':True})
(root/'proofs/PcgAffineAdvance/translation.json').write_bytes((root/'evidence/affine-advance-default/generated/translation.json').read_bytes())
assert before==source_hashes()
(root/'evidence/affine-advance-provenance.json').write_text(json.dumps({'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'source_sha256':before,'commands':commands,'equivalence':rows},indent=2))
print('Three original LCG advance methods: exact direct root, original archive/source unchanged, distinct official namespace, default/serde byte-identical, no unknown externals')
