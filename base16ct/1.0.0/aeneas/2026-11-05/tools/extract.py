from pathlib import Path
import subprocess,json
r=Path(__file__).resolve().parents[1]
roots=['base16ct::decoded_len', 'base16ct::encoded_len']
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for root in roots:cmd+=['--start-from',root]
(r/'evidence/extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (r/'evidence/charon.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/base16ct.llbc']
with (r/'evidence/aeneas.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
