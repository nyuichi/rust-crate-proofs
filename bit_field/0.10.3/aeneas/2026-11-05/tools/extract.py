from pathlib import Path
import subprocess,json
p=Path(__file__).resolve().parents[1]
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--monomorphize','--dest','evidence','--start-from','{impl bit_field::BitField for _}::get_bit','--start-from','{impl bit_field::BitField for _}::set_bit']
(p/'evidence/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest','proofs/BitField','evidence/bit_field.llbc']
(p/'evidence/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
subprocess.run(['python3','tools/provide-size-models.py'],cwd=p,check=True)

(p/'evidence/array').mkdir(exist_ok=True)
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence/array','--start-from','{impl crate::BitArray for _}::bit_length','--start-from','{impl crate::BitArray for _}::get_bit','--start-from','{impl crate::BitArray for _}::set_bit']
(p/'evidence/array/charon-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/array/charon.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/BitArrayGenerated','evidence/array/bit_field.llbc']
(p/'evidence/array/aeneas-command.json').write_text(json.dumps(cmd,indent=2))
with (p/'evidence/array/aeneas.log').open('w') as out:subprocess.run(cmd,cwd=p,check=True,stdout=out,stderr=subprocess.STDOUT)

subprocess.run(['python3','tools/check-array.py'],cwd=p,check=True)
