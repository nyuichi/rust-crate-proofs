from pathlib import Path
import json
r=Path(__file__).resolve().parents[1];x=json.loads((r/'evidence/dependency-metadata.json').read_text())
core=[p for p in x['packages'] if p['name']=='rand_core'];assert len(core)==1 and core[0]['version']=='0.10.0'
pcg=next(p for p in x['packages'] if p['name']=='rand_pcg');node=next(n for n in x['resolve']['nodes'] if n['id']==pcg['id']);assert node['features']==[]
assert any(d['pkg']==core[0]['id'] for d in node['deps'])
print('Exact rand_core0.10.0 is shared by PCG and harness; rand_pcg features=[]')
