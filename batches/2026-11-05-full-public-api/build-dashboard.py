#!/usr/bin/env python3
"""Normalize inventories without equating configurations, aliases or proof domains."""
from pathlib import Path
import argparse,collections,gzip,hashlib,json,shutil
p=argparse.ArgumentParser();p.add_argument('--source-root',type=Path,default=Path('/workspace/full-api'));a=p.parse_args();SRC=a.source_root;OUT=Path(__file__).resolve().parent
snap=OUT/'inventory-snapshots';snap.mkdir(exist_ok=True)
observations=[];inputs=[]
def stable(v):
 if isinstance(v,dict):return {k:stable(x) for k,x in v.items() if k not in ['id','rustdoc_id','raw_rustdoc_item_id','items','impls','links']}
 if isinstance(v,list):return [stable(x) for x in v]
 return v
def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':')).encode()).hexdigest()[:24]
def read(group,file):
 raw=file.read_bytes();rel=file.relative_to(SRC);name=str(rel).replace('/','__')+'.gz';(snap/name).write_bytes(gzip.compress(raw,mtime=0));inputs.append({'path':str(rel),'snapshot':'inventory-snapshots/'+name,'sha256':hashlib.sha256(raw).hexdigest(),'group':group});return json.loads(raw),len(inputs)-1
def add(r,group,crate,version,section,index,input_id,config=None):
 origin=str(r.get('origin','unspecified')).replace('_',' ').lower();category=r.get('category',r.get('item_kind',r.get('kind',section)))
 isimpl=section in ['impl_records','trait_impls'] or category=='implementation' or (category.endswith('_impl') and '_member_' not in category)
 callable_=not isimpl and (section=='callables' or category in ['callable','inherited-default'] or category.endswith('_function'))
 family='implementation_record' if isimpl else 'callable' if callable_ else 'noncallable_symbol'
 identity=r.get('rust_api_identity',r.get('canonical_api',r.get('key',r.get('path'))))
 paths=r.get('public_canonical_paths',r.get('public_paths',r.get('owner_public_paths',[]))) or []
 if not paths and r.get('canonical_path'):paths=[r['canonical_path']]
 if not identity:identity=' | '.join(paths) or str(r.get('name',r.get('owner_type',r.get('owner','unnamed'))))
 owner=r.get('owner_type',r.get('owner'));trait=r.get('trait_canonical_path',r.get('trait'))
 span=r.get('source_span',r.get('span'));signature=r.get('signature_ast',r.get('signature',r.get('definition',r.get('definition_ast'))))
 impl=r.get('impl_generics',r.get('generics',r.get('impl_ast',{}).get('generics')))
 excluded=('derived standard' in origin or category.startswith('excluded_derive_std'))
 prior=r.get('existing_theorem_evidence',r.get('existing_theorem',r.get('existing_proof',r.get('existing_mapping'))))
 workerstatus=r.get('coverage',r.get('proof_status','unproved'))
 prior=prior or (True if workerstatus in ['proved','partial-domain'] else None)
 cfgs=([config] if config else r.get('feature_configs',r.get('configurations_observed')) or [r.get('configuration',r.get('feature_configuration','unspecified'))])
 key=digest([crate,version,family,identity,owner,trait,stable(impl),stable(signature),span,origin])
 for cfg in cfgs:
  row={'observation_id':digest([key,cfg,input_id,section,index]),'rust_identity_group':key,'crate':crate,'version':version,'family':family,'item_kind':category,'rust_identity':identity,'public_paths':paths,'reexport_paths':r.get('reexport_paths',[]),'owner_type':owner,'trait':trait,'impl_generics':impl,'signature':signature,'source_span':span,'defining_crate':r.get('defining_crate'),'configuration':cfg,'target':r.get('target','x86_64-unknown-linux-gnu'),'origin':origin,'excluded_derived_standard':excluded,'inherited_boundary':('inherited' in origin or 'inherited' in category),'blanket_boundary':'blanket' in origin or 'blanket' in category,'auto_trait_boundary':'synthetic' in origin or 'synthetic' in category,'reexport_boundary':('alias' in origin or 'reexport' in section or len(paths)>1 or bool(r.get('reexport_paths'))),'server_identity':r.get('server_identity'),'server_identity_candidates':r.get('server_identity_candidates',[]),'worker_coverage':workerstatus,'existing_scoped_evidence':prior,'existing_proof_domain':r.get('existing_proof_domain',r.get('existing_domain')),'safety_preconditions':r.get('safety_preconditions'),'intended_contract':r.get('intended_behavioral_contract',r.get('intended_contract')),'remaining_obligations':r.get('remaining_obligations_or_blocker',r.get('remaining_obligations','Specification, body/dependency proof and full domain/configuration coverage remain to be reviewed.')),'obstacle':r.get('obstacle'),'integrated_status':'excluded_derived_standard' if excluded else 'prior_scoped_evidence_not_full_api_closure' if prior else 'unproved_or_interface_obligation','known_body_contract_evidence':('adapted body contract proved under the original report model' if crate=='normalize-line-endings' else 'original body contract proved under the original report models/preconditions') if prior else None,'configuration_applicability_of_prior_evidence':'must be checked against the original proof domain; name matching alone is insufficient' if prior else None,'api_full_domain_coverage':'unassessed','strict_api_obligation_complete':None,'raw_reference':{'input':input_id,'section':section,'index':index}}
  observations.append(row)
