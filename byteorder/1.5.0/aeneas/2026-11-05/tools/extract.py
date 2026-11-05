from pathlib import Path
import subprocess,json,os
p=Path(__file__).resolve().parents[1]
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for endian in ['BigEndian','LittleEndian']:
 for bits in [16,32,64,128]:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::{endian}}}::write_u{bits}']
for endian in ['BigEndian','LittleEndian']:
 cmd+=['--start-from',f'{{impl core::default::Default for crate::{endian}}}::default']
(p/'evidence/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/byteorder.llbc']
(p/'evidence/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)

(p/'evidence/signed').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/signed')]
(p/'evidence/signed/charon-command.json').write_text(json.dumps(cmd,indent=2))
signed_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/signed/rustflags.json').write_text(json.dumps({'RUSTFLAGS':signed_flags}))
with (p/'evidence/signed/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/api-selector',env=dict(os.environ,RUSTFLAGS=signed_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderApiRoots'),str(p/'evidence/signed/byteorder_api_roots.llbc')]
(p/'evidence/signed/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/signed/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-selector.py'],cwd=p,check=True)
