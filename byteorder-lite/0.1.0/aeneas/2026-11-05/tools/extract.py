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
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/byteorder_lite.llbc']
(p/'evidence/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)

(p/'evidence/signed').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/signed')]
(p/'evidence/signed/charon-command.json').write_text(json.dumps(cmd,indent=2))
signed_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/signed/rustflags.json').write_text(json.dumps({'RUSTFLAGS':signed_flags}))
with (p/'evidence/signed/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/api-selector',env=dict(os.environ,RUSTFLAGS=signed_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderLiteApiRoots'),str(p/'evidence/signed/byteorder_lite_api_roots.llbc')]
(p/'evidence/signed/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/signed/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-selector.py'],cwd=p,check=True)

(p/'evidence/variable').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence/variable']
for endian in ['BigEndian','LittleEndian']:
 for method in ['write_uint','write_uint128']:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::{endian}}}::{method}']
(p/'evidence/variable/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/variable/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/VariableGenerated','evidence/variable/byteorder_lite.llbc']
(p/'evidence/variable/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/variable/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)

subprocess.run(['python3','tools/check-variable.py'],cwd=p,check=True)

(p/'evidence/float').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/float')]
(p/'evidence/float/charon-command.json').write_text(json.dumps(cmd,indent=2))
float_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/float/rustflags.json').write_text(json.dumps({'RUSTFLAGS':float_flags}))
with (p/'evidence/float/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/float-selector',env=dict(os.environ,RUSTFLAGS=float_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderLiteFloatRoots'),str(p/'evidence/float/byteorder_lite_float_roots.llbc')]
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
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/ReadVariableGenerated','evidence/read-variable/byteorder_lite.llbc']
(p/'evidence/read-variable/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/read-variable/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-read-variable.py'],cwd=p,check=True)

(p/'evidence/u24').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/u24')]
(p/'evidence/u24/charon-command.json').write_text(json.dumps(cmd,indent=2))
u24_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/u24/rustflags.json').write_text(json.dumps({'RUSTFLAGS':u24_flags}))
with (p/'evidence/u24/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/u24-selector',env=dict(os.environ,RUSTFLAGS=u24_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderLiteU24Roots'),str(p/'evidence/u24/byteorder_lite_u24_roots.llbc')]
(p/'evidence/u24/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/u24/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-u24.py'],cwd=p,check=True)

(p/'evidence/signed-variable').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/signed-variable')]
(p/'evidence/signed-variable/charon-command.json').write_text(json.dumps(cmd,indent=2))
signed_variable_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/signed-variable/rustflags.json').write_text(json.dumps({'RUSTFLAGS':signed_variable_flags}))
with (p/'evidence/signed-variable/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/signed-variable-selector',env=dict(os.environ,RUSTFLAGS=signed_variable_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/SignedVariableRoots'),str(p/'evidence/signed-variable/signed_variable_roots.llbc')]
(p/'evidence/signed-variable/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/signed-variable/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-signed-variable.py'],cwd=p,check=True)

(p/'evidence/signed-read').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/signed-read')]
(p/'evidence/signed-read/charon-command.json').write_text(json.dumps(cmd,indent=2))
signed_read_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/signed-read/rustflags.json').write_text(json.dumps({'RUSTFLAGS':signed_read_flags}))
with (p/'evidence/signed-read/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/signed-read-selector',env=dict(os.environ,RUSTFLAGS=signed_read_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/SignedReadRoots'),str(p/'evidence/signed-read/signed_read_roots.llbc')]
(p/'evidence/signed-read/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/signed-read/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-signed-read.py'],cwd=p,check=True)

(p/'evidence/native-identity').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence/native-identity']
for sign in ['u','i']:
 for bits in [16,32,64,128]:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::LittleEndian}}::from_slice_{sign}{bits}']
for bits in [32,64]:cmd+=['--start-from',f'{{impl crate::ByteOrder for crate::LittleEndian}}::from_slice_f{bits}']
(p/'evidence/native-identity/charon-command.json').write_text(json.dumps(cmd,indent=2))
native_env=dict(os.environ,CARGO_BUILD_TARGET='x86_64-unknown-linux-gnu')
(p/'evidence/native-identity/target-env.json').write_text(json.dumps({'CARGO_BUILD_TARGET':native_env['CARGO_BUILD_TARGET'],'target_endian':'little'}))
with (p/'evidence/native-identity/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,env=native_env,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest','proofs/ByteorderLite','evidence/native-identity/byteorder_lite.llbc']
(p/'evidence/native-identity/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/native-identity/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/provide-native-no-model.py'],cwd=p,check=True)

(p/'evidence/u48').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/u48')]
(p/'evidence/u48/charon-command.json').write_text(json.dumps(cmd,indent=2))
u48_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/u48/rustflags.json').write_text(json.dumps({'RUSTFLAGS':u48_flags}))
with (p/'evidence/u48/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/u48-selector',env=dict(os.environ,RUSTFLAGS=u48_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/ByteorderLiteU48Roots'),str(p/'evidence/u48/byteorder_lite_u48_roots.llbc')]
(p/'evidence/u48/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/u48/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-u48.py'],cwd=p,check=True)

(p/'evidence/signed128').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--include','byteorder_lite','--duplicate-defaulted-methods','--remove-unused-self-clauses','--dest',str(p/'evidence/signed128')]
(p/'evidence/signed128/charon-command.json').write_text(json.dumps(cmd,indent=2))
signed128_flags='--remap-path-prefix='+str(p)+'/'+'='
(p/'evidence/signed128/rustflags.json').write_text(json.dumps({'RUSTFLAGS':signed128_flags}))
with (p/'evidence/signed128/charon.log').open('w') as out:subprocess.run(cmd,cwd=p/'tools/i128-selector',env=dict(os.environ,RUSTFLAGS=signed128_flags),check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(p/'proofs/LiteSigned128Roots'),str(p/'evidence/signed128/lite_signed128_roots.llbc')]
(p/'evidence/signed128/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/signed128/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/check-signed128.py'],cwd=p,check=True)
