from pathlib import Path
import json,hashlib,collections,re,copy,tomllib
ROOT=Path('/workspace/full-api/enum/either');OUT=ROOT/'strict-reconciliation'
PROOF=ROOT/'iterator40-integrated/either/1.19.0/aeneas/2026-11-05'
P=tomllib.load(open(PROOF/'proofs.toml','rb'))['tool']['proofs'];assert len(P)==40
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def params(g):return [p['name'] for p in (g or {}).get('params',[])]
def scoped(scope,name):return {('bound_lifetime'if name.startswith("'")else'bound_generic'):{'scope':scope,'name':name}}
def transform(x,env,depth=0):
 if isinstance(x,list):return [transform(v,env,depth)for v in x]
 if isinstance(x,str):return copy.deepcopy(env.get(x,x))if x.startswith("'")else x
 if not isinstance(x,dict):return x
 if set(x)=={'generic'}:return copy.deepcopy(env.get(x['generic'],{'unresolved_generic':x['generic']}))
 inner=dict(env)
 for p in x.get('generic_params',[]):inner[p['name']]=scoped('hrtb:'+str(depth),p['name'])
 return {k:transform(v,inner,depth+1)for k,v in x.items()}
def rawimpl(e,raw):
 if e.get('owner_raw_type')is None:return None,[]
 out=[]
 for iid,item in raw['index'].items():
  im=item['inner'].get('impl')
  if im and im['for']==e['owner_raw_type']and im.get('trait')==e.get('trait_raw_path')and im.get('generics')==e.get('impl_generics'):out.append((iid,im))
 return (out[0][1]if len(out)==1 else None),[x[0]for x in out]
def bind(e,raw,rawhash):
 im,ids=rawimpl(e,raw);owner=e.get('owner_canonical_type');selfnode={'applied_self':{'rust_type':owner,'owner_document_sha256':rawhash,'scope':'owner-instance'}}if owner else scoped('trait','Self')
 env={'Self':selfnode};bindings={}
 for name in params(e.get('impl_generics')):env[name]=scoped('impl',name)
 pattern=(im or {}).get('blanket_impl')
 if isinstance(pattern,dict)and set(pattern)=={'generic'}:
  # One-pass substitution. Never recursively substitute R inside an owner
  # Either<L,R> when source blanket binder R is replaced by that owner.
  env[pattern['generic']]=selfnode;bindings[pattern['generic']]=selfnode
 elif im:
  for name in params(e.get('impl_generics')):
   if owner and re.search(r'(?<!\w)'+re.escape(name)+r'(?!\w)',owner):env[name]=scoped('owner-free',name)
 contract=e.get('trait_method_contract')or(e.get('signature')if e.get('category')=='inherited-default'else None)
 method=(contract or {}).get('raw')if e.get('category')=='inherited-default'else(e.get('signature')or{}).get('raw')
 traitbindings={};tenv={'Self':selfnode}
 if contract:
  args=((e.get('trait_raw_path')or{}).get('args')or{}).get('angle_bracketed',{}).get('args',[])
  for par,arg in zip((contract.get('trait_generics')or{}).get('params',[]),args):
   value=transform(arg,env);value={'applied_trait_argument':{'value':value,'source_document_sha256':rawhash}};tenv[par['name']]=value;traitbindings[par['name']]=value
 menv=tenv if e.get('category')=='inherited-default'else dict(env)
 for name in params((method or{}).get('generics')):menv[name]=scoped('method',name)
 predicates=transform((e.get('impl_generics')or{}).get('where_predicates',[]),env)
 return {'ir_note':'Scoped diagnostic IR, not emitted Rust or a trait-solver proof. Applied owner/arguments retain source-document contexts; projections and residual predicates are not silently solved.','matching_impl_ids_document_local':ids,'self_type':selfnode,'blanket_substitution':bindings,'impl_scopes':{k:v for k,v in env.items()if k!='Self'},'trait_argument_substitution':traitbindings,'method_scopes':{n:menv[n]for n in params((method or{}).get('generics'))},'impl_parameter_declarations':e.get('impl_generics'),'implicit_sized_note':'Rust default Sized and explicit ?Sized remain obligations in these declarations; no blanket predicate is discarded.', 'impl_predicates_after_substitution':predicates,'instantiated_method':transform(method,menv)if method else None,'method_signature_document':(contract or{}).get('provenance_document'),'method_signature_document_sha256':(contract or{}).get('document_sha256'),'applicability':'Conditional on all impl/type/method/Sized/lifetime/associated-type predicates; unresolved is NOT not-callable. No arbitrary L/R trait fulfillment is assumed.','associated_projection_resolution':'Retained symbolically with actual applied Self and trait context; not substituted by an unrelated owner or erased Item.'}
def keys(e):
 k=set(e.get('server_identity_candidates',[]))|set(e.get('public_paths',[]))
 if e.get('trait_raw_path'):
  full=e['trait_canonical_with_arguments'];base=e['trait_canonical_path'];short=e['trait_raw_path']['path'];k.add('<'+e['owner_canonical_type']+' as '+short+full[len(base):]+'>::'+str(e['name']))
 return k
