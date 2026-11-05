from pathlib import Path
import json,tomllib,hashlib,collections,re
B=Path('/workspace/full-api/prng'); V={'oorandom':'11.1.5','rand_pcg':'0.10.2','rand_xoshiro':'0.8.1','rand_xorshift':'0.5.0'}
DEP=json.load(open(B/'rustdoc/dependencies/x86_64-unknown-linux-gnu/doc/rand_core.json'))
DTRAITS={v['name']:v for v in DEP['index'].values() if 'trait' in v['inner']}
MODES=['default','no-default-features','all-features']
def canonical(j,p):
 if not p:return None
 return '::'.join(j['paths'].get(str(p['id']),{}).get('path',[p['path']]))
def ty(j,t):
 if t is None:return '()'
 if 'resolved_path' in t:
  p=t['resolved_path'];return canonical(j,p)+args(j,p.get('args'))
 if 'slice' in t:return '['+ty(j,t['slice'])+']'
 if 'primitive' in t:return t['primitive']
 if 'generic' in t:return t['generic']
 if 'borrowed_ref' in t:
  p=t['borrowed_ref'];return '&'+('mut ' if p['is_mutable'] else '')+ty(j,p['type'])
 if 'tuple' in t:return '('+','.join(ty(j,x) for x in t['tuple'])+')'
 if 'array' in t:return '['+ty(j,t['array']['type'])+';'+t['array']['len']+']'
 return json.dumps(t,sort_keys=True,separators=(',',':'))
def args(j,a):
 if not a:return ''
 a=a.get('angle_bracketed',{});items=[]
 for x in a.get('args',[]):
  items.append(ty(j,x['type']) if 'type' in x else str(x.get('lifetime',x.get('const',x))))
 if a.get('constraints'):items.append(json.dumps(a['constraints'],sort_keys=True))
 return '<'+','.join(items)+'>' if items else ''
def exported(j):
 out=collections.defaultdict(list)
 def visit(id,prefix):
  n=j['index'][str(id)];inn=n['inner']
  if 'module' in inn:
   if id != j['root']:prefix += '::'+n['name']
   for i in inn['module']['items']:
    x=j['index'][str(i)]
    if x['visibility']=='public':visit(i,prefix)
  elif 'use' in inn:
   u=inn['use']
   if u['id'] is not None:out[str(u['id'])].append(prefix+'::'+u['name'])
  else:
   path=prefix+'::'+n['name'];out[str(id)].append(path)
   if 'module' in inn:visit(id,path)
 visit(j['root'],j['index'][str(j['root'])]['name'])
 return out
def contract(m):
 if m=='from_seed':return 'Exact seed-byte to internal-state initialization, including the documented zero-seed replacement behavior.'
 if m=='seed_from_u64':return 'Exact u64 seed expansion and resulting initialized state.'
 if m in ['new','new_inc']:return 'Exact initialized PCG state/stream and constructor warm-up steps for every documented seed.'
 if m=='from_state':return 'Preserve the supplied state/stream representation exactly.'
 if m in ['state','get_state','get_stream']:return 'Return the exact current internal state/stream without mutation.'
 if m in ['set_state','set_stream']:return 'Exact documented replacement of state/stream, with any normalization explicitly stated.'
 if m in ['next_u32','next_u64','try_next_u32','try_next_u64','rand_u32','rand_u64','rand_i32','rand_i64']:return 'Exact output word and state transition, including modular arithmetic and failure/termination domain.'
 if m in ['fill_bytes','try_fill_bytes']:return 'Exact destination byte sequence and final state for arbitrary buffer lengths; account for leftover bytes and callback behavior.'
 if m in ['jump','long_jump','advance']:return 'Exact state advancement/jump polynomial for arbitrary initial states and advance amounts.'
 if m=='rand_float':return 'Exact floating-point construction, output interval and state transition.'
 if m=='rand_range':return 'Exact bounded-range behavior, rejection-loop progress/termination conditions and invalid-range failure behavior.'
 if m in ['from_rng','try_from_rng','fork','try_fork']:return 'Exact initialization/composition conditional on explicit external RNG callback contracts, including propagated errors and consumed bytes.'
 if m in ['serialize','deserialize','deserialize_in_place']:return 'Exact serde representation/state correspondence, errors and serializer/deserializer callback assumptions.'
 if m=='fmt':return 'Exact formatter output or explicit formatter callback contract and propagated error behavior.'
 return 'Specify the documented output/state/reference behavior and all success, error, panic and termination conditions before proving this body.'
