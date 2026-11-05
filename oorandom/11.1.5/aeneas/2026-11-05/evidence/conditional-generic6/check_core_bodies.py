import json,hashlib,copy,subprocess
from pathlib import Path
r=Path(__file__).resolve().parent
raw=json.loads((r/'CoreConversions.llbc').read_text());c=raw['translated']
core=Path('/workspace/proof-tools/rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/mod.rs')
sha=lambda b:hashlib.sha256(b).hexdigest()
def name(n):return '::'.join(p['Ident'][0] for p in n if 'Ident'in p)
def inspect(crate, selected_methods=("into","try_into","try_from")):
 refs={};tys={}
 def walk(x):
  if isinstance(x,dict):
   if list(x)==['Value'] and isinstance(x['Value'],list) and len(x['Value'])==2:
    i,v=x['Value']
    if isinstance(v,dict) and set(v)=={'kind','trait_decl_ref'}:refs[i]=v
    if v=='Never':tys[i]=v
    if isinstance(v,dict) and (set(v)=={'TypeVar'} or set(v)=={'Never'} or (set(v)=={'Adt'} and len(v['Adt'])==2)):tys[i]=v
   for v in x.values():walk(v)
  elif isinstance(x,list):
   for v in x:walk(v)
 walk(crate)
 def deref(v,cache):
  return cache[v['Deduplicated']] if 'Deduplicated'in v else v['Value'][1]
 def var(v):return deref(v,tys)['TypeVar']['Free']
 rows=[]
 for method,line,callback,mask in [('into',779,'From',['StorageLive','Call','StorageDead','Return']),('try_into',805,'TryFrom',['StorageLive','Call','StorageDead','Return']),('try_from',821,'Into',['StorageLive','StorageLive','Call','Assign','StorageDead','StorageDead','Return'])]:
  if method not in selected_methods:continue
  fs=[f for f in crate['fun_decls'] if f and f['item_meta']['name'][:2]==[{'Ident':['core',0]},{'Ident':['convert',0]}] and name(f['item_meta']['name']).endswith('::'+method)]
  assert len(fs)==1;f=fs[0];meta=f['item_meta'];assert not meta['is_local'] and meta['opacity']=='Transparent' and meta['attr_info']['rename'] is None
  span=meta['span']['Untagged']['data'];assert span['beg']['line']==line;file=crate['files'][span['file_id']];assert file['name']=={'Local':'/rustc/library/core/src/convert/mod.rs'} and file['crate_name']=='core'
  body=f['body']['Structured'];stmts=body['body']['statements'];assert [k['kind'] if isinstance(k['kind'],str) else next(iter(k['kind'])) for k in stmts]==mask
  call=next(x['kind']['Call']['call'] for x in stmts if isinstance(x['kind'],dict) and 'Call'in x['kind'])
  ref,method_id=call['func']['Regular']['kind']['Trait'];ref=deref(ref,refs);assert ref['kind']=={'Clause':[{'Free':0},{'regions':[]}]} and method_id==0
  clause=f['generics']['trait_clauses'];assert len(clause)==1 and clause[0]['clause_id']==0
  declared=clause[0]['trait_']['skip_binder'];assert ref['trait_decl_ref']==declared
  trait=crate['trait_decls'][declared['id']];assert name(trait['item_meta']['name'])=='core::convert::'+callback
  assert trait['methods'][0]['skip_binder']['name']=={'From':'from','TryFrom':'try_from','Into':'into'}[callback]
  assert [var(x) for x in declared['generics']['types']]==([1,0,2] if method=='try_into' else [1,0])
  assert len(call['args'])==1 and call['args'][0]['Move']['kind']=={'Local':1}
  assert call['dest']['kind']=={'Local':2 if method=='try_from' else 0}
  impl_id=next(p['Impl']['Trait'] for p in meta['name'] if 'Impl'in p);impl=crate['trait_impls'][impl_id]
  assert not impl['is_negative'] and not impl['is_unsafe'];assert name(crate['trait_decls'][impl['impl_trait']['id']]['item_meta']['name'])=='core::convert::'+{'into':'Into','try_into':'TryInto','try_from':'TryFrom'}[method]
  assert [v['name'] for v in f['generics']['types']]==(['T','U','Clause0_Error'] if method=='try_into' else ['T','U'])
  assert var(f['signature']['inputs'][0])==(1 if method=='try_from' else 0)
  if method=='into':assert var(f['signature']['output'])==1
  if method=='try_from':
   assignment=next(s['kind']['Assign'] for s in stmts if isinstance(s['kind'],dict) and 'Assign'in s['kind']);assert assignment[0]['kind']=={'Local':0}
   kind,args=assignment[1]['Aggregate'];adt,variant,union=kind['Adt'];assert variant==0 and union is None
   assert name(crate['type_decls'][adt['id']]['item_meta']['name'])=='core::result::Result';assert crate['type_decls'][adt['id']]['kind']['Enum'][0]['name']=='Ok'
   assert len(args)==1 and args[0]['Move']['kind']=={'Local':2};assert deref(adt['generics']['types'][1],tys)=='Never'
  rows.append({'method':method,'def_id':f['def_id'],'source_file':file,'source_span':span,'source_generics':[x['name'] for x in f['generics']['types']],'actual_trait':name(crate['trait_decls'][impl['impl_trait']['id']]['item_meta']['name']),'actual_dictionary_trait':name(trait['item_meta']['name']),'dictionary_parameter_order':[var(x) for x in declared['generics']['types']],'source_body_has_one_original_dictionary_call':True,'statement_shape':mask,'body_sha256':sha(json.dumps(f['body'],sort_keys=True).encode())})
 return rows