ALL=[];summary={};proof_matches=[];shadowall=[];ctorall=[];serdeall=[];inputs={}
for mode in ['default','all','minimal','serde_minimal']:
 ip=ROOT/('inventory-'+mode+'.json');rp=ROOT/mode/'rustdoc.json';inv=json.load(open(ip));raw=json.load(open(rp));rh=sha(rp);inputs[mode]={'inventory_sha256':sha(ip),'rawrustdoc_sha256':rh,'rows':len(inv['entries'])}
 E=inv['entries'];callables=collections.defaultdict(list)
 for e in E:
  if e['category']=='callable':callables[e['canonical_rust_api_identity']].append(e)
 matched={}
 if mode=='default':
  for p in P:
   f=[e for e in E if p['api']in keys(e)];assert len(f)==1,(p['api'],len(f));matched[f[0]['local_id']]=p;proof_matches.append({'proof':p,'inventory_id':f[0]['local_id'],'full_rust_identity':f[0]['canonical_rust_api_identity'],'origin':f[0]['origin'],'category':f[0]['category']})
 for e in E:
  cat=e['category'];rid=e['canonical_rust_api_identity'];state='structural-or-implementation-obligation-retained';over=[]
  if cat=='callable':state='unproved-callable-conditional'
  if cat=='inherited-default':
   over=callables.get(rid,[]);state='shadowed-default-declaration-retained'if over else'active-default-candidate-conditional-unproved'
   if over:shadowall.append({'configuration':mode,'full_rust_identity':rid,'server_candidates':e['server_identity_candidates'],'shadow_entry':e['local_id'],'actual_override_entries':[x['local_id']for x in over],'override_origins':[x['origin']for x in over],'actual_sources':[x['source_span']for x in over],'reason':'Compiler provided_trait_methods includes explicitly overridden names. Concrete dispatch uses the actual override; keep default declaration evidence, not a second callable of this identity.'})
  if cat=='callable'and e['origin']=='derived standard':state='excluded-actual-derived-standard-body-only'
  if mode=='default'and e['local_id']in matched:state='mapped40-model-contract-with-explicit-domain'
  if mode=='default'and e['trait_canonical_path']=='either::into_either::IntoEither'and cat=='inherited-default'and not over and e['local_id']not in matched:
   state='generic-actual-blanket-specialization-covered-not-extra-machine-claim'
  if mode=='default'and e['origin']=='custom trait default'and cat=='callable':state='original-default-body-helper-covered-conditional-not-extra-machine-claim'
  if e['item_kind']=='variant':
   state='data-constructor-operation-pending-explicit-binding';ctorall.append({'configuration':mode,'inventory_id':e['local_id'],'public_identity':rid,'source_span':e['source_span'],'operation':'construct variant and retain exact payload under valid Rust values; symbolic tag, not physical layout/ABI','proof_boundary':'Generated datatype and source are used by existing proofs; no separate constructor callable binding/machine claim exists.'})
  if e['origin']=='derived nonstandard'or('serde'in str(e.get('trait_canonical_path')))or(cat=='callable'and'::serde_untagged'in rid):
   serdeall.append({'configuration':mode,'id':e['local_id'],'identity':rid,'category':cat,'origin':e['origin'],'status':state,'excluded':False})
  row={'configuration':mode,'original_id':e['local_id'],'rust_identity':rid,'server_candidates':e['server_identity_candidates'],'category':cat,'origin':e['origin'],'status':state,'original_coverage_label':e['coverage'],'raw_source':e['source_span'],'proof':matched.get(e['local_id']),'no_api_or_inventory_row_deleted':True}
  if cat in['callable','inherited-default']:row['scoped_binding']=bind(e,raw,rh)
  ALL.append(row)
 C=collections.Counter(x['status']for x in ALL if x['configuration']==mode);summary[mode]={'original_rows':len(E),'row_classifications':dict(C),'shadowed_default_rows':sum(1 for x in shadowall if x['configuration']==mode),'nonshadow_inherited_candidates':sum(1 for x in ALL if x['configuration']==mode and x['category']=='inherited-default'and x['status']!='shadowed-default-declaration-retained'),'not_a_concrete_unconditional_API_count':True}
 # Two actual variant items each have a root-level and enum-qualified alias.
 variants=[(iid,v)for iid,v in raw['index'].items()if'variant'in v['inner']and v['name']in['Left','Right']];assert len(variants)==2
 summary[mode]['variant_items']=[{'document_local_id':i,'name':v['name'],'kind':v['inner']['variant']['kind'],'source':v['span']}for i,v in variants];summary[mode]['constructor_operations']=2;summary[mode]['constructor_alias_rows']=sum(1 for x in ctorall if x['configuration']==mode)
 assert summary[mode]['constructor_alias_rows']==4
assert len(proof_matches)==40
OUT.mkdir(exist_ok=True)
for name,value in {'rows.json':ALL,'counts.json':summary,'proof40-bindings.json':proof_matches,'shadowed-defaults.json':shadowall,'constructors.json':ctorall,'serde-obligations.json':serdeall,'inputs.json':inputs}.items():(OUT/name).write_text(json.dumps(value,indent=2)+'\n')
print(json.dumps(summary,indent=2));print('All original rows retained:',len(ALL),'mapped40',len(proof_matches))
