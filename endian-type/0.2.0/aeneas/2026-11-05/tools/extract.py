#!/usr/bin/env python3
from pathlib import Path
import subprocess,json,os
r=Path(__file__).resolve().parents[1]
roots=[f'{{impl core::ops::bit::{trait} for endian_type::{typ}<_>}}::{op}' for typ in ['BigEndian','LittleEndian'] for trait,op in [('BitAnd','bitand'),('BitOr','bitor'),('BitXor','bitxor')]]
cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest',str(r/'evidence')]
for root in roots: cmd+=['--start-from',root]
(r/'evidence/extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (r/'evidence/charon.log').open('w') as f: subprocess.run(cmd,cwd=r,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=f,stderr=subprocess.STDOUT,check=True)
cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-warnings-as-errors','-dest',str(r/'proofs/Generated'),str(r/'evidence/endian_type.llbc')]
with (r/'evidence/aeneas.log').open('w') as f: subprocess.run(cmd,cwd=r,stdout=f,stderr=subprocess.STDOUT,check=True)
