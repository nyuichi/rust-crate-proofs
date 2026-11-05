from pathlib import Path
import json,tomllib,collections
base=Path(__file__).parent

def typ(t):
 if not t:return '?'
 if 'primitive' in t:return t['primitive']
 if 'generic' in t:return t['generic']
 if 'resolved_path'in t:
  p=t['resolved_path'];a=p.get('args');args=[]
  if a and 'angle_bracketed'in a:
   for x in a['angle_bracketed']['args']:args.append(typ(x['type']) if 'type'in x else str(x))
  return p['path']+('<'+', '.join(args)+'>' if args else '')
 if 'slice'in t:return '['+typ(t['slice'])+']'
 if 'array'in t:return '['+typ(t['array']['type'])+'; '+str(t['array']['len'])+']'
 if 'borrowed_ref'in t:return '&'+('mut ' if t['borrowed_ref']['is_mutable'] else '')+typ(t['borrowed_ref']['type'])
 return json.dumps(t,sort_keys=True)
summary={}
for name in ['byteorder','byteorder-lite','bit_field']:
 existing=tomllib.loads((base/name/'existing-proofs.toml').read_text())['tool']['proofs'];mappings={p['api']:p for p in existing}
 combined={};configs={}
 for cfg in (['default','no-default','all','i128-only'] if name=='byteorder' else ['default','no-default','all']):
  d=json.loads((base/name/cfg/'rustdoc.json').read_text());idx=d['index'];paths=d['paths'];root=idx[str(d['root'])];crate=root['name']
  reachable={};todo=[(d['root'],crate)]
  while todo:
   id,path=todo.pop();v=idx.get(str(id))
   if not v:continue
   inner=v['inner'];kind=next(iter(inner));reachable[str(id)]=path
   if kind=='module':
    for c in inner[kind]['items']:
     child=idx[str(c)];inn=child['inner'];k=next(iter(inn))
     if k=='use':
      u=inn[k];target=u.get('id')
      if target is not None:todo.append((target,path+'::'+u['name']))
     elif child['visibility']=='public':todo.append((c,path+'::'+(child['name'] or '?')))
  records=[]
  def add(key,category,v,**extra):
   rec={'key':key,'category':category,'name':v.get('name'),'span':v.get('span'),'attrs':v.get('attrs'),**extra}
   rec['proof_status']='unproved';rec['existing_mapping']=None
   if key in mappings:rec['proof_status']='proved_scoped';rec['existing_mapping']=mappings[key]
   records.append(rec)
  public_traits={id:idx[id] for id in reachable if 'trait'in idx[id]['inner']}
  public_types={id:idx[id] for id in reachable if next(iter(idx[id]['inner'])) in ['enum','struct','union','type_alias']}
  for id,path in reachable.items():
   v=idx[id];kind=next(iter(v['inner']));add(path,'public_'+kind,v,definition=v['inner'][kind])
   if kind=='trait':
    for child in v['inner']['trait']['items']:
     cv=idx[str(child)];ck=next(iter(cv['inner']));definition=cv['inner'][ck]
     add(path+'::'+cv['name'],'trait_declaration_'+ck,cv,trait=path,definition=definition,body_kind='default_body' if ck=='function' and definition['has_body'] else 'required_or_nonfunction')
  seen=set()
  for id,v in idx.items():
   if 'impl'not in v['inner']:continue
   im=v['inner']['impl'];tr=im.get('trait');selfty=typ(im['for']);trid=str(tr['id']) if tr else None
   local_trait=trid in public_traits; selfid=str(im['for'].get('resolved_path',{}).get('id'));public_self=selfid in public_types
   if not(local_trait or public_self):continue
   traitpath=reachable.get(trid) or ('::'.join(paths[trid]['path']) if trid in paths else (tr['path'] if tr else None))
   # Prefix local concrete receiver names, preserving primitive/generic receiver exactly.
   selfpath=reachable.get(selfid,selfty)
   implkey=f'<{selfpath} as {traitpath}>' if tr else selfpath
   derived='automatically_derived' in v.get('attrs',[]) and bool(traitpath and (traitpath.startswith('core::') or traitpath.startswith('std::')))
   external=not local_trait and (v['crate_id']!=0 or not v.get('span') or not v['span']['filename'].startswith('src/'))
   cat='excluded_derive_std_impl' if derived else ('external_blanket_impl' if external and im.get('blanket_impl') else 'external_synthetic_impl' if im['is_synthetic'] else 'external_trait_impl' if external else 'local_trait_impl' if tr else 'inherent_impl')
   add(implkey,cat,v,receiver=selfpath,trait=traitpath,definition=im)
   if derived:records[-1]['proof_status']='excluded_user_derive_std_only'
   for item in im['items']:
    cv=idx.get(str(item))
    if not cv:continue
    ck=next(iter(cv['inner']));key=implkey+'::'+str(cv['name'])
    add(key,cat+'_member_'+ck,cv,receiver=selfpath,trait=traitpath,definition=cv['inner'][ck],impl_key=implkey)
    if derived:records[-1]['proof_status']='excluded_user_derive_std_only'
   if external and not derived:
    for method in im['provided_trait_methods']:
     add(implkey+'::'+method,'inherited_external_default_function',{'name':method,'span':None,'attrs':[]},receiver=selfpath,trait=traitpath,definition=None,impl_key=implkey,signature_status='Dependency-owned default; exact dependency trait signature is not present in local rustdoc JSON. Tracked open external boundary.')
   if local_trait:
    byname={idx[str(c)]['name']:idx[str(c)] for c in public_traits[trid]['inner']['trait']['items']}
    for method in im['provided_trait_methods']:
     if method not in byname:continue
     cv=byname[method];ck=next(iter(cv['inner']));add(implkey+'::'+method,'inherited_local_default_'+ck,cv,receiver=selfpath,trait=traitpath,definition=cv['inner'][ck],impl_key=implkey)
  # deduplicate identical canonical routes within config, preserving metadata variants explicitly
  for r in records:
   key=(r['key'],r['category']);skey=json.dumps(key)
   if skey not in combined:combined[skey]={**r,'feature_configs':[]}
   combined[skey]['feature_configs'].append(cfg)
  configs[cfg]={'format_version':d['format_version'],'target':d['target'],'records':len(records),'categories':dict(collections.Counter(r['category'] for r in records))}
 rows=list(combined.values())
 import hashlib
 for r in rows:
  r['crate']=name;r['version']=d['crate_version'];r['local_id']=hashlib.sha256((name+'::'+r['key']+'::'+r['category']).encode()).hexdigest()[:20]
  r['rust_api_identity']=r['key'];r['server_identity']=r['existing_mapping']['api'] if r.get('existing_mapping') else None
  r['canonical_path']=r['key'];r['reexport_paths']=[];r['target']='x86_64-unknown-linux-gnu'
  r['defining_crate']='dependency (see trait path)' if 'external' in r['category'] else crate
  r['coverage']='partial-domain' if r['proof_status']=='proved_scoped' else 'explicitly excluded derived-standard' if r['proof_status'].startswith('excluded') else 'unproved'
  r['safety_preconditions']=r['existing_mapping']['precondition'] if r.get('existing_mapping') else 'Not yet specified; inspect signature/docs and callback/overflow/index requirements before proving.'
  r['intended_contract']='Existing theorem: exact endian bytes written with suffix preservation, or u8 bit read/update; see theorem and precondition.' if r.get('existing_mapping') else 'Unspecified: requires API-specific useful behavior and normal/error/panic/termination contract; not discharged.'
  r['remaining_obligations']='Feature and full-input domains outside the mapped theorem remain open.' if r.get('existing_mapping') else 'No successful mapped proof; exact behavioral contract and body/dependency proof remain open.'
  r['origin']='derived standard' if r['proof_status'].startswith('excluded') else 'inherited default' if r['category'].startswith('inherited') else 'external blanket or synthetic' if 'external' in r['category'] else 'macro manual' if name=='bit_field' and r.get('span',{} ) and r['span']['begin'][0]==282 else 'crate manual'
 proofrows=[r for r in rows if r['proof_status']=='proved_scoped'];excluded=[r for r in rows if r['proof_status'].startswith('excluded')]
 uncovered=[r for r in rows if r['proof_status']=='unproved'];missing=[m for m in mappings if not any(r['key']==m for r in rows)]
 output={'crate':name,'version':d['crate_version'],'scope':'Publicly reachable local declarations, local implementations, inherited local defaults, public-type external/blanket/synthetic implementations. Trait declarations and implementation instances are separate obligations; records are not a count of distinct executable bodies.','feature_configs':configs,'mapping_count':len(mappings),'matched_mapping_records':len(proofrows),'unmatched_mappings':missing,'records':rows}
 (base/name/'inventory.json').write_text(json.dumps(output,indent=2))
 text=f'# {name} {d["crate_version"]}: complete API tracking inventory\n\nDefault, no-default-features and all-features were collected separately from unchanged crates.io Rust source and manifest using Rust nightly-2026-09-17 rustdoc JSON and compiler expanded source. Target x86_64-unknown-linux-gnu.\n\n{len(rows)} tracking records; {len(proofrows)} existing scoped proof mappings matched; {len(excluded)} derive std records excluded by user instruction; {len(uncovered)} records have no established proof. These are API tracking records, not distinct body counts.\n\n'
 text+='## Counts\n\n| Category | Count |\n|---|---:|\n'+'\n'.join(f'| {k} | {v} |' for k,v in sorted(collections.Counter(r['category'] for r in rows).items()))+'\n\n## Every tracked API\n\n| API | Category | Features | Status |\n|---|---|---|---|\n'
 text+='\n'.join('| `'+r['key'].replace('|','\\|')+'` | '+r['category']+' | '+', '.join(r['feature_configs'])+' | '+r['proof_status']+' |' for r in rows)
 text+='\n\n## Interpretation and gaps\n\n`proved_scoped` retains the exact existing theorem precondition, not all-input or full-feature correctness. A specific receiver proof does not prove the generic trait declaration, inherited methods or I/O callbacks. Public type/alias/trait declarations need representation and contract review; no executable theorem is inferred from their presence. External standard-library blanket/synthetic implementations remain explicitly unproved, not silently excluded. Only automatically_derived standard trait implementations are excluded. The JSON retains exact signatures, type parameters, bounds, implementation generics, spans, attributes, provided defaults and per-feature membership.\n'
 (base/name/'INVENTORY.md').write_text(text)
 summary[name]={'records':len(rows),'matched_scoped_mappings':len(proofrows),'derive_std_excluded':len(excluded),'unproved_records':len(uncovered),'unmatched_mappings':missing,'categories':dict(collections.Counter(r['category'] for r in rows))}
(base/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2))
