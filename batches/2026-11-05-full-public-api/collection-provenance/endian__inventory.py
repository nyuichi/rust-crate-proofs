from pathlib import Path
import json,hashlib,tomllib,re,collections
ROOT=Path(__file__).resolve().parent
STDDIR=Path('/workspace/proof-tools/rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/share/doc/rust/json')
STD={n:json.loads((STDDIR/(n+'.json')).read_text()) for n in ['core','alloc','std']}
GLOBAL={}
for dn,d in STD.items():
 for id,item in d['index'].items():
  path=d['paths'].get(id,{}).get('path')
  if path:GLOBAL['::'.join(path)]=(d,item)
def path(d,p):
 id=str(p.get('id',''));return '::'.join(d.get('paths',{}).get(id,{}).get('path',[])) or p.get('path','?')
def args(d,a):
 if not a:return ''
 if 'angle_bracketed' in a:
  x=a['angle_bracketed'];v=[]
  for g in x['args']:
   if 'type' in g:v.append(ty(d,g['type']))
   elif 'lifetime' in g:v.append(g['lifetime'])
   else:v.append(json.dumps(g,sort_keys=True))
  for c in x.get('constraints',[]):v.append(c['name']+'='+json.dumps(c['binding'],sort_keys=True))
  return '<'+', '.join(v)+'>' if v else ''
 return json.dumps(a,sort_keys=True)
def ty(d,t):
 if not isinstance(t,dict):return str(t)
 if 'resolved_path' in t:
  p=t['resolved_path'];return path(d,p)+args(d,p.get('args'))
 if 'generic' in t:return t['generic']
 if 'primitive' in t:return t['primitive']
 if 'borrowed_ref' in t:
  p=t['borrowed_ref'];return '&'+((p.get('lifetime') or '')+' ' if p.get('lifetime') else '')+('mut ' if p['is_mutable'] else '')+ty(d,p['type'])
 if 'raw_pointer' in t:
  p=t['raw_pointer'];return '*'+('mut ' if p['is_mutable'] else 'const ')+ty(d,p['type'])
 if 'tuple' in t:return '('+', '.join(ty(d,v) for v in t['tuple'])+')'
 if 'slice' in t:return '['+ty(d,t['slice'])+']'
 if 'array' in t:return '['+ty(d,t['array']['type'])+'; '+t['array']['len']+']'
 if 'qualified_path' in t:
  p=t['qualified_path'];return '<'+ty(d,p['self_type'])+' as '+(path(d,p['trait']) if p.get('trait') else '?')+'>::'+p['name']+args(d,p.get('args'))
 return json.dumps(t,sort_keys=True)
def trait_def(d,tr):
 if not tr:return None
 item=d['index'].get(str(tr['id']))
 if item and 'trait' in item['inner']:return d,item
 p=path(d,tr)
 if p in GLOBAL and 'trait' in GLOBAL[p][1]['inner']:return GLOBAL[p]
 candidates=[(dd,i) for dd in STD.values() for i in dd['index'].values() if i['name']==tr['path'].split('::')[-1] and 'trait' in i['inner']]
 return candidates[0] if len(candidates)==1 else None
def get_trait_method(d,tr,name):
 found=trait_def(d,tr)
 if not found:return None
 dd,t=found
 for id in t['inner']['trait']['items']:
  i=dd['index'].get(str(id))
  if i and i['name']==name:return dd,i
 return None

