from pathlib import Path
import json,os,subprocess,hashlib,tarfile
b=Path(__file__).resolve().parents[1];root=b;archive=b/'evidence/upstream.crate'
with tarfile.open(archive) as t:
 for m in t.getmembers():
  if m.isfile() and (m.name.endswith('.rs') or m.name.endswith('/Cargo.toml') or m.name.endswith('/Cargo.lock')):assert (root/Path(m.name).relative_to('rand_xoshiro-0.8.1')).read_bytes()==t.extractfile(m).read()
entries=json.loads((b/'tools/serializer-engines.json').read_text())
for mode in ['default','serde']:
 out=b/'evidence'/('serializers-'+mode);out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset','aeneas','--sysroot','default','--dest-file',str(out/'serializers.llbc')]
 for e in entries:args+=['--start-from','crate::'+e['module']+'::'+e['type']+'::state']
 if mode=='serde':args+=['--','--features','serde']
 lean=['aeneas','-backend','lean','-split-files','-emit-json','-namespace','XoshiroSerializationModel','-abort-on-error','-dest',str(out/'generated'),str(out/'serializers.llbc')]
 for command,log in [(args,'charon.log'),(lean,'aeneas.log')]:
  with (out/log).open('w') as f:subprocess.run(command,cwd=root,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=f,stderr=subprocess.STDOUT,check=True)
rows=[];(b/'proofs/Serializers').mkdir(exist_ok=True)
for n in ['Types.lean','Funs.lean']:
 a=(b/'evidence/serializers-default/generated'/n).read_bytes();c=(b/'evidence/serializers-serde/generated'/n).read_bytes();assert a==c,n;(b/'proofs/Serializers'/n).write_bytes(a);rows.append({'file':n,'sha256':hashlib.sha256(a).hexdigest(),'byte_identical':True})
(b/'proofs/Serializers/translation.json').write_bytes((b/'evidence/serializers-default/generated/translation.json').read_bytes())
(b/'evidence/serializer-equivalence.json').write_text(json.dumps(rows,indent=2))
print('Original15serializer direct roots; distinct namespace; default/serde generatedTypes/Funs byte-identical')
