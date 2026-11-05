from pathlib import Path
import subprocess,json
r=Path(__file__).resolve().parents[1]
roots=['ascii::ascii_char::AsciiChar::is_ascii_blank', 'ascii::ascii_char::AsciiChar::is_ascii_digit', 'ascii::ascii_char::AsciiChar::is_ascii_whitespace', 'ascii::ascii_char::AsciiChar::is_ascii_control']
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for root in roots:cmd+=['--start-from',root]
(r/'evidence/extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (r/'evidence/charon.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/ascii.llbc']
with (r/'evidence/aeneas.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
