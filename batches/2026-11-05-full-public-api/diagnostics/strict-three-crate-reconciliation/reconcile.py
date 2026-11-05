from pathlib import Path
import json,copy,collections,hashlib,tomllib,re
ROOT=Path(__file__).resolve().parents[1];OUT=Path(__file__).resolve().parent
CONFIGS={'endian-type':['default','all'],'nohash-hasher':['default','all','no-default','default-release','all-release','no-default-release'],'normalize-line-endings':['default','all']}
NAMES={'HashMap':'std::collections::hash::map::HashMap','HashSet':'std::collections::hash::set::HashSet','RandomState':'std::hash::random::RandomState','BuildHasherDefault':'core::hash::BuildHasherDefault','BuildNoHashHasher':'nohash_hasher::BuildNoHashHasher','NoHashHasher':'nohash_hasher::NoHashHasher','Normalized':'normalize_line_endings::Normalized','Global':'alloc::alloc::Global'}
def render(t):
 if not isinstance(t,dict):return str(t)
 if 'reconciled_display' in t:return t['reconciled_display']
 if 'generic' in t:return t['generic']
 if 'primitive' in t:return t['primitive']
 if 'resolved_path' in t:
  p=t['resolved_path'];name=NAMES.get(p['path'].split('::')[-1],p['path']);return name+args(p.get('args'))
 if 'tuple' in t:return '('+', '.join(render(v) for v in t['tuple'])+')'
 if 'array' in t:return '['+render(t['array']['type'])+'; '+t['array']['len']+']'
 if 'slice' in t:return '['+render(t['slice'])+']'
 if 'borrowed_ref' in t:
  v=t['borrowed_ref'];return '&'+((v['lifetime']+' ') if v.get('lifetime') else '')+('mut ' if v['is_mutable'] else '')+render(v['type'])
 if 'raw_pointer' in t:return '*'+('mut ' if t['raw_pointer']['is_mutable'] else 'const ')+render(t['raw_pointer']['type'])
 if 'qualified_path' in t:
  p=t['qualified_path'];return '<'+render(p['self_type'])+' as '+(p.get('trait') or {}).get('path','?')+'>::'+p['name']+args(p.get('args'))
 return json.dumps(t,sort_keys=True)
def args(a):
 if not a:return ''
 if 'angle_bracketed' not in a:return json.dumps(a,sort_keys=True)
 q=a['angle_bracketed'];v=[render(x['type']) if 'type' in x else x['lifetime'] if 'lifetime' in x else json.dumps(x,sort_keys=True) for x in q['args']]
 return '<'+', '.join(v)+'>' if v else ''
def subst(x,b):
 if isinstance(x,dict):
  if set(x)=={'generic'} and x['generic'] in b:return copy.deepcopy(b[x['generic']]) # one substitution, not capture of inner target parameters
  return {k:subst(v,b) for k,v in x.items()}
 if isinstance(x,list):return [subst(v,b) for v in x]
 return x
def owner_display(a):
 s=a['rust_api_identity'];return s[1:].split(' as ',1)[0] if s.startswith('<') else s.rsplit('::',1)[0]
def find_impl(a,imps):
 z=[i for i in imps if i['owner_type']==a['owner_type'] and i['trait']==a['trait'] and i.get('alias_label')==a.get('alias_label') and i.get('source_span')==a.get('impl_source_span')]
 if not a['inherited_default']:z=[i for i in z if str(a['raw_rustdoc_item_id']) in {str(x) for x in i['impl_ast']['items']}]
 else:z=[i for i in z if a['name'] in i['impl_ast']['provided_trait_methods']]
 assert len(z)<=1,(a['local_id'],len(z));return z[0] if z else None
