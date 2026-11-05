#!/usr/bin/env python3
from pathlib import Path
import subprocess,json,os
r=Path(__file__).resolve().parents[1]
methods=['write','write_u8','write_u16','write_u32','write_u64','write_usize','write_i8','write_i16','write_i32','write_i64','write_isize','finish']
roots=[f'{{impl core::hash::Hasher for nohash_hasher::NoHashHasher<_>}}::{m}' for m in methods]
roots+=['{impl core::default::Default for nohash_hasher::NoHashHasher<_>}::default','{impl core::clone::Clone for nohash_hasher::NoHashHasher<_>}::clone']
for release in [False,True]:
 out=r/'evidence/no-std'/('release' if release else 'debug');out.mkdir(parents=True,exist_ok=True)
 cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest-file',str(out/'nohash_hasher.llbc')]
 for root in roots:cmd+=['--start-from',root]
 cmd+=['--','--no-default-features']+(['--release'] if release else [])
 (out/'extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
 with (out/'charon.log').open('w') as f:subprocess.run(cmd,cwd=r,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=f,stderr=subprocess.STDOUT,check=True)
 folder='NoStdReleaseGenerated' if release else 'NoStdGenerated'
 namespace='nohash_hasher_no_std_release' if release else 'nohash_hasher_no_std'
 cmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-warnings-as-errors','-namespace',namespace,'-dest',str(r/'proofs'/folder),str(out/'nohash_hasher.llbc')]
 with (out/'aeneas.log').open('w') as f:subprocess.run(cmd,cwd=r,stdout=f,stderr=subprocess.STDOUT,check=True)
 actual=(r/'proofs'/folder/'NohashHasher.lean').read_text().replace(namespace,'nohash_hasher_release' if release else 'nohash_hasher')
 expected=(r/'proofs'/('ReleaseGenerated' if release else 'Generated')/'NohashHasher.lean').read_text()
 assert actual==expected,'Unexpected std/no-std body difference'
 print(('release' if release else 'debug')+': namespace-normalized original std/no-std generated bodies are byte-identical')
