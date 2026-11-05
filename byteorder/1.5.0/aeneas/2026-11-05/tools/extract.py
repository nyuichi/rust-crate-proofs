from pathlib import Path
import subprocess,json
p=Path(__file__).resolve().parents[1]
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for endian in ['BigEndian','LittleEndian']:
 for bits in [16,32,64,128]:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::{endian}}}::write_u{bits}']
(p/'evidence/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/byteorder.llbc']
(p/'evidence/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
