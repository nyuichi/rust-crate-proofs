from pathlib import Path
import json,os,subprocess,hashlib
root=Path(__file__).resolve().parents[1];os.chdir(root)
ev=root/'evidence/serde';ev.mkdir(exist_ok=True)
args=json.loads((ev/'charon-command.json').read_text())
with (ev/'charon.log').open('w') as log:
 subprocess.run(args,check=True,stdout=log,stderr=subprocess.STDOUT,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
with (ev/'aeneas.log').open('w') as log:
 subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','evidence/serde/generated','evidence/serde/rand_pcg.llbc'],check=True,stdout=log,stderr=subprocess.STDOUT)
a=(root/'proofs/RandPcg.lean').read_bytes();b=(ev/'generated/RandPcg.lean').read_bytes()
assert a==b,'Selected serde/default translations differ'
result={'configuration':['serde'],'target':'x86_64-unknown-linux-gnu','byte_identical':True,'sha256':hashlib.sha256(a).hexdigest(),'scope':'Only the selected original body callgraph; serde trait implementations themselves remain unproved.'}
(ev/'translation-equivalence.json').write_text(json.dumps(result,indent=2))
print('Selected original methods have byte-identical official translations with and without serde')
