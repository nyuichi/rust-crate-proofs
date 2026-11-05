from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile
p=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
for f,v in json.loads((p/'FAMILY-INPUT-PINS.json').read_text()).items():assert sha((p/f).read_bytes())==v,f
for tool,pin in [('charon','79dfdb2fba1fc991dc336d056151c6c06f016462a83885b4325cff93085baa55'),('aeneas','f37fd52cd42a0ee78b5bdaa70bc518caa07bba3ed54fd9759eb2c0a58451b8a8')]:
 assert sha(Path(shutil.which(tool)).resolve().read_bytes())==pin,tool
rust=subprocess.check_output(['rustc','-vV'],text=True);assert 'commit-hash: 923c95cdf5ba65cea505aa2ea829f578e1506ed8' in rust and 'host: x86_64-unknown-linux-gnu' in rust
core=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip())/'lib/rustlib/src/rust/library/core/src/hash/mod.rs';assert sha(core.read_bytes())==sha((p/'inline-route/core-hash-std09.rs').read_bytes())
cases=[('inline-route','nohash_prefix_inline','NohashPrefixInline','generated',False,False,['prefix']),('inline-wide-panic','nohash_wide_panic_inline','NohashWidePanicInline','generated-dce',False,True,['wide_unsigned','wide_signed']),('inline-nostd-debug','nohash_nostd_debug','NohashNostdDebug','generated',False,True,['prefix','wide_unsigned','wide_signed']),('inline-nostd-release','nohash_nostd_release','NohashNostdRelease','generated',True,True,['prefix','wide_unsigned','wide_signed']),('inline-std-release-all','nohash_std_release_all','NohashStdReleaseAll','generated',True,True,['prefix','wide_unsigned','wide_signed'])]
for folder,crate,module,gen,release,dce,roots in cases:
 with tempfile.TemporaryDirectory(prefix='nohash-family-replay-') as temporary:
  tmp=Path(temporary);env=os.environ.copy()
  for k in ['RUSTC','RUSTDOC','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET']:env.pop(k,None)
  env.update(CARGO_NET_OFFLINE='true',RUSTUP_TOOLCHAIN='nightly-2026-09-17',RUSTFLAGS='-Zmir-enable-passes=+Inline'+(',+SimplifyLocals-final' if dce else ''))
  cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--mir','optimized','--dest',str(tmp),'--include','nohash_hasher::NoHashHasher']
  for root in roots:cmd+=['--start-from','crate::'+root]
  if release:cmd+=['--','--release']
  subprocess.run(cmd,cwd=p/folder,env=env,check=True)
  subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(tmp/'generated'),str(tmp/(crate+'.llbc'))],check=True,env=env)
  for filename in [module+'.lean','translation.json']:
   expected=(p/folder/gen/filename).read_text();actual=(tmp/'generated'/filename).read_text()
   # Only relocation of the archive-identical original source prefix is permitted.
   frozen='/workspace/full-api/endian/nohash-length-prefix/original'
   actual=actual.replace(str((p/'original').resolve()),frozen)
   assert actual==expected,(folder,filename)
  print('regeneration exact (source-prefix relocation only):',folder)
