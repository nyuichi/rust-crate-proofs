from pathlib import Path
import subprocess,os,json,hashlib
r=Path(__file__).resolve().parent
rows=json.loads((r/'sources.json').read_text())
for name,ver,source,configs in rows:
 s=Path(source); proofs=s/'proofs.toml'
 for config in configs:
  out=r/name/config;out.mkdir(parents=True,exist_ok=True)
  env=dict(os.environ,CARGO_TARGET_DIR=str(out/'target'),CARGO_NET_OFFLINE='true')
  feature_args={'default':[],'all':['--all-features'],'no-default':['--no-default-features']}[config]
  cmd=['cargo','rustdoc','--locked','--lib','--target','x86_64-unknown-linux-gnu',*feature_args,'--','-Z','unstable-options','--output-format','json']
  with (out/'rustdoc.log').open('w') as f: subprocess.run(cmd,cwd=s,env=env,stdout=f,stderr=subprocess.STDOUT,check=True)
  doc=out/'target/x86_64-unknown-linux-gnu/doc'/f'{name.replace("-","_")}.json'
  (out/'rustdoc.json').write_bytes(doc.read_bytes())
  cmd2=['cargo','rustc','--locked','--lib','--target','x86_64-unknown-linux-gnu',*feature_args,'--','-Zunpretty=expanded']
  with (out/'expanded.rs').open('w') as f,(out/'expanded.log').open('w') as err:subprocess.run(cmd2,cwd=s,env=env,stdout=f,stderr=err,check=True)
  info={'crate':name,'version':ver,'source':source,'configuration':config,'target':'x86_64-unknown-linux-gnu','rustdoc_command':cmd,'expansion_command':cmd2,'source_sha256':{str(p.relative_to(s)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [s/'Cargo.toml',*sorted((s/'src').rglob('*.rs'))]},'proofs_toml':proofs.read_text()}
  (out/'provenance.json').write_text(json.dumps(info,indent=2)+'\n')
 print(name,'collected',flush=True)
