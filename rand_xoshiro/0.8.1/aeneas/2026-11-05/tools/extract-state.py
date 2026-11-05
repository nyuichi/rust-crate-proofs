from pathlib import Path
import json,subprocess,os,hashlib,re
root=Path(__file__).resolve().parents[1];os.chdir(root)
roots=['crate::splitmix64::SplitMix64::state']
for mode in ['default','serde']:
 out=root/'evidence'/('splitmix-state-'+mode);out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file',str(out/'split_mix_state.llbc')]
 for x in roots:args+=['--start-from',x]
 if mode=='serde':args+=['--','--features','serde']
 with (out/'charon.log').open('w') as log:subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=log,stderr=subprocess.STDOUT)
 with (out/'aeneas.log').open('w') as log:subprocess.run(['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(out/'generated'),str(out/'split_mix_state.llbc')],check=True,stdout=log,stderr=subprocess.STDOUT)
 assert not json.loads((out/'split_mix_state.llbc').read_text())['has_errors']
 assert not re.search(r'\b(sorry|axiom)\b','\n'.join(p.read_text() for p in (out/'generated').glob('*.lean')))
rows=[]
for name in ['Types.lean','Funs.lean']:
 a=(root/'evidence/splitmix-state-default/generated'/name).read_bytes();b=(root/'evidence/splitmix-state-serde/generated'/name).read_bytes();assert a==b,'Serde/default SplitMixState models differ'
 (root/'proofs/SplitMixState'/name).write_bytes(a);rows.append({'file':name,'sha256':hashlib.sha256(a).hexdigest(),'byte_identical':True})
(root/'proofs/SplitMixState/translation.json').write_bytes((root/'evidence/splitmix-state-default/generated/translation.json').read_bytes())
(root/'evidence/splitmix-state-equivalence.json').write_text(json.dumps(rows,indent=2))
print('Original SplitMix64 state body: default/serde byte-identical, no unknown externals')
