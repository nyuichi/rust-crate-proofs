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

(p/'evidence/float').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/float')]
(p/'evidence/float/charon-command.json').write_text(json.dumps(cmd,indent=2))
float_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/float/rustflags.json').write_text(json.dumps({'RUSTFLAGS':float_flags}))
with (p/'evidence/float/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/float-selector',env=dict(os.environ,RUSTFLAGS=float_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderFloatRoots'),str(p/'evidence/float/byteorder_float_roots.llbc')]
(p/'evidence/float/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/float/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/provide-float-models.py'],cwd=p,check=True)
subprocess.run(['python3','tools/check-float-selector.py'],cwd=p,check=True)

(p/'evidence/read-variable').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence/read-variable']
for endian in ['BigEndian','LittleEndian']:
 for method in ['read_uint','read_uint128']:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::{endian}}}::{method}']
(p/'evidence/read-variable/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/read-variable/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/ReadVariableGenerated','evidence/read-variable/byteorder.llbc']
(p/'evidence/read-variable/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/read-variable/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-read-variable.py'],cwd=p,check=True)

(p/'evidence/u24').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/u24')]
(p/'evidence/u24/charon-command.json').write_text(json.dumps(cmd,indent=2))
u24_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/u24/rustflags.json').write_text(json.dumps({'RUSTFLAGS':u24_flags}))
with (p/'evidence/u24/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/u24-selector',env=dict(os.environ,RUSTFLAGS=u24_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderU24Roots'),str(p/'evidence/u24/byteorder_u24_roots.llbc')]
(p/'evidence/u24/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/u24/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-u24.py'],cwd=p,check=True)

(p/'evidence/native-identity').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence/native-identity']
for sign in ['u']:
 for bits in [16,32,64,128]:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::LittleEndian}}::from_slice_{sign}{bits}']
(p/'evidence/native-identity/charon-command.json').write_text(json.dumps(cmd,indent=2))
native_env=dict(os.environ,CARGO_BUILD_TARGET='x86_64-unknown-linux-gnu')
(p/'evidence/native-identity/target-env.json').write_text(json.dumps({'CARGO_BUILD_TARGET':native_env['CARGO_BUILD_TARGET'],'target_endian':'little'}))
with (p/'evidence/native-identity/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,env=native_env,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest','proofs/Byteorder','evidence/native-identity/byteorder.llbc']
(p/'evidence/native-identity/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/native-identity/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/provide-native-no-model.py'],cwd=p,check=True)

(p/'evidence/read-pure').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/read-pure')]
(p/'evidence/read-pure/charon-command.json').write_text(json.dumps(cmd,indent=2))
read_pure_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/read-pure/rustflags.json').write_text(json.dumps({'RUSTFLAGS':read_pure_flags}))
with (p/'evidence/read-pure/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/read-pure-selector',env=dict(os.environ,RUSTFLAGS=read_pure_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderReadPureRoots'),str(p/'evidence/read-pure/byteorder_read_pure_roots.llbc')]
(p/'evidence/read-pure/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/read-pure/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-read-pure.py'],cwd=p,check=True)
