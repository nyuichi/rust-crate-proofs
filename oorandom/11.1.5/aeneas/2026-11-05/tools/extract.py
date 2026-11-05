import os,subprocess
args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file','evidence/oorandom.llbc']
for t,bits in [('Rand32','32'),('Rand64','64')]:
 for m in ['new','new_inc','state','from_state','rand_u'+bits,'rand_i'+bits]:args += ['--start-from','crate::'+t+'::'+m]
subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs','evidence/oorandom.llbc'],check=True)
