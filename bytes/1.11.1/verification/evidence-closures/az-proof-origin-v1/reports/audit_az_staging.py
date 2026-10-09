from pathlib import Path
import json,hashlib,collections
sha=lambda b:hashlib.sha256(b).hexdigest();repo=Path('/workspace/bytes-work');stage=Path('/workspace/work/az-origin-source-v1');v=repo/'bytes/1.11.1/verification';rp=Path('/workspace/work/az-origin-recipe-v1.json');pp=stage/'metadata/AZ_INPUT_BINDINGS.json';ip=Path('/workspace/work/AZ_REQUIRED_INPUTS_v1.json')
assert sha(rp.read_bytes())=='69e9061c4d689536980c0c7023a644a449bdb5e1c35c78ffa1236824f1899e04';assert sha(pp.read_bytes())=='f89d6bf961b7fc3d98e16e18e4352f7f55453788da884da940644ad45b598193';assert sha(ip.read_bytes())=='aa02d915e100a72baaf4583f701d738ff1e2c8cc88621b0471b2db9d5325493d'
recipe=json.loads(rp.read_text());profile=json.loads(pp.read_text());req=json.loads(ip.read_text());ayp=v/'evidence-closures/ay-admitted-reuse-v1/manifest.json';assert sha(ayp.read_bytes())==profile['ancestor_raw_manifest_sha256']=='71ed49af75822b9281ec87b7d0009c13463cf4989ebbcf71f36705cc586139d8';ay=json.loads(ayp.read_text());af={r['path']:r for r in ay['files']};bindings={r['destination']:r for r in profile['bindings']};rows={r['destination']:r for r in req['entries']};assert len(bindings)==len(profile['bindings'])==1353 and set(bindings)==set(rows)
sources={s['id']:s for s in recipe['sources']};mounts={m['path_prefix']:m for m in recipe['mounts']}
for dest,b in bindings.items():
 r=rows[dest];assert (b['sha256'],b['size'])==(r['sha256'],r['size'])
 if b['resolution']=='local':
  rel=Path(b['source_path']);assert not rel.is_absolute() and '..'not in rel.parts;q=stage/rel;raw=q.read_bytes();assert not q.is_symlink() and (sha(raw),len(raw))==(b['sha256'],b['size']);assert sources[mounts[dest]['source_id']]['path']==b['source_path']
 else:
  assert b['resolution']=='ancestor' and b['source_path'].startswith('ancestors/ay/');a=af[b['source_path'][len('ancestors/ay/'):]];assert (a['sha256'],a['size'])==(b['sha256'],b['size'])
prov={e['id']:e for e in ay['provenance']};oldmounts={m['id']:m for m in ay['mounts']};imports=recipe['imports'];assert len(imports)==1 and imports[0]['manifest_sha256']==profile['ancestor_raw_manifest_sha256'];sel=imports[0]['mounts'];assert len(sel)==len(oldmounts) and len({m['source_mount_id'] for m in sel})==len(sel)
for m in sel:
 old=oldmounts[m['source_mount_id']];assert m['path_prefix']=='ancestors/ay/'+old['path_prefix'] and m['role']==prov[old['provenance_id']]['role']
r=json.loads(Path('/workspace/work/AZ_ORIGIN_INVENTORY_AUDIT.json').read_text());r.update(binding_profile_sha256=sha(pp.read_bytes()),recipe_sha256=sha(rp.read_bytes()),staged_bindings_independently_verified=1353,resolutions=dict(collections.Counter(b['resolution'] for b in bindings.values())),imported_AY_mounts=len(sel),actual_manifest_audit_pending=True);Path('/workspace/work/AZ_ORIGIN_INVENTORY_AUDIT.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