oldcov=json.loads((ROOT/'closure-gaps/coverage-reconciliation.json').read_text());cov={(a['crate'],a['configuration'],a['local_id']):a for a in oldcov['rows']}
rows=[];summaries=[];shadow=[];inputs={};excluded=[];fixed=[]
for crate,configs in CONFIGS.items():
 for config in configs:
  path=ROOT/crate/config/'inventory.json';raw=json.loads(path.read_text());inputs[str(path.relative_to(ROOT))]=hashlib.sha256(path.read_bytes()).hexdigest();imps=raw['impl_records'];counts=collections.Counter()
  for a in raw['callables']:
   imp=find_impl(a,imps);b={};alias=a.get('alias_label');own=owner_display(a)
   alias_binding={}
   if alias:
    typ='K' if alias=='IntMap' else 'T'
    if alias=='BuildNoHashHasher':alias_binding['H']={'reconciled_display':'nohash_hasher::NoHashHasher<T>'}
    else:alias_binding.update(S={'reconciled_display':f'nohash_hasher::BuildNoHashHasher<{typ}>'},A={'reconciled_display':'alloc::alloc::Global (alias default allocator)'})
    def owner_mentions(v,name):
     if isinstance(v,dict):return v.get('generic')==name or any(owner_mentions(q,name) for q in v.values())
     if isinstance(v,list):return any(owner_mentions(q,name) for q in v)
     return False
    alias_binding={k:v for k,v in alias_binding.items() if owner_mentions(a['owner_type_ast'],k)}
    b.update(alias_binding)
   pattern=imp and imp['impl_ast'].get('blanket_impl')
   if isinstance(pattern,dict) and 'generic' in pattern:b[pattern['generic']]={'reconciled_display':own}
   tr=subst(a.get('trait_ast'),b);newtrait=(a['trait'].split('<',1)[0]+args(tr.get('args'))) if tr else None
   corrected=f'<{own} as {newtrait}>::{a["name"]}' if tr else a['rust_api_identity']
   stdderived=a['automatically_derived'] and bool(a['trait'] and a['trait'].startswith(('core::','alloc::','std::')))
   nonstdderived=a['automatically_derived'] and not stdderived
   mismatch=alias in ['IntMap','IntSet'] and 'RandomState' in a['owner_type']
   badbound=a['availability'].startswith('not_callable')
   gated=(a.get('stability')or{}).get('level')=='unstable'
   classification='excluded_derived_standard' if stdderived else 'inapplicable_specialized_alias_hasher' if mismatch else 'not_callable_unsatisfied_additional_bound' if badbound else 'requires_consumer_nightly_feature_gate' if gated else 'signature_applicable_under_stated_type_bounds'
   counts[classification]+=1
   status=(cov.get((crate,config,a['local_id']))or{}).get('reconciled_status','unproved')
   evidence=[]
   if crate=='normalize-line-endings':
    status='unproved'
    if a['name']=='normalized' and a['origin']=='crate_manual':status='existing_adapted_machine1_plus_original_constructor_init_Lean';evidence=['normalize-original-importable','normalize-original']
    elif a['name']=='next' and a['origin']=='crate_manual':status='original_Kani_finite0..4_plus_separate_adapted_pipeline';evidence=['normalize-original']
    elif a['inherited_default'] and a['name'] in ['nth','count','last']:status='original_Kani_finite0..4_actual_std_default';evidence=['normalize-iterator-defaults-kani']
    elif a['inherited_default'] and a['name'] in ['position','find']:status='original_Kani_finite0..4_pure_character_equality';evidence=['normalize-iterator-search-kani']
    elif a['inherited_default'] and a['name']=='size_hint':status='original_Kani_finite0..4_initial_state_only';evidence=['normalize-original']
   row={'crate':crate,'configuration':config,'original_local_id':a['local_id'],'original_inventory':str(path.relative_to(ROOT)),'raw_row':a,'raw_impl_record':imp,'corrected_rust_identity_display':corrected,'binder_substitution':b,'binder_substitution_reason':'blanket_impl template generic binds concrete Self, not the wrapper payload parameter; alias fixes hasher/allocator' if b else None,'instantiated_trait_ast':tr,'instantiated_impl_bounds':{'remaining_template_params':[q for q in imp['impl_ast']['generics']['params'] if q['name'] not in b],'where_predicates':subst(imp['impl_ast']['generics']['where_predicates'],b),'original_template_params':imp['impl_ast']['generics']['params']} if imp else None,'instantiated_signature_ast':subst(a['signature_ast'],b),'alias_definition':a.get('alias_target_ast'),'strict_classification':classification,'std_feature_gate':(a.get('stability')or{}).get('feature') if gated else None,'nonstandard_derived':nonstdderived,'evidence_status':status,'evidence_directories':evidence,'not_a_new_machine_claim':True}
   if mismatch:row['reason']='Original impl requires RandomState; source alias fixes BuildHasherDefault<NoHashHasher<key>>. Enabling allocator_api does not change this type mismatch.';excluded.append(row)
   if corrected!=a['rust_api_identity']:fixed.append({'crate':crate,'configuration':config,'local_id':a['local_id'],'old':a['rust_api_identity'],'corrected':corrected,'substitution':b,'raw_impl_pattern':pattern})
   rows.append(row)
  for imp in imps:
   ids={str(v) for v in imp['impl_ast']['items']};actual=[a for a in raw['callables'] if a.get('alias_label')==imp.get('alias_label') and a['owner_type']==imp['owner_type'] and a['trait']==imp['trait'] and not a['inherited_default'] and str(a['raw_rustdoc_item_id']) in ids]
   names={a['name'] for a in actual};both=names&set(imp['impl_ast']['provided_trait_methods'])
   if both:
    inherited=[a for a in raw['callables'] if a.get('alias_label')==imp.get('alias_label') and a['owner_type']==imp['owner_type'] and a['trait']==imp['trait'] and a['inherited_default'] and a['name'] in both];assert not inherited
    shadow.append({'crate':crate,'configuration':config,'raw_impl':imp,'provided_trait_methods_shadowed_by_actual_items':sorted(both),'already_omitted_by_original_generator':True,'do_not_subtract_from_existing_ledger_again':True,'actual_implementation_rows':[a for a in actual if a['name'] in both]})
  summaries.append({'crate':crate,'configuration':config,'ledger_entries_preserved':len(raw['callables']),'strict_counts':dict(counts),'applicable_signature_candidates_including_feature_gated':counts['signature_applicable_under_stated_type_bounds']+counts['requires_consumer_nightly_feature_gate'],'type_bound_satisfaction_for_arbitrary_T':'not asserted; these are conditional impl/signature candidates','structural_counts':dict(collections.Counter(x['kind'] for x in raw['types_constants_and_associated_items']))})
