from pathlib import Path
import os,subprocess,tempfile,hashlib,json,shutil
p=Path(__file__).resolve().parent
for n,h in json.loads((p/'SOURCE-PINS.json').read_text()).items():assert hashlib.sha256((p/n).read_bytes()).hexdigest()==h,n
for tool,h in [('charon','79dfdb2fba1fc991dc336d056151c6c06f016462a83885b4325cff93085baa55'),('aeneas','f37fd52cd42a0ee78b5bdaa70bc518caa07bba3ed54fd9759eb2c0a58451b8a8')]:assert hashlib.sha256(Path(shutil.which(tool)).resolve().read_bytes()).hexdigest()==h
with tempfile.TemporaryDirectory(prefix='xorshift-reflexive-replay-')as t:
 out=Path(t);env=os.environ.copy()
 for k in ['RUSTC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET']:env.pop(k,None)
 env.update(CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=str(out/'target'),RUSTUP_TOOLCHAIN='nightly-2026-09-17',RUSTFLAGS='-Zmir-enable-passes=+Inline,+RemoveStorageMarkers,+ReferencePropagation,+DeadStoreElimination-final,+SimplifyLocals-final')
 cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--mir','optimized','--dest',str(out)]
 for n in ['original_from','original_borrow','original_borrow_mut']:cmd+=['--start-from','crate::'+n]
 cmd+=['--include','rand_xorshift::XorShiftRng']
 subprocess.run(cmd,cwd=p,env=env,check=True)
 subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(out/'generated'),str(out/'xorshift_reflexive_original.llbc')],env=env,check=True)
 old='/workspace/full-api/prng/kani-xorshift-public-composition/original'
 for n in ['XorshiftReflexiveOriginal.lean','translation.json']:
  text=(out/'generated'/n).read_text().replace(str(p/'original'),old).replace('original/src/',old+'/src/')
  assert text==(p/'generated'/n).read_text(),n
 print('PASS generated Lean/translation exact after fixed original source-prefix relocation only')