def inventory(name,ver,config,d,source,alias_label=None,alias_target=None):
 entries=[];implrows=[];types=[];proofs=tomllib.loads((source/'proofs.toml').read_text())['tool']['proofs']
 proof_by_method={}
 for p in proofs:proof_by_method.setdefault(p['api'].split('::')[-1],[]).append(p)
 def add(i,imp=None,implitem=None,origin='crate_manual',inherited=False,dd=None):
  dd=dd or d; inner=i['inner'];kind=list(inner)[0]
  owner=ty(d,imp['for']) if imp else None
  tr=path(d,imp['trait'])+args(d,imp['trait'].get('args')) if imp and imp.get('trait') else None
  meth=i['name'];canonical=(f'<{owner} as {tr}>::{meth}' if tr else f'{owner}::{meth}' if owner else name.replace('-','_')+'::'+meth)
  defining_canonical=canonical
  if alias_label:
   alias_types={'IntMap':'nohash_hasher::IntMap<K, V>','IntSet':'nohash_hasher::IntSet<T>','BuildNoHashHasher':'nohash_hasher::BuildNoHashHasher<T>'}
   stdname=alias_target['standard_target'][1]
   def replace_owner(n):
    if isinstance(n,dict):
     if 'resolved_path' in n and n['resolved_path']['path'].split('::')[-1]==stdname:return {'generic':alias_types[alias_label]}
     return {k:replace_owner(v) for k,v in n.items()}
    if isinstance(n,list):return [replace_owner(v) for v in n]
    return n
   exposed_owner=ty(d,replace_owner(imp['for']))
   canonical=f'<{exposed_owner} as {tr}>::{meth}' if tr else f'{exposed_owner}::{meth}'
  availability='conditional_on_recorded_impl_and_method_bounds'
  if name=='normalize-line-endings' and inherited and meth in ['rev','rposition','cycle','copied','cloned','__iterator_get_unchecked']:
   availability='not_callable_for_the_original_opaque_return_type: additional Self/Item bound is not implemented (Item=char; no Clone/DoubleEndedIterator/ExactSizeIterator/TrustedRandomAccess impl)'

  attr=implitem.get('attrs',[]) if implitem else []
  derived='automatically_derived' in attr
  stdtrait=bool(tr and tr.startswith(('core::','std::','alloc::')))
  excluded=derived and stdtrait
  if excluded:origin='derived_standard'
  cov='explicitly_excluded_derived_standard' if excluded else 'unproved'
  evidence=[];scope=None
  if not alias_label and not inherited and not excluded:
   for p in proof_by_method.get(meth,[]):
    ownerbase=(owner or '').split('::')[-1].split('<')[0]
    matches=(not imp and name=='normalize-line-endings') or (imp and ownerbase in p['api'] and (not tr or tr.split('<')[0].split('::')[-1] in p['api']))
    if matches:
     cov='proved' if name!='normalize-line-endings' else 'partial_domain_adapted_source'
     if name=='nohash-hasher' and config.startswith('no-default'):cov='unproved_feature_configuration'
     evidence.append(p)
     scope=p['precondition']
  if name=='normalize-line-endings' and meth=='next' and not inherited:
   cov='partial_domain_adapted_source';scope='Aeneas adapted char-guard source; Kani one-step equality only for terminating nonpanic two-reply iterator model. Not original-body Aeneas or arbitrary iterator contextual equivalence.'
  unsafe=inner.get('function',{}).get('header',{}).get('is_unsafe',False)
  blocker=None;contract=None;safety=[]
  if name=='endian-type' and not alias_label and cov=='unproved':
   if meth in ['from_bytes','to_bytes']:
    blocker='Original bodies use transmute; Aeneas byte-layout/transmute semantics and a representation relation must be established. All integer instantiations remain unproved.';contract='Exact conversion between wrapper memory representation and fixed-size byte array, including both directions and native target layout.'
   elif meth=='from_byte_slice':
    blocker='Unsafe raw-pointer read and alignment/initialized-value obligations are not covered by the existing bitwise proofs.';safety=['Sufficient byte length is only a debug assertion in the source.','The pointer must satisfy alignment and valid initialized T representation; arbitrary Sized+Copy T need not accept every byte pattern.','Pointer/provenance and read access must be valid.'];contract='Under complete valid-read preconditions, return the exact native-memory T representation wrapped in the selected endian type.'
   elif meth in ['as_byte_slice','as_byte_slice_mut']:
    blocker='Raw slice construction, validity of all exposed bytes (including padding), aliasing, and mutation preserving valid T must be analyzed; no blanket safety claim.';safety=['All exposed size_of::<T>() bytes must be initialized for readable u8 access.','Mutable byte writes must preserve a valid T before typed use and obey aliasing/provenance requirements.'];contract='Expose exactly the native-memory bytes of the wrapped T with correct lifetime/aliasing behavior.'
   elif tr and tr.split('<')[0].endswith('::From'):
    blocker='Original scalar endian conversion/swap_bytes or cross-endian conversion not proved. Explicit From trait-generic roots were rejected by pinned Charon; wildcard root/exclude remains an original-source extraction route to investigate.';contract='Integer conversion represents the documented endian byte order; cross-wrapper conversion preserves the represented integer and round trips.'
  if name=='nohash-hasher' and meth=='fmt' and origin=='crate_manual':blocker='Pinned Aeneas Formatter is opaque and Debug/write_str/write_fmt helpers use a simplistic model ignoring emitted text. A meaningful exact formatting specification needs a strengthened model; proving model .ok alone is insufficient.';contract='Emit the same Debug text as the original tuple Debug builder, preserving formatter failures and flags.'
  if name=='normalize-line-endings' and cov=='partial_domain_adapted_source':blocker='Original char Switch unsupported in pinned Aeneas. Existing adapted-source Lean proof and restricted Kani local equivalence do not discharge arbitrary original iterator histories, callbacks, panic/divergence, or inherited Iterator methods.';contract='CR/LF normalization for original opaque iterator, with precise callback and iterator-history semantics, including finite output correctness and supported repeated-call behavior.'
  if inherited or origin=='external_blanket' or alias_label:
   blocker=blocker or 'Dependency/default/blanket body is not proved by the crate-local theorem; generic bounds and callback/destructor semantics must be preserved.'
   contract=contract or 'Meet the upstream public method contract under the recorded specialization and generic bounds; prove required composition with the crate body rather than infer it from one-step helper coverage.'
  if cov=='proved':contract=contract or ('Exact underlying operator result preservation for arbitrary operator dictionaries; no independent proof of concrete callback implementations.' if name=='endian-type' else 'Exact per-method state/result contract under the stored debug/release theorem domain.')
  sig=inner.get('function') or inner
  stable_hash=hashlib.sha256(json.dumps([name,ver,config,canonical,origin,i.get('span'),imp and imp.get('generics')],sort_keys=True).encode()).hexdigest()[:20]
  entries.append({'crate':name,'version':ver,'local_id':stable_hash,'rust_api_identity':canonical,'public_canonical_paths':[canonical],'defining_api_identity':defining_canonical,'availability':availability,'server_identity':None,'server_identity_status':'not_inferred_from_erased_catalogue_names','kind':kind,'name':meth,'owner_type':owner,'owner_type_ast':imp and imp['for'],'trait':tr,'trait_ast':imp and imp.get('trait'),'impl_generics':imp and imp['generics'],'signature_ast':sig,'defining_crate':dd.get('external_crates',{}).get(str(i.get('crate_id')),{}).get('name', dd['index'].get(str(dd['root']),{}).get('name')),'source_span':i.get('span'),'impl_source_span':implitem and implitem.get('span'),'configuration':config,'target':'x86_64-unknown-linux-gnu','features':{'default':config.startswith('default'),'all_features':config.startswith('all'),'no_default_features':config.startswith('no-default')},'origin':origin,'inherited_default':inherited,'automatically_derived':derived,'stability':i.get('stability'),'attrs':i.get('attrs'),'unsafe_callable':unsafe,'safety_preconditions':safety,'intended_behavioral_contract':contract,'coverage':cov,'existing_theorem_evidence':evidence,'existing_proof_domain':scope,'remaining_obligations_or_blocker':blocker,'alias_label':alias_label,'alias_target_ast':alias_target,'raw_rustdoc_item_id':i['id']})
 # Direct free functions and type/constant declarations.
 if not alias_label:
  impl_method_ids={str(x) for i in d['index'].values() if 'impl' in i['inner'] for x in i['inner']['impl']['items']}
  for i in d['index'].values():
   k=list(i['inner'])[0]
   if k=='function' and str(i['id']) not in impl_method_ids and i['visibility']=='public':add(i)
   if (i['visibility']=='public' or (name=='normalize-line-endings' and i['name']=='Normalized')) and k in ['struct','enum','union','trait','type_alias','constant','static']:
    types.append({'rustdoc_id':i['id'],'name':i['name'],'kind':k,'path':'::'.join(d['paths'].get(str(i['id']),{}).get('path',[])),'source_span':i['span'],'definition_ast':i['inner'][k],'configuration':config,'target':'x86_64-unknown-linux-gnu','coverage':'unproved_type_invariant_or_alias_contract','note':'Opaque-return representation is private but inventoried for behavioral reachability.' if name=='normalize-line-endings' and i['name']=='Normalized' else 'Type/constant inventory is separate from callable-body counts.'})
 for implitem in d['index'].values():
  if 'impl' not in implitem['inner']:continue
  imp=implitem['inner']['impl']
  if alias_target is not None:
   # Restrict official dependency docs to impls listed by the alias target type,
   # preserving all referenced borrowed/specialized owner implementations.
   if str(implitem['id']) not in alias_target['impl_ids']:continue
  localspan=implitem.get('span') and implitem['span']['filename'].startswith('src/')
  blanket=imp['blanket_impl'] is not None
  origin='external_blanket' if blanket else 'auto_trait' if imp['is_synthetic'] else 'crate_manual' if localspan and not alias_label else 'external_alias_target' if alias_label else 'external_dependency_impl'
  if localspan and name=='endian-type' and not alias_label:origin='macro_manual' if implitem['span']['begin'][0]>=121 and 'automatically_derived' not in implitem['attrs'] else 'crate_manual'
  if 'automatically_derived' in implitem['attrs']:origin='derived_standard'
  implrows.append({'rustdoc_id':implitem['id'],'owner_type':ty(d,imp['for']),'trait':path(d,imp['trait'])+args(d,imp['trait'].get('args')) if imp['trait'] else None,'impl_ast':imp,'source_span':implitem['span'],'attrs':implitem['attrs'],'origin':origin,'configuration':config,'alias_label':alias_label})
  actualnames=set()
  for id in imp['items']:
   item=d['index'].get(str(id))
   if not item:continue
   if 'function' in item['inner']:
    if imp['trait'] or item['visibility']=='public':add(item,imp,implitem,origin)
    actualnames.add(item['name'])
   else: # Associated constants/types are structural, not callable bodies.
    types.append({'rustdoc_id':item['id'],'name':item['name'],'kind':list(item['inner'])[0],'owner_type':ty(d,imp['for']),'trait':path(d,imp['trait']) if imp['trait'] else None,'definition_ast':item['inner'],'configuration':config,'source_span':item['span'],'origin':origin,'alias_label':alias_label,'coverage':'explicitly_excluded_derived_standard' if origin=='derived_standard' else 'unproved_associated_item'})
  for meth in imp['provided_trait_methods']:
   if meth in actualnames:continue
   found=get_trait_method(d,imp['trait'],meth)
   if found:
    dd,item=found;add(item,imp,implitem,origin if origin=='derived_standard' else 'inherited_default',True,dd)
   else:
    fake={'id':'unresolved:'+meth,'name':meth,'inner':{'function':{'signature_unresolved':True}},'attrs':[],'span':None,'stability':None,'crate_id':None};add(fake,imp,implitem,'inherited_default',True)
 return entries,types,implrows

