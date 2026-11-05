from pathlib import Path
import subprocess,os
base=Path('/workspace/full-api/prng')
for src in sorted((base/'sources').iterdir()):
 name=src.name.rsplit('-',1)[0]
 for mode in ['default','all-features']:
  out=base/'expanded'/name;out.mkdir(parents=True,exist_ok=True)
  args=['cargo','rustc','--manifest-path',str(src/'Cargo.toml'),'--lib','--locked','--target-dir',str(base/'expanded-target')]
  if mode=='all-features':args+=['--all-features']
  args+=['--','-Zunpretty=expanded']
  with (out/f'{mode}.rs').open('w') as stdout,(out/f'{mode}.log').open('w') as stderr:
   p=subprocess.run(args,stdout=stdout,stderr=stderr,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
  print(name,mode,p.returncode,flush=True)