# Already merged records are expanded to observations for each recorded configuration.
for n in ['byteorder','byteorder-lite','bit_field']:
 d,i=read('bits',SRC/'bits'/n/'inventory.json')
 for ix,r in enumerate(d['records']):add(r,'bits',n,d['version'],'records',ix,i)
for n in ['either','base16ct','ascii']:
 d,i=read('enum',SRC/'enum'/n/'coverage-gap.json')
 for ix,r in enumerate(d['entries']):add(r,'enum',n,d['version'],'entries',ix,i)
versions={'oorandom':'11.1.5','rand_pcg':'0.10.2','rand_xorshift':'0.5.0','rand_xoshiro':'0.8.1','endian-type':'0.2.0','nohash-hasher':'0.2.0','normalize-line-endings':'0.3.0'}
for n in ['oorandom','rand_pcg','rand_xorshift','rand_xoshiro']:
 d,i=read('prng',SRC/'prng'/(n+'.json'))
 for sec in ['callables','trait_impls','symbols_and_associated_items','external_crate_reexports']:
  for ix,r in enumerate(d.get(sec,[])):add(r,'prng',n,d['version'],sec,ix,i)
for n in ['endian-type','nohash-hasher','normalize-line-endings']:
 for file in sorted((SRC/'endian'/n).glob('*/inventory.json')):
  d,i=read('endian',file);cfg=file.parent.name
  for sec in ['callables','impl_records','types_constants_and_associated_items']:
   for ix,r in enumerate(d.get(sec,[])):add(r,'endian',n,versions[n],sec,ix,i,cfg)
assert len(set(r['crate'] for r in observations))==13
metrics=[]
for n in sorted(set(r['crate'] for r in observations)):
 rs=[r for r in observations if r['crate']==n];eligible=[r for r in rs if not r['excluded_derived_standard']]
 metrics.append({'crate':n,'version':rs[0]['version'],'configurations_observed':sorted(set(r['configuration'] for r in rs)),'observation_count':len(rs),'rust_identity_groups':len(set(r['rust_identity_group'] for r in rs)),'by_family':dict(collections.Counter(r['family'] for r in rs)),'by_origin':dict(collections.Counter(r['origin'] for r in rs)),'derived_standard_excluded_observations':len(rs)-len(eligible),'eligible_observations':len(eligible),'prior_scoped_evidence_observations':sum(bool(r['existing_scoped_evidence']) for r in eligible),'inherited_boundary_observations':sum(r['inherited_boundary'] for r in rs),'blanket_boundary_observations':sum(r['blanket_boundary'] for r in rs),'reexport_boundary_observations':sum(r['reexport_boundary'] for r in rs),'strict_complete_observations':0,'full_api_closure_unassessed_observations':len(eligible),'whole_crate_api_completion':False})
summary={'schema_version':1,'scope':'Observed compiler inventories only; observations are not distinct executable bodies; configuration coverage is not all cfg combinations. All prior proof evidence remains scoped and does not close full-API obligations.','source_snapshot_base':'104fe4cc3fe53eeb83686ab65e53cbd14550abe2','crate_count':13,'strict_whole_crate_completions':0,'inventory_input_count':len(inputs),'observation_count':len(observations),'metrics':metrics,'inputs':inputs}
(OUT/'dashboard.json').write_text(json.dumps(summary,indent=2)+'\n')
(OUT/'api-observations.json.gz').write_bytes(gzip.compress((json.dumps(observations,indent=2)+'\n').encode(),mtime=0))
obsdir=OUT/'observations';obsdir.mkdir(exist_ok=True)
for n in sorted(set(r['crate'] for r in observations)):
 (obsdir/(n+'.json.gz')).write_bytes(gzip.compress((json.dumps([r for r in observations if r['crate']==n],indent=2)+'\n').encode(),mtime=0))
old=OUT/'api-observations.json'
if old.exists():old.unlink()
updates=OUT/'proof-updates.json'
if not updates.exists():updates.write_text(json.dumps({'schema_version':1,'policy':'Add only after checking source/body/specification/success and failure domains/model boundaries/integration evidence. Never silently promote prior mappings to full coverage.','updates':[]},indent=2)+'\n')
for f in ['inventory-schema.md']:
 shutil.copy2(SRC/f,OUT/f)
for f in ['identity-and-publication.md','current-coverage-audit.json']:shutil.copy2(SRC/'integration'/f,OUT/f)
print(json.dumps({'crates':13,'observations':len(observations),'input_snapshots':len(inputs),'strict_whole_crate_completions':0}))
