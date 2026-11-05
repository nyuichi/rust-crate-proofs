from pathlib import Path
import subprocess,json,re,hashlib
p=Path(__file__).resolve().parents[1];meta=json.loads((p/'evidence/integration69-inputs.json').read_text());mods=meta['configured_modules'];logs=[]
def run(cmd,input=None):
 r=subprocess.run(cmd,cwd=p/'proofs',input=input,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);logs.append(r.stdout);(p/'evidence/integration69.log').write_text('\n'.join(logs));assert r.returncode==0,(cmd,r.returncode)
run(['lake','build'])
run(['lake','env','lean','--stdin'],'\n'.join('import '+m for m in mods)+'\n')
(p/'evidence/simultaneous-import69.log').write_text(logs[-1])
for m in mods:run(['lake','env','lean',m+'.lean'])
text='\n'.join(logs);names=[]
for m in mods:names+=re.findall(r'^#print axioms ([\w.]+)',(p/'proofs'/f'{m}.lean').read_text(),re.M)
assert len(names)==meta['audit_count']
for name in names:
 hits=re.findall("'"+re.escape(name)+r"' depends on axioms: \[(.*?)\]",text,re.S)
 assert hits or "'"+name+"' does not depend on any axioms"in text,name
 for h in hits:assert set(x.strip()for x in h.split(','))<={'propext','Classical.choice','Quot.sound'},(name,h)
meta['status']='integrated proof audit successful; external25 source-discovery proposal unimplemented, no publication claim';meta['all_configured_modules_simultaneously_imported']=True
(p/'evidence/integration69-completed.json').write_text(json.dumps(meta,indent=2))
print(f'69 unique API bindings, {len(mods)} simultaneous modules, {len(names)} standard-three theorem audits successful; publication source discovery remains unsupported')
