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
root_selector='crate::pcg128::Mcg128Xsl64::advance';commands=[]
for mode in ['default','serde']:
 out=root/'evidence'/('advance-'+mode);out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset','aeneas','--sysroot','default','--start-from',root_selector,'--dest-file',str(out/'pcg_advance.llbc')]
 if mode=='serde':args+=['--','--features','serde']
 lean=['aeneas','-backend','lean','-emit-json','-split-files','-namespace','PcgAdvanceModel','-abort-on-error','-dest',str(out/'generated'),str(out/'pcg_advance.llbc')]
 for command,log in [(args,'charon.log'),(lean,'aeneas.log')]:
  with (out/log).open('w') as f:subprocess.run(command,cwd=root,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=f,stderr=subprocess.STDOUT,check=True)
 j=json.loads((out/'pcg_advance.llbc').read_text());assert not j['has_errors'];assert j['translated']['target_information'][0]['key']=='x86_64-unknown-linux-gnu'
 assert not re.search(r'\b(sorry|axiom)\b','\n'.join(p.read_text() for p in (out/'generated').glob('*.lean')))
 commands.append({'configuration':mode,'charon':args,'aeneas':lean,'root':root_selector,'api':'rand_pcg::Mcg128Xsl64::advance','namespace':'PcgAdvanceModel'})
rows=[]
for name in ['Types.lean','Funs.lean']:
 a=(root/'evidence/advance-default/generated'/name).read_bytes();b=(root/'evidence/advance-serde/generated'/name).read_bytes();assert a==b,name;(root/'proofs/PcgAdvance'/name).write_bytes(a);rows.append({'file':name,'sha256':hashlib.sha256(a).hexdigest(),'default_serde_byte_identical':True})
(root/'proofs/PcgAdvance/translation.json').write_bytes((root/'evidence/advance-default/generated/translation.json').read_bytes())
assert before==source_hashes()
(root/'evidence/advance-provenance.json').write_text(json.dumps({'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'source_sha256':before,'commands':commands,'equivalence':rows},indent=2))
print('Original Mcg128Xsl64::advance: exact direct root, original archive/source unchanged, distinct official namespace, default/serde byte-identical, no unknown externals')
