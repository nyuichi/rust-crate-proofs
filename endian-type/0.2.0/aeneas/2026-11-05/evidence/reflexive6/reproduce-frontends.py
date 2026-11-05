from pathlib import Path
import os,json,hashlib,subprocess,tempfile,shutil
p=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
for path,expected in json.loads((p/'INPUT-PINS.json').read_text()).items():assert sha((p/path).read_bytes())==expected,path
for tool,pin in [('charon','79dfdb2fba1fc991dc336d056151c6c06f016462a83885b4325cff93085baa55'),('aeneas','f37fd52cd42a0ee78b5bdaa70bc518caa07bba3ed54fd9759eb2c0a58451b8a8')]:assert sha(Path(shutil.which(tool)).resolve().read_bytes())==pin,tool
compiler=subprocess.check_output(['rustc','-vV'],text=True);assert 'commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8' in compiler and 'host: x86_64-unknown-linux-gnu' in compiler
with tempfile.TemporaryDirectory(prefix='endian-reflexive6-') as name:
 out=Path(name);env=os.environ.copy()
 for v in ['RUSTC','RUSTDOC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET']:env.pop(v,None)
 env.update(CARGO_NET_OFFLINE='true',RUSTUP_TOOLCHAIN='nightly-2026-09-17',RUSTFLAGS='-Zmir-enable-passes=+Inline,+RemoveStorageMarkers,+ReferencePropagation,+DeadStoreElimination-final,+SimplifyLocals-final')
 cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--mir','optimized','--dest',str(out)]
 for method in ['big_from','big_borrow','big_borrow_mut','little_from','little_borrow','little_borrow_mut']:cmd+=['--start-from','crate::'+method]
 cmd+=['--include','endian_type::BigEndian','--include','endian_type::LittleEndian']
 subprocess.run(cmd,cwd=p,env=env,check=True)
 assert (out/'endian_reflexive_original.llbc').read_bytes()==(p/'endian_reflexive_original.llbc').read_bytes(),'LLBC differs'
 subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(out/'generated'),str(out/'endian_reflexive_original.llbc')],env=env,check=True)
 for file in ['EndianReflexiveOriginal.lean','translation.json']:assert (out/'generated'/file).read_bytes()==(p/'generated'/file).read_bytes(),file
 print('all original LLBC/Lean/metadata exact regenerated')
