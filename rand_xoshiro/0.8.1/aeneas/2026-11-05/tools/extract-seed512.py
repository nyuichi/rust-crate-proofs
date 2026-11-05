from pathlib import Path
import json,subprocess,os,hashlib,re
root=Path(__file__).resolve().parents[1];os.chdir(root)
roots=['{impl core::default::Default for crate::common::Seed512}::default','{impl core::convert::AsRef for crate::common::Seed512}::as_ref','{impl core::convert::AsMut for crate::common::Seed512}::as_mut','crate::common::Seed512::iter']
for mode in ['default','serde']:
 out=root/'evidence'/('seed512-'+mode);out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file',str(out/'seed512.llbc')]
 for x in roots:args+=['--start-from',x]
 if mode=='serde':args+=['--','--features','serde']
 with (out/'charon.log').open('w') as log:subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=log,stderr=subprocess.STDOUT)
 with (out/'aeneas.log').open('w') as log:subprocess.run(['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(out/'generated'),str(out/'seed512.llbc')],check=True,stdout=log,stderr=subprocess.STDOUT)
 assert not json.loads((out/'seed512.llbc').read_text())['has_errors']
 assert not re.search(r'\b(sorry|axiom)\b','\n'.join(p.read_text() for p in (out/'generated').glob('*.lean')))
rows=[]
for name in ['Types.lean','Funs.lean']:
 a=(root/'evidence/seed512-default/generated'/name).read_bytes();b=(root/'evidence/seed512-serde/generated'/name).read_bytes();assert a==b,'Serde/default Seed512 models differ'
 (root/'proofs/Seed512'/name).write_bytes(a);rows.append({'file':name,'sha256':hashlib.sha256(a).hexdigest(),'byte_identical':True})
(root/'proofs/Seed512/translation.json').write_bytes((root/'evidence/seed512-default/generated/translation.json').read_bytes())
(root/'evidence/seed512-equivalence.json').write_text(json.dumps(rows,indent=2))
print('All four original Seed512 bodies: default/serde byte-identical, no unknown externals')
