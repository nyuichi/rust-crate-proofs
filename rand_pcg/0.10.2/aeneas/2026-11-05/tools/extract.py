import subprocess,os
args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file','evidence/rand_pcg.llbc']
for module,t in [('pcg64','Lcg64Xsh32'),('pcg128','Lcg128Xsl64'),('pcg128cm','Lcg128CmDxsm64'),('pcg128','Mcg128Xsl64')]:
 for m in (['state','stream','from_state','new'] if t!='Mcg128Xsl64' else ['state','new']):args+=['--start-from','crate::'+module+'::'+t+'::'+m]
args+=['--start-from','{impl rand_core::TryRng for crate::pcg64::Lcg64Xsh32}::try_next_u32']
subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs','evidence/rand_pcg.llbc'],check=True)
