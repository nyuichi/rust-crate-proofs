#!/usr/bin/env python3
"""Retain a deliberately non-complete client; restore canonical source always."""
import os,pathlib,subprocess,sys
r=pathlib.Path(__file__).resolve().parent
p=r/'src/driver.rs';original=p.read_bytes()
needle=b'    let (second_last, second_recovery) = registry.retire(second, cursor.borrow_mut());'
assert original.count(needle)==1
variant=original.replace(needle,b'    let (_, unretired) = registry.register(second.borrow(), cursor.borrow_mut());\n'+needle)
assert len(sys.argv)==2,'pass absolute run log path'
try:
    p.write_bytes(variant)
    env=dict(os.environ,BYTES_SCOPE_DIAGNOSTIC='1')
    with open(sys.argv[1],'w') as log:
        result=subprocess.run([str(r/'run-proof.sh')],env=env,stdout=log,stderr=subprocess.STDOUT)
    assert result.returncode==1,'expected semantic failure after translation; inspect log on any other status'
    subprocess.run([sys.executable,str(r/'evidence.py'),'capture','cursor-negative-extra-owner-2026-10-09',sys.argv[1],'--status','diagnostic'],check=True)
finally:
    p.write_bytes(original)
