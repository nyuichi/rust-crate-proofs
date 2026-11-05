from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile
p=Path(__file__).resolve().parent
sha=lambda x:hashlib.sha256(x).hexdigest()
for file,expected in json.loads((p/'INPUT-PINS.json').read_text()).items():assert sha((p/file).read_bytes())==expected,file
for tool,expected in [('charon','79dfdb2fba1fc991dc336d056151c6c06f016462a83885b4325cff93085baa55'),('aeneas','f37fd52cd42a0ee78b5bdaa70bc518caa07bba3ed54fd9759eb2c0a58451b8a8')]:assert sha(Path(shutil.which(tool)).resolve().read_bytes())==expected,tool
compiler=subprocess.check_output(['rustc','-vV'],text=True);assert 'commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8' in compiler and 'host: x86_64-unknown-linux-gnu' in compiler
cases=[('std-debug','nohash_string_inline','NohashStringInline','generated-reference',False),('nostd-debug','nohash_string_nostd_debug','NohashStringNostdDebug','generated',False),('nostd-release','nohash_string_nostd_release','NohashStringNostdRelease','generated',True),('std-release','nohash_string_std_release','NohashStringStdRelease','generated',True)]
for folder,crate,module,generated,release in cases:
 with tempfile.TemporaryDirectory(prefix='nohash-string-replay-') as d:
  out=Path(d);env=os.environ.copy()
  for k in ['RUSTC','RUSTDOC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET']:env.pop(k,None)
  env.update(RUSTUP_TOOLCHAIN='nightly-2026-09-17',CARGO_NET_OFFLINE='true',RUSTFLAGS='-Zmir-enable-passes=+Inline,+RemoveStorageMarkers,+ReferencePropagation,+DeadStoreElimination-final,+SimplifyLocals-final')
  cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--mir','optimized','--dest',str(out),'--start-from','crate::write_text','--include','nohash_hasher::NoHashHasher']
  if release:cmd+=['--','--release']
  subprocess.run(cmd,cwd=p/folder,env=env,check=True)
  subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(out/'generated'),str(out/(crate+'.llbc'))],env=env,check=True)
  for file in [module+'.lean','translation.json']:
   actual=(out/'generated'/file).read_text().replace(str((p/'original').resolve()),'/workspace/full-api/endian/nohash-length-prefix/original')
   assert actual==(p/folder/generated/file).read_text(),(folder,file)
  print('exact regenerated output; original-source prefix relocation only:',folder)
