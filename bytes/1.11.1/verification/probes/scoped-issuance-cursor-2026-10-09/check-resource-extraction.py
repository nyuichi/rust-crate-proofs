#!/usr/bin/env python3
"""Compiler controls: pure cursor observations cannot be State or Perm."""
import json,os,pathlib,subprocess
r=pathlib.Path(__file__).resolve().parent;p=r/'src/driver.rs';original=p.read_bytes()
f=r/'generated/type-controls';f.mkdir(parents=True,exist_ok=True);rows=[]
for name,result in [('extract_state','crate::lifecycle::State<Marker>'),('extract_perm','Perm<ModelAtomic>')]:
    addition=f'''\n#[check(ghost)]\nfn cannot_{name}(cursor: crate::event::ScopeCursor<crate::lifecycle::State<Marker>>) -> {result} {{\n    *cursor.observation()\n}}\n'''.encode()
    source=original+addition
    (f/(name+'.rs')).write_bytes(source)
    try:
        p.write_bytes(source)
        with (f/(name+'.log')).open('w') as log:
            status=subprocess.run([str(r/'run-proof.sh')],env=dict(os.environ,
                BYTES_TRANSLATE_ONLY='1',BYTES_SCOPE_DIAGNOSTIC='1'),stdout=log,stderr=subprocess.STDOUT).returncode
        text=(f/(name+'.log')).read_text()
        assert status!=0 and 'error[E0308]' in text and 'mismatched types' in text, (name,status,text[-2000:])
        assert 'Goal Coma.' not in text, 'type rejection is not a failed VC'
        rows.append(dict(control=name,phase='Rust typechecking before VC/prover',exit_status=status,
                         expected='E0308: Observation tuple cannot be affine State/Perm',passed=True))
    finally:p.write_bytes(original)
(f/'results.json').write_text(json.dumps(rows,indent=2)+'\n')
print(json.dumps(rows))
