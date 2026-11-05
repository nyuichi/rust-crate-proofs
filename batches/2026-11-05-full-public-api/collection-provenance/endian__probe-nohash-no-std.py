from pathlib import Path
import os,subprocess,hashlib,json
src=Path('/workspace/rust-crate-proofs/nohash-hasher/0.2.0/aeneas/2026-11-05');r=Path('/workspace/full-api/endian/probes/nohash-no-std');r.mkdir(parents=True,exist_ok=True)
roots=[f'{{impl core::hash::Hasher for nohash_hasher::NoHashHasher<_>}}::{m}' for m in ['write','write_u8','write_u16','write_u32','write_u64','write_usize','write_i8','write_i16','write_i32','write_i64','write_isize','finish']]
roots += ['{impl core::default::Default for nohash_hasher::NoHashHasher<_>}::default','{impl core::clone::Clone for nohash_hasher::NoHashHasher<_>}::clone']
for release in [False,True]:
 out=r/('release' if release else 'debug');out.mkdir(exist_ok=True)
 cmd=['charon','cargo','--preset','aeneas','--sysroot','default','--dest-file',str(out/'nohash_hasher.llbc')]
 for root in roots:cmd+=['--start-from',root]
 cmd+=['--','--no-default-features']+(['--release'] if release else [])
 env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=str(r/'target'))
 with (out/'charon.log').open('w') as f:subprocess.run(cmd,cwd=src,env=env,stdout=f,stderr=subprocess.STDOUT,check=True)
 acmd=['aeneas','-backend','lean','-emit-json','-abort-on-error','-warnings-as-errors','-dest',str(out/'Generated'),str(out/'nohash_hasher.llbc')]
 if release:acmd+=['-namespace','nohash_hasher_release']
 with (out/'aeneas.log').open('w') as f:subprocess.run(acmd,cwd=src,env=env,stdout=f,stderr=subprocess.STDOUT,check=True)
 expected=src/'proofs'/('ReleaseGenerated' if release else 'Generated')/'NohashHasher.lean';actual=out/'Generated/NohashHasher.lean'
 same=actual.read_bytes()==expected.read_bytes();(out/'comparison.json').write_text(json.dumps({'commands':[cmd,acmd],'identical_generated_lean':same,'expected':str(expected),'actual':str(actual),'sha256_expected':hashlib.sha256(expected.read_bytes()).hexdigest(),'sha256_actual':hashlib.sha256(actual.read_bytes()).hexdigest()},indent=2)+'\n')
 print('release' if release else 'debug','byte-identical=',same,flush=True)
