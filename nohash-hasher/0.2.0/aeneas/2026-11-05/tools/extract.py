#!/usr/bin/env python3
"""Extract original method bodies using official Charon root selection."""
from pathlib import Path
import subprocess, json, os, sys
release = "--release" in sys.argv
root = Path(__file__).resolve().parents[1]
methods = ['write','write_u8','write_u16','write_u32','write_u64','write_usize','write_i8','write_i16','write_i32','write_i64','write_isize','finish']
roots = [f'{{impl core::hash::Hasher for nohash_hasher::NoHashHasher<_>}}::{m}' for m in methods]
roots += ['{impl core::default::Default for nohash_hasher::NoHashHasher<_>}::default', '{impl core::clone::Clone for nohash_hasher::NoHashHasher<_>}::clone']
output = root/'evidence/release' if release else root/'evidence'
output.mkdir(exist_ok=True)
cmd = ['charon','cargo','--preset','aeneas','--sysroot','default','--dest',str(output)]
for r in roots: cmd += ['--start-from',r]
if release: cmd += ['--', '--release']
(output/'extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (output/'charon.log').open('w') as f: subprocess.run(cmd,cwd=root,check=True,stdout=f,stderr=subprocess.STDOUT)
dest = root/'proofs/ReleaseGenerated' if release else root/'proofs/Generated'
cmd = ['aeneas','-backend','lean','-emit-json','-abort-on-error','-warnings-as-errors','-dest',str(dest),str(output/'nohash_hasher.llbc')]
if release: cmd += ['-namespace', 'nohash_hasher_release']
(output/'aeneas-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
with (output/'aeneas.log').open('w') as f: subprocess.run(cmd,cwd=root,check=True,stdout=f,stderr=subprocess.STDOUT)