rows=inspect(c)
negative=[]
for case in ['into-wrong-argument','into-extra-instruction','tryinto-wrong-source-line','tryfrom-Err-variant','into-opaque']:
 altered=copy.deepcopy(c);method='try_from' if case=='tryfrom-Err-variant' else ('try_into' if case.startswith('tryinto') else 'into');f=next(f for f in altered['fun_decls'] if f and f['item_meta']['name'][:2]==[{'Ident':['core',0]},{'Ident':['convert',0]}] and name(f['item_meta']['name']).endswith('::'+method));stmts=f['body']['Structured']['body']['statements']
 if case=='into-wrong-argument':next(s['kind']['Call']['call'] for s in stmts if isinstance(s['kind'],dict) and 'Call'in s['kind'])['args'][0]['Move']['kind']['Local']=0
 elif case=='into-extra-instruction':stmts.append(copy.deepcopy(stmts[-1]))
 elif case=='tryinto-wrong-source-line':f['item_meta']['span']['Untagged']['data']['beg']['line']=779
 elif case=='tryfrom-Err-variant':next(s['kind']['Assign'] for s in stmts if isinstance(s['kind'],dict) and 'Assign'in s['kind'])[1]['Aggregate'][0]['Adt'][1]=1
 else:f['body']='Opaque'
 try:inspect(altered)
 except (AssertionError,KeyError,TypeError):negative.append({'case':case,'rejected':True})
 else:raise AssertionError(case+' accepted')
source=core.read_text().splitlines();body_source={n:'\n'.join(source[a-1:z]) for n,a,z in [('Into',769,782),('TryInto',798,808),('TryFrom',814,824)]}
result={'kind':'original core body structural audit; not a new formal semantics or proof of arbitrary callback behavior','rustc':subprocess.check_output(['rustc','-vV'],text=True),'original_source_path':str(core),'original_source_sha256':sha(core.read_bytes()),'source_virtual_prefix_mapping':{'/rustc/library':str(core.parents[3])},'original_source_blocks':body_source,'llbc_sha256':sha((r/'CoreConversions.llbc').read_bytes()),'rows':rows,'negative_guard_checks':negative,'proof_status':{'TryFrom_Rand32_Rand64':'direct original generated body proven, standard3 audit','Into_TryInto_four':'direct original generated bodies proven using separately patched default-off finite export2 translator; original Rust/IR unchanged; full coimport standard3' }}
two=inspect(json.loads((r/'CoreDelegations.llbc').read_text())['translated'], ('into','try_into'))
result['two_body_input_rows']=two
result['two_body_input_sha256']=sha((r/'CoreDelegations.llbc').read_bytes())
(r/'original-core-bodies.json').write_text(json.dumps(result,indent=2)+'\n');print('PASS3 original bodies/5 negative guards')