ALL={};summary=[]
for crate,ver in V.items():
 src=B/'sources'/f'{crate}-{ver}';manifest=tomllib.loads((src/'Cargo.toml').read_text()); cfg=[]
 for p in (src/'src').rglob('*.rs'):
  for n,line in enumerate(p.read_text().splitlines(),1):
   if '#[cfg' in line:cfg.append({'file':str(p.relative_to(src)),'line':n,'text':line.strip()})
 old=tomllib.loads((Path('/workspace/publish-batch-ten')/crate/ver/'aeneas/2026-11-05/proofs.toml').read_text())
 mappings=old['tool']['proofs'];records=[];impls=[];symbols=[];reexports=[]
 for mode in MODES:
  j=json.load(open(B/f'rustdoc/{crate}/{mode}/x86_64-unknown-linux-gnu/doc/{crate}.json'));idx=j['index'];ex=exported(j)
  for alias_id,alias_paths in list(ex.items()):
   alias_node=idx.get(alias_id,{})
   target=alias_node.get('inner',{}).get('type_alias',{}).get('type',{}).get('resolved_path')
   if target and str(target['id']) in idx:
    for alias_path in alias_paths:
     if alias_path not in ex[str(target['id'])]:ex[str(target['id'])].append(alias_path)
  types={k:v for k,v in idx.items() if str(k) in ex and any(x in v['inner'] for x in ['struct','enum','union'])}
  for k,paths in ex.items():
   n=idx.get(k)
   if n:
    symbols.append({'configuration':mode,'public_paths':paths,'kind':next(iter(n['inner'])),'name':n['name'],'source_span':n['span'],'raw':n['inner']})
   elif paths and paths[0].endswith('::rand_core'):reexports.append({'configuration':mode,'public_paths':paths,'defining_crate':'rand_core','scope':'All public rand_core items; separate dependency inventory below. No bodies are covered by existing crate proofs.'})
  for type_id,type_node in types.items():
   for field_id,field_node in idx.items():
    if field_node['visibility']=='public' and field_node['span'] and field_node['crate_id']==0 and 'struct_field' in field_node['inner']:
     shape=type_node['inner'].get('struct',{}).get('kind',{})
     if str(field_id) in {str(x) for x in (shape.get('tuple',[]) if isinstance(shape,dict) else [])}:
      symbols.append({'configuration':mode,'owner_public_paths':ex[type_id],'kind':'public_field','name':field_node['name'],'raw':field_node['inner'],'source_span':field_node['span'],'coverage':'unproved_data_invariant'})
  for n in idx.values():
   if 'impl' not in n['inner']:continue
   imp=n['inner']['impl'];tp=imp['for'].get('resolved_path')
   if tp is None or str(tp['id']) not in types:continue
   owner=ty(j,imp['for']);trait=canonical(j,imp['trait']);trait_full=(trait+args(j,imp['trait'].get('args'))) if trait else None
   derived='automatically_derived' in n['attrs'];std=trait and trait.split('::')[0] in ['core','std','alloc'];blanket=imp['blanket_impl'] is not None
   synthetic=imp['is_synthetic']
   origin='external_blanket' if blanket else 'synthetic_auto_trait' if synthetic else 'derived_standard' if derived and std else 'derived_nonstandard' if derived else 'crate_manual'
   span=n['span'];fn=(span or {}).get('filename','')
   if origin=='crate_manual' and span and (src/fn).is_file():
    source_line=(src/fn).read_text().splitlines()[span['begin'][0]-1]
    if re.search(r'\w+!\s*\(',source_line):origin='macro_manual'
   implrecord={'configuration':mode,'owner':owner,'owner_public_paths':ex[str(tp['id'])],'trait':trait_full,'origin':origin,'generics':imp['generics'],'blanket_impl':imp['blanket_impl'],'is_synthetic':synthetic,'source_span':span,'provided_trait_methods':imp['provided_trait_methods'],'attrs':n['attrs'],'rustdoc_id':n['id']}
   impls.append(implrecord)
   own_names=set()
   def add(method,raw,kind,method_origin,source,signature=None):
    canonical_api=(f'<{owner} as {trait_full}>::{method}' if trait_full else owner+'::'+method)
    ident={'crate':crate,'version':ver,'canonical_api':canonical_api,'configuration':mode,'target':'x86_64-unknown-linux-gnu','kind':kind,'impl_generics':imp['generics']}
    key=hashlib.sha256(json.dumps(ident,sort_keys=True).encode()).hexdigest()[:20]
    status='explicitly_excluded_derived_standard' if method_origin=='derived_standard' else 'unproved'
    theorem=None
    # Four prior public reports are default-feature, one-step proofs only.
    for m in mappings:
     api=m['api'];shortowner=types[str(tp['id'])]['name']
     if method==api.rsplit('::',1)[-1] and shortowner in api and (mode in ['default','no-default-features'] or crate=='oorandom') and method_origin not in ['external_blanket','inherited_default']:
      status='modeled_boundary' if crate=='rand_xorshift' else 'proved';theorem=m;break
    records.append(dict(ident,local_id=key,public_paths=[p+'::'+method for p in ex[str(tp['id'])]],owner=owner,trait=trait_full,method=method,signature=signature,raw_item=raw,defining_crate=(trait.split('::')[0] if method_origin in ['external_blanket','inherited_default'] and trait else crate),source_span=source,origin=method_origin,safety_preconditions='Unsafe-call obligations and generic trait bounds apply as given by the retained signature.' if signature and signature.get('header',{}).get('is_unsafe') else 'Well-typed values and declared generic bounds; additional behavioral preconditions remain to be specified.',intended_contract=contract(method),existing_theorem=theorem,existing_domain=theorem['precondition'] if theorem else None,coverage=status,remaining_obligations=[] if status in ['proved','explicitly_excluded_derived_standard'] else ['Prove original body and relevant callees for this exact specialization, feature configuration and behavioral domain.'],obstacle='Std fmt model carries a Formatter type axiom; callable formatter contracts need a supported model.' if method=='fmt' else 'External callback/serde behavior must be modeled with explicit contracts.' if method in ['serialize','deserialize','deserialize_in_place','from_rng','try_from_rng','fork','try_fork'] else None))
   for id in imp['items']:
    item=idx[str(id)];kind=next(iter(item['inner']));own_names.add(item['name'])
    if kind=='function':
     if trait or item['visibility']=='public':add(item['name'],item,kind,origin,item['span'],item['inner']['function'])
    else:symbols.append({'configuration':mode,'owner':owner,'trait':trait_full,'origin':origin,'kind':kind,'name':item['name'],'raw':item['inner']})
   # Inherited dependency defaults have no item node in this crate JSON.
   for name in imp['provided_trait_methods']:
    if name in own_names:continue
    if origin=='derived_standard':o='derived_standard'
    else:o='inherited_default'
    d=DTRAITS.get(imp['trait']['path'] if imp['trait'] else '')
    ds=[DEP['index'][str(id)] for id in d['inner']['trait']['items'] if DEP['index'][str(id)]['name']==name] if d else []
    add(name,ds[0] if ds else {'dependency_trait_default':True},'function',o,ds[0]['span'] if ds else None,ds[0]['inner'].get('function') if ds else None)
  # Public free functions (none in these crate roots currently).
  for id,paths in ex.items():
   item=idx.get(id)
   if item and 'function' in item['inner']:
    records.append({'local_id':hashlib.sha256((mode+paths[0]).encode()).hexdigest()[:20],'crate':crate,'version':ver,'canonical_api':paths[0],'configuration':mode,'target':'x86_64-unknown-linux-gnu','kind':'function','owner':None,'trait':None,'method':item['name'],'signature':item['inner']['function'],'origin':'crate_manual','coverage':'unproved','public_paths':paths,'source_span':item['span'],'intended_contract':contract(item['name'])})
 default_ids={x['canonical_api'] for x in records if x['configuration']=='default'}
 for entry in records:
  entry['active_features']=['serde'] if entry['configuration']=='all-features' and 'serde' in manifest.get('features',{}) else []
  entry['cfg_requirement']='feature = serde' if entry['canonical_api'] not in default_ids and 'serde' in manifest.get('features',{}) else 'Available without local feature gates; see retained source cfg scan.'
 rawhashes={str(p.relative_to(src)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [src/'Cargo.toml',*sorted((src/'src').rglob('*.rs'))]}
 result={'crate':crate,'version':ver,'toolchain':'nightly-2026-09-17','rustdoc_format':61,'target':'x86_64-unknown-linux-gnu','source_reference':'/workspace/publish-batch-ten/'+crate+'/'+ver+'/aeneas/2026-11-05','source_sha256':rawhashes,'manifest_features':manifest.get('features',{}),'configurations':MODES,'cfg_source_scan':cfg,'configuration_notes':'No default feature is declared. For rand crates the sole crate feature is serde, so default/no-default select the same local feature subset and all-features selects serde. Tests are excluded. This is not proof of other targets or dependency feature combinations.','callables':records,'trait_impls':impls,'symbols_and_associated_items':symbols,'external_crate_reexports':reexports,'existing_proofs_toml':mappings,'inherited_boundary_notes':'Synthetic auto traits with no methods are retained as implementation metadata. External blanket implementations are listed separately and remain unproved. Default trait methods are listed even if no local body exists. rand_core crate reexports are an explicit external API boundary, with its own inventory. Generic impl bounds and type/lifetime arguments are retained; no downstream-created implementation is claimed body-proved.'}
 (B/f'{crate}.json').write_text(json.dumps(result,indent=2))
 counts=collections.Counter((r['configuration'],r['origin'],r['coverage']) for r in records)
 summary.append((crate,len(records),counts));ALL[crate]=result
# Shared dependency-reexport API items; preserve public callable trait definitions.
ex=exported(DEP);depitems=[]
for id,paths in ex.items():
 n=DEP['index'].get(id)
 if not n:continue
 if 'function' in n['inner']:depitems.append({'paths':paths,'kind':'function','item':n,'coverage':'unproved_external_reexport'})
 if 'trait' in n['inner']:
  for i in n['inner']['trait']['items']:
   x=DEP['index'][str(i)]
   if 'function' in x['inner']:depitems.append({'paths':[p+'::'+x['name'] for p in paths],'kind':'trait_method','trait':n['name'],'item':x,'coverage':'unproved_external_reexport'})
# Keep concrete dependency type impl methods and defaults as explicit reexport obligations.
for n in DEP['index'].values():
 if 'impl' not in n['inner']:continue
 imp=n['inner']['impl'];tp=imp['for'].get('resolved_path')
 if not tp or str(tp['id']) not in ex:continue
 owner=ty(DEP,imp['for']);trait=canonical(DEP,imp['trait'])
 trait_full=trait+args(DEP,imp['trait'].get('args')) if trait else None
 own=set()
 for id in imp['items']:
  item=DEP['index'][str(id)];own.add(item['name'])
  if 'function' in item['inner'] and (trait or item['visibility']=='public'):
   depitems.append({'paths':[p+'::'+item['name'] for p in ex[str(tp['id'])]],'canonical_api':f'<{owner} as {trait_full}>::'+item['name'] if trait else owner+'::'+item['name'],'kind':'impl_method','trait':trait_full,'owner':owner,'impl_generics':imp['generics'],'blanket_impl':imp['blanket_impl'],'item':item,'coverage':'unproved_external_reexport'})
 for name in imp['provided_trait_methods']:
  if name not in own:depitems.append({'paths':[p+'::'+name for p in ex[str(tp['id'])]],'canonical_api':f'<{owner} as {trait_full}>::'+name,'kind':'inherited_default','trait':trait_full,'owner':owner,'impl_generics':imp['generics'],'coverage':'unproved_external_reexport'})
(B/'rand_core-reexports.json').write_text(json.dumps({'crate':'rand_core','version':DEP['crate_version'],'consumer_crates':['rand_pcg','rand_xoshiro','rand_xorshift'],'callables':depitems,'public_symbol_ids_and_paths':dict(ex),'public_trait_definitions':[n for n in DEP['index'].values() if 'trait' in n['inner'] and str(n['id']) in ex],'complete_raw_rustdoc':'rustdoc/dependencies/x86_64-unknown-linux-gnu/doc/rand_core.json','note':'Each public rand_core path is additionally reachable as consumer::rand_core::<path_without_rand_core>. Generic std blankets on dependency types form a separate open boundary; not claimed verified.'},indent=2))
lines=['# PRNG public API coverage inventory','','Compiler inventory of unchanged source; verification is incomplete. Runtime identity retains owner, trait/type arguments and configuration. Server API labels may erase these distinctions.','']
for crate,_,_ in summary:
 r=ALL[crate];lines += ['## '+crate+' '+V[crate],'','| Configuration | Callables | Excluded derive std | Existing proved/model boundary | Unproved |','|---|---:|---:|---:|---:|']
 for mode in MODES:
  rows=[x for x in r['callables'] if x['configuration']==mode];c=collections.Counter(x['coverage'] for x in rows)
  lines.append(f"| {mode} | {len(rows)} | {c['explicitly_excluded_derived_standard']} | {c['proved']+c['modeled_boundary']} | {c['unproved']} |")
 lines += ['','Every listed callable remains a separate JSON entry. Auto traits/associated items/type aliases/public data shapes are retained separately. All serde derives remain in scope.','', '| Owner | Trait | Method | Origin | Coverage |','|---|---|---|---|---|']
 for x in r['callables']:
  if x['configuration']=='all-features' and x['origin'] not in ['derived_standard','external_blanket']:
   lines.append('| '+x['owner']+' | '+str(x['trait'] or '')+' | '+x['method']+' | '+x['origin']+' | '+x['coverage']+' |')
 lines+=['']
lines+=['## Configuration and boundaries','','All twelve rustdoc builds succeeded: default, no-default and all features for four crates, on x86_64 only. Expanded source for default/all features is retained. No local default feature is declared; only the three rand crates have the serde feature, so these configurations cover the local feature subsets. Platform-dependent integer/float semantics and dependency feature combinations remain separate obligations.','','The rand crates publicly reexport rand_core. Its trait, free-function and concrete-type implementation callable inventory is retained in rand_core-reexports.json; these dependency bodies are unproved. Standard automatic blanket and inherited defaults remain separately enumerated. No API has been silently excluded except actual derive-generated standard implementations.','','Existing reports prove only one output/state step per crate. serde configurations are explicitly unproved even when source bodies look unchanged. Constructor/seed/composition, manual Debug, buffer filling, floating/ranged output and every additional generator variant require additional proofs.','','The next tractable increment is oorandom state/from_state and rand_xoshiro macro-authored state accessors, followed by seed initialization and next_u64 compositions. Formatter/serde callbacks and rejection-loop termination require explicit semantic boundaries.']
(B/'INVENTORY.md').write_text('\n'.join(lines)+'\n')
for crate,_,_ in summary:
 r=ALL[crate];print(crate,collections.Counter((x['configuration'],x['coverage']) for x in r['callables']))
print('external rand_core callables',len(depitems))
