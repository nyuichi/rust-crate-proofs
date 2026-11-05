from pathlib import Path
import subprocess,os,json,tarfile,hashlib
r=Path(__file__).resolve().parents[1];src=r;archive=r/'evidence/upstream.crate'
with tarfile.open(archive) as t:
 for m in t.getmembers():
  if m.isfile():assert (src/Path(*Path(m.name).parts[1:])).read_bytes()==t.extractfile(m).read(),m.name
out=r/'evidence/integer-models';out.mkdir(exist_ok=True)
(out/'source-provenance.json').write_text(json.dumps({'archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'files':{str(p.relative_to(src)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [src/'Cargo.toml',src/'src/lib.rs']},'all_archive_files_byte_identical':True},indent=2)+'\n')
cmd=['charon','cargo','--preset','aeneas','--no-dedup-serialized-ast','--sysroot','default','--include','endian_type','--dest-file',str(r/'evidence/integer-models/endian_verification_driver.llbc'),'--','--locked','--target','x86_64-unknown-linux-gnu']
(out/'extraction-command.json').write_text(json.dumps(cmd,indent=2)+'\n')
env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=str(r/'target/integer-driver'))
remap=[f'--remap-path-prefix={src}=endian-type-0.2.0',f'--remap-path-prefix={r/"tools/integer-driver"}=verification-driver']
if 'CARGO_ENCODED_RUSTFLAGS' in env:env['CARGO_ENCODED_RUSTFLAGS']+='\x1f'+'\x1f'.join(remap)
else:env['RUSTFLAGS']=env.get('RUSTFLAGS','')+' '+' '.join(remap)
(out/'compiler-path-remapping.json').write_text(json.dumps({'added_flags':remap,'purpose':'Official rustc metadata-path remapping for reproducible dependency source spans; original source and MIR operations unchanged.'},indent=2)+'\n')
with (out/'charon.log').open('w') as f:subprocess.run(cmd,cwd=r/'tools/integer-driver',env=env,stdout=f,stderr=subprocess.STDOUT,check=True)
cmd=['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(r/'proofs/EndianVerificationDriver'),str(r/'evidence/integer-models/endian_verification_driver.llbc')]
with (out/'aeneas.log').open('w') as f:subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,check=True)
