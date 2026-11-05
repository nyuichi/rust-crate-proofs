from pathlib import Path
import json,re,hashlib,tarfile,subprocess
p=Path(__file__).resolve().parents[1]
with tarfile.open(p/'evidence/upstream.crate') as t:
 for m in t.getmembers():
  if m.isfile() and (m.name.endswith('.rs') or m.name.endswith('/Cargo.toml') or m.name.endswith('/Cargo.lock')):
   assert (p/Path(m.name).relative_to('rand_xoshiro-0.8.1')).read_bytes()==t.extractfile(m).read(),m.name
rows=[]
for mode in ['default','serde']:
 j=json.loads((p/f'evidence/serializers-{mode}/serializers.llbc').read_text());assert not j['has_errors'];j=j['translated']
 fd=[f for f in j['fun_decls'] if f is not None];ids=[f['def_id'] for f in fd if [x['Ident'][0] for x in f['item_meta']['name'] if 'Ident'in x]==['core','mem','size_of']];assert len(ids)==1
 cache={}
 def types(x):
  if isinstance(x,dict):
   if 'Value'in x and isinstance(x['Value'],list) and len(x['Value'])==2:
    k,v=x['Value']
    if isinstance(v,dict):
     try:
      ty=v['Scalar']['Integer']['Unsigned']
      if isinstance(ty,str):cache[k]=ty
     except (KeyError,TypeError):pass
   for v in x.values():types(v)
  elif isinstance(x,list):
   for v in x:types(v)
 types(j)
 calls=[]
 def walk(x,caller):
  if isinstance(x,dict):
   r=x.get('Regular',{})
   if isinstance(r,dict) and r.get('kind',{}).get('Fun')==ids[0]:
    g=r['generics'];assert not g['trait_refs'] and not g['const_generics'];ts=g['types'];assert len(ts)==1
    a=ts[0];k=a.get('Deduplicated',a.get('Value',[None])[0]);ty=cache[k];assert ty in ['U32','U64'];calls.append({'caller':caller,'type':ty})
   for v in x.values():walk(v,caller)
  elif isinstance(x,list):
   for v in x:walk(v,caller)
 for f in fd:walk(f.get('body'),f['def_id'])
 assert len(calls)==11 and sum(c['type']=='U32'for c in calls)==5 and sum(c['type']=='U64'for c in calls)==6,calls
 generated=p/f'evidence/serializers-{mode}/generated'
 template=(generated/'FunsExternal_Template.lean').read_text();assert re.findall(r'^axiom (.*)$',template,re.M)==['core.mem.size_of (T : Type _) : Result Std.Usize']
 lean='\n'.join(q.read_text()for q in generated.glob('*.lean') if 'Template'not in q.name)
 uses=re.findall(r'core\.mem\.size_of Std\.(\w+)',lean);assert sorted(uses)==sorted(['U32']*5+['U64']*6),uses
 assert all(not re.search(r'\b(?:axiom|sorry|opaque)\b',q.read_text())for q in generated.glob('*.lean')if'Template'not in q.name)
 rows.append({'config':mode,'fun_declarations':len(fd),'size_of_calls':calls,'external_template_only':'core.mem.size_of','supplied_instances':['U32=4','U64=8']})
(p/'evidence/serializer-typed-use-guard.json').write_text(json.dumps(rows,indent=2))
print('Archive Rust/Cargo unchanged; all declarations scanned; only 5 U32 and 6 U64 size_of calls per configuration, sole custom external. Open class supplies exactly two frozen instances; not a universal layout theorem.')
