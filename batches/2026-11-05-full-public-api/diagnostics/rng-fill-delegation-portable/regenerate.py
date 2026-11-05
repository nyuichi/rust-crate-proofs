#!/usr/bin/env python3
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,tempfile
p=Path(__file__).resolve().parent;pin=json.loads((p/'pins.json').read_text());sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest();ap=argparse.ArgumentParser();ap.add_argument('--generate-only',action='store_true');ap.add_argument('--target-dir');a=ap.parse_args();env=dict(os.environ,CARGO_NET_OFFLINE='true',RUSTUP_TOOLCHAIN='nightly-2026-09-17');commands=[]
if a.target_dir:env['CARGO_TARGET_DIR']=a.target_dir
def call(argv,cwd=p):
 r=subprocess.run(argv,cwd=cwd,env=env,capture_output=True,text=True);commands.append({'argv':list(map(str,argv)),'cwd':str(cwd),'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr});assert r.returncode==0,commands[-1];return r.stdout
for n,h in pin['original_files'].items():assert sha(p/'original'/n)==h
assert sha(p/'upstream.crate')==pin['upstream_archive_sha256']
for n,h in pin['driver_files'].items():assert sha(p/n)==h
for name,v in pin['official_tools'].items():
 exe=Path(shutil.which(name)).resolve();assert sha(exe)==v['sha256'];assert call([str(exe),'version'if name=='charon'else'-version']).strip()==v['version']
version=call(['rustc','-vV']);assert 'commit-hash: '+pin['Rust_commit']in version and'host: '+pin['host']in version
metadata=json.loads(call(['cargo','metadata','--offline','--locked','--format-version','1']));deps=[x for x in metadata['packages']if x['name']=='rand_core'and x['version']=='0.10.0'];assert len(deps)==1;registry=Path(deps[0]['manifest_path']).parent
for n,h in pin['original_files'].items():
 if n.endswith('.rs')or n in ['Cargo.toml','Cargo.toml.orig']:assert sha(registry/n)==h,n
with tempfile.TemporaryDirectory(prefix='rng-fill-original-replay-')as temp:
 tmp=Path(temp);llbc=tmp/'fill.llbc';call(['charon','cargo','--preset','aeneas','--sysroot','default','--include','rand_core','--start-from','crate::original_fill_root','--dest-file',str(llbc),'--','--locked']);call(['aeneas','-backend','lean','-split-files','-emit-json','-namespace','RngFillOriginal','-abort-on-error','-dest',str(tmp/'generated'),str(llbc)])
 for n,h in pin['generated_files'].items():assert sha(tmp/'generated'/n)==h,n
 print('PASS: original registry runtime source/Cargo/archive pins; official two-function generation byte-identical')
if not a.generate_only:
 call(['lake','build'],p/'proofs');text=call(['lake','env','lean','Proofs.lean'],p/'proofs');assert'sorryAx'not in text and text.count('core.fmt.Formatter')==4;print('PASS: four kernel theorems with explicitly retained Formatter type axiom (not standard3-only)')
(p/'evidence/replay-commands.json').write_text(json.dumps(commands,indent=2))
