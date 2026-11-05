from pathlib import Path
import subprocess,json,os,hashlib
base=Path(__file__).parent
for name in ['byteorder','byteorder-lite','bit_field']:
 p=base/name;src=p/'source'
 for cfg,args in [('default',[]),('no-default',['--no-default-features']),('all',['--all-features'])]:
  out=p/cfg;out.mkdir(exist_ok=True)
  env=dict(os.environ,CARGO_NET_OFFLINE='true',CARGO_TARGET_DIR=str(p/'target'))
  commands=[['cargo','rustdoc','--lib',*args,'--','-Z','unstable-options','--output-format','json'],['cargo','rustc','--lib',*args,'--','-Zunpretty=expanded']]
  for i,cmd in enumerate(commands):
   result=subprocess.run(cmd,cwd=src,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
   (out/('rustdoc.log' if i==0 else 'expanded.stderr')).write_bytes(result.stderr)
   if i==1:(out/'expanded.rs').write_bytes(result.stdout)
   assert result.returncode==0,(name,cfg,cmd,result.stderr.decode())
   if i==0:
    import shutil
    shutil.copy(p/'target/doc'/f'{name.replace("-","_")}.json',out/'rustdoc.json')
  (out/'commands.json').write_text(json.dumps(commands,indent=2))
 print(name,'complete',flush=True)
