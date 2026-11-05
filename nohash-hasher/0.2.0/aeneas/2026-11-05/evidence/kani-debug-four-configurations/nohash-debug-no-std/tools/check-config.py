from pathlib import Path
import json,tarfile
r=Path(__file__).resolve().parents[1]
for mode in ['debug','release']:
 x=json.loads((r/f'evidence/metadata-{mode}.json').read_text())
 p=next(p for p in x['packages'] if p['name']=='nohash-hasher')
 node=next(n for n in x['resolve']['nodes'] if n['id']==p['id'])
 assert node['features']==[]
 root=next(p for p in x['packages'] if p['name']=='nohash-debug-no-std-probe')
 dep=next(d for d in root['dependencies'] if d['name']=='nohash-hasher')
 assert dep['uses_default_features'] is False and dep['features']==[]
with tarfile.open(r/'evidence/upstream.crate') as t:
 for f in ['src/lib.rs','Cargo.toml']:
  assert (r/'source'/f).read_bytes()==t.extractfile('nohash-hasher-0.2.0/'+f).read()
assert (r/'src/lib.rs').read_bytes()==(r.parent/'nohash-debug-probe/src/lib.rs').read_bytes()
print('Dependency features empty in both resolved graphs; archive/source/manifest and fixed harness identical')