# Predicate-bearing candidates remain conditional; do not advertise all-T applicability.
def has_constraints(v):
 if isinstance(v,dict):
  if 'trait_bound' in v and v['trait_bound'].get('modifier')!='maybe':return True
  if 'outlives' in v:return True
  return any(has_constraints(q) for q in v.values())
 if isinstance(v,list):return any(has_constraints(q) for q in v)
 return False
for a in rows:
 a['retains_explicit_type_or_callback_conditions']=has_constraints(a['raw_row']['impl_generics']) or has_constraints(a['raw_row']['signature_ast'].get('generics',{}))
 a['bounds_resolution_status']='No universal arbitrary-type satisfaction claim; raw and instantiated constraints retained. Only explicitly known false additional bounds are separated as not-callable.'
for a in summaries:
 a['candidate_rows_with_explicit_type_or_callback_conditions']=sum(q['retains_explicit_type_or_callback_conditions'] for q in rows if q['crate']==a['crate'] and q['configuration']==a['configuration'] and q['strict_classification'] in ['signature_applicable_under_stated_type_bounds','requires_consumer_nightly_feature_gate'])
OUT.mkdir(exist_ok=True)
for file,data in [('strict-rows.json',rows),('summary.json',summaries),('inapplicable-alias-exact-rows.json',excluded),('blanket-identity-corrections.json',fixed),('shadowed-provided-methods.json',shadow),('input-inventory-hashes.json',inputs)]:
 (OUT/file).write_text(json.dumps(data,indent=2)+'\n')
print(json.dumps(summaries,indent=2))
