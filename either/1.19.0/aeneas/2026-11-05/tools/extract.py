from pathlib import Path
import subprocess,json
r=Path(__file__).resolve().parents[1]
roots = ['either::Either::is_left', 'either::Either::is_right', 'either::Either::left', 'either::Either::right', 'either::Either::flip', 'either::Either::left_or', 'either::Either::right_or', 'either::Either::left_and', 'either::Either::right_and', 'either::Either::factor_none', 'either::Either::factor_first', 'either::Either::factor_second', 'either::Either::as_ref', 'either::Either::as_mut', 'either::Either::is_left_and', 'either::Either::is_right_and', 'either::Either::is_left_or', 'either::Either::is_right_or', 'either::Either::map_left', 'either::Either::map_right', 'either::Either::map_left_or', 'either::Either::map_right_or', 'either::Either::left_or_else', 'either::Either::right_or_else', 'either::Either::left_and_then', 'either::Either::right_and_then']
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest','evidence']
for root in roots:cmd+=['--start-from',root]
(r/'evidence/extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (r/'evidence/charon.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs/Generated','evidence/either.llbc']
with (r/'evidence/aeneas.log').open('w') as f:subprocess.run(cmd,cwd=r,check=True,stdout=f,stderr=subprocess.STDOUT)
