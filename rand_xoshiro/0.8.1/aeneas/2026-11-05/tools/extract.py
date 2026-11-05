from pathlib import Path
import os,subprocess,re,json
args=['charon','cargo','--preset=aeneas','--sysroot','default','--dest-file','evidence/rand_xoshiro.llbc'];entries=[]
for p in sorted(Path('src').glob('*.rs')):
 if p.stem in ['lib','common']:continue
 text=p.read_text();name=re.search(r'pub struct (\w+)',text).group(1)
 primary='32' if p.stem.startswith('xoroshiro64') or p.stem.startswith('xoshiro128') else '64'
 for bits in ([primary,'32'] if p.stem=='splitmix64' else [primary]):
  args+=['--start-from','{impl rand_core::TryRng for crate::'+p.stem+'::'+name+'}::try_next_u'+bits]
  entries.append({'module':p.stem,'type':name,'bits':bits})
Path('evidence/entries.json').write_text(json.dumps(entries,indent=2))
subprocess.run(args,check=True,env=dict(os.environ,CARGO_NET_OFFLINE='true'))
subprocess.run(['aeneas','-backend','lean','-emit-json','-abort-on-error','-dest','proofs','evidence/rand_xoshiro.llbc'],check=True)
