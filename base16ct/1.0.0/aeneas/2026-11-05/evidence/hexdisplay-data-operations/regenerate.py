from pathlib import Path
import subprocess,tempfile,json,hashlib
p=Path(__file__).resolve().parent
for n,h in json.loads((p/'original-pins.json').read_text()).items():assert hashlib.sha256((p/n).read_bytes()).hexdigest()==h
with tempfile.TemporaryDirectory() as t:
 out=Path(t)
 subprocess.run(['charon','cargo','--preset','aeneas','--sysroot','default','--dest',str(out),'--start-from','crate::construct','--start-from','crate::project','--start-from','crate::construct_project'],cwd=p,check=True)
 subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(out/'generated'),str(out/'hexdisplay_constructor.llbc')],check=True)
 old='/workspace/full-api/enum/base16ct/optimized-inline-probe/original'
 for name in ['HexdisplayConstructor.lean','translation.json']:
  a=(out/'generated'/name).read_text().replace(str(p/'original'),old).replace('original/src/',old+'/src/')
  assert a==(p/'generated'/name).read_text(),name
 subprocess.run(['lake','build','ConstructorProofs'],cwd=p/'proofs',check=True)
 print('PASS exact generated output after original dependency source-prefix relocation only; kernel proofs built')
