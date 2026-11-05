from pathlib import Path
import subprocess,os
p=Path(__file__).resolve().parent
for name,crate,release in [('nostd-debug','nohash_string_nostd_debug',False),('nostd-release','nohash_string_nostd_release',True),('std-release','nohash_string_std_release',True)]:
 q=p/name;env=os.environ.copy();env.update(CARGO_NET_OFFLINE='true',RUSTFLAGS='-Zmir-enable-passes=+Inline,+RemoveStorageMarkers,+ReferencePropagation,+DeadStoreElimination-final,+SimplifyLocals-final')
 cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--mir','optimized','--dest',str(q),'--start-from','crate::write_text','--include','nohash_hasher::NoHashHasher']
 if release:cmd+=['--','--release']
 with (q/'charon.log').open('w') as log:subprocess.run(cmd,cwd=q,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 with (q/'aeneas.log').open('w') as log:subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest',str(q/'generated'),str(q/(crate+'.llbc'))],env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
 print(name,'original compiler-inlined caller generated')
