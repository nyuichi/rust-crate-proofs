#!/usr/bin/env python3
"""Stage an explicit AZ current-input profile for the shared capture composer.

Inventory completeness is established separately by the correspondence checker
and independent audit. This producer never scans evidence or decides admission.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

AY_PIN = '71ed49af75822b9281ec87b7d0009c13463cf4989ebbcf71f36705cc586139d8'
COMPOSER_PIN = 'c400a8bd249f9133f61474ccb45baa85e2d9f6e5e5ef8485e72a616e5a3cb023'

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2)+'\n').encode()

def safe(value):
    p = Path(value)
    assert isinstance(value,str) and value and not p.is_absolute() and '..' not in p.parts
    assert p.as_posix()==value and '\\' not in value
    return value

def regular(path):
    assert path.is_file() and not any(p.is_symlink() for p in [path,*path.parents]), f'unsafe input {path}'
    return path.read_bytes()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inventory',type=Path,required=True)
    parser.add_argument('--ancestor-manifest',type=Path,required=True)
    parser.add_argument('--ancestor-cas',type=Path,required=True)
    parser.add_argument('--repository-root',type=Path,required=True)
    parser.add_argument('--source-root',type=Path,required=True)
    parser.add_argument('--recipe',type=Path,required=True)
    args=parser.parse_args()
    repo=args.repository_root.resolve(strict=True); stage=args.source_root.absolute()
    assert repo==Path('/workspace/bytes-work'), 'published locators use the actual repository root'
    assert stage.is_relative_to(Path('/workspace/work')) and not stage.exists(), 'fresh scratch source root required'
    assert not args.recipe.absolute().is_relative_to(stage), 'recipe output must be disjoint from its source root'
    manifest_path=args.ancestor_manifest.resolve(strict=True); cas=args.ancestor_cas.resolve(strict=True)
    manifest_raw=regular(manifest_path); assert sha(manifest_raw)==AY_PIN, 'AY raw manifest changed'
    ancestor=json.loads(manifest_raw); assert ancestor['schema']=='evidence-closure-v1'
    composer=repo/'bytes/1.11.1/verification/tools/evidence_capture.py'
    assert sha(regular(composer))==COMPOSER_PIN, 'shared composer changed'
    inventory_raw=regular(args.inventory); inventory=json.loads(inventory_raw)
    assert inventory['schema']=='az-required-input-inventory-v1'
    assert inventory['entries'], 'explicit inventory is empty'
    by_identity={}
    for row in sorted(ancestor['files'],key=lambda r:r['path']):
        by_identity.setdefault((row['sha256'],row['size']),row['path'])
    rows=[]; sources=[]; mounts=[]; names=set(); local=[]
    # Validate every explicit slot before creating the staging directory.
    for n,row in enumerate(sorted(inventory['entries'],key=lambda r:r['destination'])):
        assert set(row)=={'destination','source','sha256','size','category'}
        destination=safe(row['destination']); assert destination not in names; names.add(destination)
        assert destination.startswith('az/') and type(row['size']) is int and row['size']>=0
        path=Path(row['source']); assert path.is_absolute()
        raw=regular(path); assert (sha(raw),len(raw))==(row['sha256'],row['size']), f'changed input {destination}'
        inherited=by_identity.get((row['sha256'],row['size']))
        binding={k:row[k] for k in ('destination','sha256','size','category')}
        if inherited is not None:
            binding.update(resolution='ancestor',source_path='ancestors/ay/'+inherited)
        else:
            sid=f'local-{n:05d}'; rel=f'inputs/{sid}'
            binding.update(resolution='local',source_path=rel)
            local.append((rel,raw));sources.append(dict(id=sid,kind='local_file',path=rel))
            mounts.append(dict(id=sid,source_id=sid,path_prefix=destination,role='diagnostic_control' if row['category']=='diagnostic_control' else 'proof_origin'))
        rows.append(binding)
    profile=dict(schema='az-current-input-bindings-v1',version=1,probe='original-root-phase-clone-2026-10-09',ancestor_raw_manifest_sha256=AY_PIN,required_inventory_sha256=sha(inventory_raw),bindings=rows,source_root_policy='local source paths are relative to the fresh source-root; ancestor paths are verified imported closure slots',full_original_admitted=False)
    for sid,path,raw,destination in [
        ('binding-profile','metadata/AZ_INPUT_BINDINGS.json',encoded(profile),'az/evidence/AZ_INPUT_BINDINGS.json'),
        ('required-inventory','metadata/AZ_REQUIRED_INPUTS.json',inventory_raw,'az/evidence/AZ_REQUIRED_INPUTS.json'),
        ('capture-producer','metadata/prepare_capture.py',regular(Path(__file__)),'az/evidence/prepare_capture.py')]:
        local.append((path,raw));sources.append(dict(id=sid,kind='local_file',path=path));mounts.append(dict(id=sid,source_id=sid,path_prefix=destination,role='proof_origin'))
    assert not names.intersection(m['path_prefix'] for m in mounts[-3:]), 'metadata output collision'
    provenance={row['id']:row for row in ancestor['provenance']}
    imported=dict(id='ay',manifest_path=manifest_path.relative_to(repo).as_posix(),manifest_sha256=AY_PIN,cas_root=cas.relative_to(repo).as_posix(),mounts=[dict(source_mount_id=m['id'],path_prefix='ancestors/ay/'+m['path_prefix'],role=provenance[m['provenance_id']]['role']) for m in ancestor['mounts']])
    recipe=dict(schema='evidence-capture-recipe-v1',version=1,root_id='bytes-original-root-phase-clone-az-proof-origin',required_roles=sorted({m['role'] for m in imported['mounts']}|{'proof_origin'}),sources=sources,mounts=mounts,imports=[imported])
    stage.mkdir(parents=True)
    for path,raw in local:
        target=stage/path; target.parent.mkdir(parents=True,exist_ok=True)
        with target.open('xb') as out:out.write(raw)
    assert not args.recipe.exists(), 'recipe output already exists'
    with args.recipe.open('xb') as out:out.write(encoded(recipe))
    print(json.dumps(dict(source_root=str(stage),repository_root=str(repo),recipe=str(args.recipe),recipe_sha256=sha(encoded(recipe)),profile_sha256=sha(encoded(profile)),required_slots=len(rows),ancestor_slots=sum(r['resolution']=='ancestor' for r in rows),local_slots=sum(r['resolution']=='local' for r in rows),producer_sha256=sha(regular(Path(__file__))),status='staged explicit inputs; shared composer preflight and independent completeness audit still required'),indent=2))

if __name__=='__main__':main()
