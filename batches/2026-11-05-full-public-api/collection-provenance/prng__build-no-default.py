from pathlib import Path
import shutil,subprocess,os,json
base=Path('/workspace/full-api/prng')
versions={'oorandom':'11.1.5','rand_pcg':'0.10.2','rand_xoshiro':'0.8.1','rand_xorshift':'0.5.0'}
for name,ver in versions.items():
 src=Path('/workspace/publish-batch-ten')/name/ver/'aeneas/2026-11-05';dst=base/'sources'/f'{name}-{ver}'
 shutil.copytree(src,dst,ignore=shutil.ignore_patterns('proofs','evidence','target','tools','.gitignore','verify-*.bash'),dirs_exist_ok=True)
 for mode in ['no-default-features']:
  out=base/'rustdoc'/name/mode;out.mkdir(parents=True,exist_ok=True)
  args=['cargo','rustdoc','--manifest-path',str(dst/'Cargo.toml'),'--lib','--locked','--target','x86_64-unknown-linux-gnu','--target-dir',str(out)]
  if mode=='no-default-features':args+=['--no-default-features']
  args+=['--','-Z','unstable-options','--output-format','json','--document-private-items']
  with (out/'build.log').open('w') as log:
   p=subprocess.run(args,env=dict(os.environ,CARGO_NET_OFFLINE='true'),stdout=log,stderr=subprocess.STDOUT)
  print(name,mode,p.returncode,flush=True)