rows=json.loads((ROOT/'sources.json').read_text());summaries=[]
for name,ver,source,configs in rows:
 source=Path(source); configs=configs+(['default-release','all-release','no-default-release'] if name=='nohash-hasher' else [])
 allentries=[];alltypes=[];allimpls=[]
 for config in configs:
  d=json.loads((ROOT/name/config/'rustdoc.json').read_text())
  if name=='normalize-line-endings':d=json.loads((ROOT/name/'private/target/x86_64-unknown-linux-gnu/doc/normalize_line_endings.json').read_text())
  entries,types,impls=inventory(name,ver,config,d,source)
  if name=='nohash-hasher':
   for alias in ['BuildNoHashHasher']+(['IntMap','IntSet'] if not config.startswith('no-default') else []):
    desired={'BuildNoHashHasher':('core','BuildHasherDefault'),'IntMap':('std','HashMap'),'IntSet':('std','HashSet')}[alias];dd=STD[desired[0]]
    matches=[x for x in dd['index'].values() if x['name']==desired[1] and 'struct' in x['inner']]
    assert len(matches)==1,(alias,len(matches))
    target=matches[0];aliasitem=next(x for x in types if x['name']==alias and x['kind']=='type_alias')
    aliasinfo={'type':aliasitem['definition_ast']['type'],'standard_target':desired,'impl_ids':[str(x) for x in target['inner']['struct']['impls']],'argument_substitution':{'BuildNoHashHasher':'H=NoHashHasher<T>','IntMap':'K=K,V=V,S=BuildHasherDefault<NoHashHasher<K>>','IntSet':'T=T,S=BuildHasherDefault<NoHashHasher<T>>'}[alias]}
    ee,tt,ii=inventory(name,ver,config,dd,source,alias,aliasinfo);entries+=ee;types+=tt;impls+=ii
  for row in entries:row['callable']=True
  allentries+=entries;alltypes+=types;allimpls+=impls
  counts=dict(collections.Counter(x['coverage'] for x in entries));origins=dict(collections.Counter(x['origin'] for x in entries))
  summaries.append({'crate':name,'configuration':config,'callables':len(entries),'callable_with_unsatisfied_additional_bound':sum(x['availability'].startswith('not_callable') for x in entries),'coverage':counts,'origins':origins,'structural_items':len(types),'impl_records':len(impls)})
  (ROOT/name/config/'inventory.json').write_text(json.dumps({'callables':entries,'types_constants_and_associated_items':types,'impl_records':impls},indent=2)+'\n')
 (ROOT/name/'inventory-all-configurations.json').write_text(json.dumps({'crate':name,'version':ver,'callables':allentries,'types_constants_and_associated_items':alltypes,'impl_records':allimpls},indent=2)+'\n')
 default=[x for x in allentries if x['configuration']=='default']
 lines=[f'# {name} {ver}: full public API inventory','', 'Target: x86_64-unknown-linux-gnu; Rust nightly-2026-09-17; rustdoc JSON format 61. Original source/manifest are unchanged. Type arguments, impl generic bounds, method signature AST, raw spans, feature/profile configuration, and dependency origin are retained in JSON. Server catalogue identity is deliberately separate and is not guessed from erased names.','', '| Configuration | Callable entries | Coverage | Structural items |','|---|---:|---|---:|']
 for c in summaries:
  if c['crate']==name:lines.append(f"| {c['configuration']} | {c['callables']} | {json.dumps(c['coverage'])} | {c['structural_items']} |")
 lines+=['','## Default configuration callable ledger','','| Origin | Rust API identity (type parameters preserved) | Coverage |','|---|---|---|']
 for x in default:lines.append('| '+x['origin']+' | `'+x['rust_api_identity'].replace('|','\\|')+'` | '+x['coverage']+' |')
 lines+=['','## Applicability and remaining work','','Only automatically_derived standard-trait implementations are excluded. Manual/macro From, Debug, unsafe representation APIs, trait defaults, external blankets and alias-target APIs remain in the ledger. A passing subset is not full coverage. Default/all-features are collected independently; all-features is not evidence for every cfg combination. No non-x86_64 target has been checked. Stable and nightly/hidden API metadata are retained without silently discarding entries.','', 'Dependency methods are enumerated for crate-defined types and explicitly reexported/type-aliased targets. Their further concrete return types do not recursively expand the entire standard library. Opaque return iterator behavior is included using the original private-items rustdoc supplement. Downstream arbitrary user trait implementations form an open callback boundary, not a finite set of crate-supplied bodies.','']
 if name=='normalize-line-endings':lines+=['Callable ledger entries include six trait methods whose additional Self/Item bounds are unsatisfied for the original opaque type; these are explicitly tagged not callable rather than silently erased. The only exported named callable is normalized. Its returned opaque Iterator exposes the Iterator/blanket operations in this ledger even though Normalized is private. Existing Lean verification uses a char-guard adaptation and arbitrary finite input lists; the original char Switch remains unsupported. Kani checks restricted one-step nonpanic two-reply iterator equality, not arbitrary opaque iterator contextual equivalence. All inherited Iterator combinators remain unproved.','']
 if name=='endian-type':lines+=['Existing six bitwise body contracts preserve arbitrary underlying operator results. Sixty manual integer From implementations and forty-six inherent byte representation/slice methods are not covered. Only from_byte_slice is declared unsafe; safe methods containing unsafe blocks still require a real validity/layout analysis and are not excluded. Formatter methods produced by standard derives are excluded with their provenance.','']
 if name=='nohash-hasher':lines+=['The fourteen existing body contracts cover default std builds with debug assertions both enabled and disabled, not the no-default-feature build merely because source expressions look similar. Manual Debug::fmt is unproved; the pinned Aeneas formatting model ignores emitted text and must be strengthened before a useful formatting claim. Manual Clone::clone_from and four inherited Hasher defaults are not covered. BuildNoHashHasher and std-only IntMap/IntSet alias APIs are explicitly listed as dependency bodies under alias substitutions, not counted as crate-local verified methods.','']
 (ROOT/name/'INVENTORY.md').write_text('\n'.join(lines))
(ROOT/'summary.json').write_text(json.dumps({'toolchain':'nightly-2026-09-17','target':'x86_64-unknown-linux-gnu','configurations':summaries,'standard_docs':{n:{'path':str(STDDIR/(n+'.json')),'sha256':hashlib.sha256((STDDIR/(n+'.json')).read_bytes()).hexdigest()} for n in STD}},indent=2)+'\n')
print(json.dumps(summaries,indent=2))
