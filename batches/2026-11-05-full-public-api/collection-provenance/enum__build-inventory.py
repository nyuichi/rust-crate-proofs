import json,pathlib,hashlib,tomllib,collections,re
ROOT=pathlib.Path('/workspace/full-api/enum')
VERS={'either':'1.19.0','ascii':'1.1.0','base16ct':'1.0.0'}
def digest(o):return hashlib.sha256(json.dumps(o,sort_keys=True).encode()).hexdigest()[:20]
def render(t):
 if t is None:return '()'
 if isinstance(t,str):return t
 if not isinstance(t,dict):return str(t)
 if 'resolved_path' in t:
  p=t['resolved_path'];return p['path']+args(p.get('args'))
 if 'generic' in t:return t['generic']
 if 'primitive' in t:return t['primitive']
 if 'borrowed_ref' in t:
  b=t['borrowed_ref'];return '&'+((b.get('lifetime') or '')+' ' if b.get('lifetime') else '')+('mut ' if b['is_mutable'] else '')+render(b['type'])
 if 'raw_pointer' in t:
  b=t['raw_pointer'];return '*'+('mut ' if b['is_mutable'] else 'const ')+render(b['type'])
 if 'tuple' in t:return '('+', '.join(map(render,t['tuple']))+')'
 if 'slice' in t:return '['+render(t['slice'])+']'
 if 'array' in t:return '['+render(t['array']['type'])+'; '+str(t['array']['len'])+']'
 if 'qualified_path' in t:
  p=t['qualified_path'];return '<'+render(p['self_type'])+' as '+render({'resolved_path':p['trait']})+'>::'+p['name']+args(p.get('args'))
 if 'impl_trait' in t:return 'impl '+json.dumps(t['impl_trait'],sort_keys=True)
 return json.dumps(t,sort_keys=True)
def args(a):
 if not a:return ''
 x=a.get('angle_bracketed')
 if x:
  v=[render(z.get('type',z.get('lifetime',z.get('const',z)))) for z in x['args']]
  return '<'+', '.join(v)+'>' if v else ''
 return json.dumps(a,sort_keys=True)
def sig(i):
 f=i['inner'].get('function')
 if not f:return None
 return {'display':('unsafe ' if f['header']['is_unsafe'] else '')+('const ' if f['header']['is_const'] else '')+'fn '+str(i['name'])+'('+', '.join(n+': '+render(t) for n,t in f['sig']['inputs'])+') -> '+render(f['sig']['output']),'raw':f}
def analyze(name,ver,mode):
 d=json.load(open(ROOT/name/mode/'rustdoc.json'));idx=d['index'];paths=d['paths'];exports=collections.defaultdict(set);visited=set()
 def expose(iid,prefix,alias=None):
  i=idx.get(str(iid))
  if not i:return
  k,x=next(iter(i['inner'].items()))
  nm=alias or i.get('name')
  p=prefix+([nm] if nm else [])
  if k=='use':
   if x['id'] is not None:
    if x['is_glob']:
     target=idx.get(str(x['id']),{});b=target.get('inner',{}).get('module')
     if b:
      for j in b['items']:expose(j,prefix)
     else:expose(x['id'],prefix,x['name'])
    else:expose(x['id'],prefix,x['name'])
   return
  exports[str(iid)].add('::'.join(p))
  if k=='module':
   key=(iid,tuple(p))
   if key in visited:return
   visited.add(key)
   for j in x['items']:expose(j,p)
 expose(d['root'],[],name)
 src=pathlib.Path('/workspace/publish-batch-ten')/name/ver/'aeneas/2026-11-05'
 mappings=tomllib.loads((src/'proofs.toml').read_text())['tool']['proofs'];proofmap={x['api']:x for x in mappings}
 entries=[];seen=set()
 def add(i,identity,category,owner=None,impl=None,public_paths=None,origin='crate manual',extra=None):
  kind=next(iter(i['inner']));sg=sig(i);im=impl or {};tr=im.get('trait');trait=(tr['path']+args(tr.get('args'))) if tr else None
  trpath=paths.get(str(tr['id']),{}).get('path',[]) if tr else []
  der=any('automatically_derived' in str(a) for a in im.get('_attrs',[]))
  span=i.get('span') or im.get('_span')
  if origin=='crate manual' and span and category=='callable':
   sourcefile=src/span['filename']
   if sourcefile.exists():
    text=sourcefile.read_text(); lines=text.splitlines(); at=span['begin'][0]
    macro_ranges=[]; active=None; depth=0
    for lineno,line in enumerate(lines,1):
     cleaned=re.sub(r'"(?:\\.|[^"\\])*"','""',line.split('//')[0])
     if active is None and re.search(r'\bmacro_rules!\s*',cleaned):active=lineno;depth=0
     if active is not None:
      depth+=cleaned.count('{')-cleaned.count('}')
      if depth==0 and lineno>active:macro_ranges.append((active,lineno));active=None
    if any(a<=at<=b for a,b in macro_ranges) or re.match(r'\s*(?:[A-Za-z_]\w*::)*[A-Za-z_]\w*!\s*[({\[]',lines[at-1]):origin='macro manual'
  if der and category!='inherited-default':origin='derived standard' if trpath and trpath[0] in ['core','std','alloc'] else 'derived nonstandard'
  if category=='inherited-default':origin='inherited default'
  elif im.get('is_synthetic'):origin='synthetic auto trait'
  elif im.get('blanket_impl') is not None:origin='crate blanket' if im.get('_span') and (src/im['_span']['filename']).exists() else 'external blanket'
  elif impl and not im.get('_span'):origin='external implementation'
  excluded=origin=='derived standard'
  aliases=sorted(public_paths or [])
  candidate=[]
  for p in aliases:
   p=p.removeprefix(name+'::');candidate.append(p)
  proof=next((proofmap[p] for p in candidate if p in proofmap),None)
  cov='explicitly excluded derived-standard' if excluded else (('proved' if mode=='default' else 'partial-domain') if proof else 'unproved')
  if category!='callable' and category!='inherited-default':cov='structural API; proof obligations pending'
  if origin in ['external blanket','external implementation','synthetic auto trait'] or category=='inherited-default':cov='dependency boundary unproved'
  e={'crate':name,'version':ver,'local_id':digest({'identity':identity,'category':category}),'rust_api_identity':identity,'server_identity_candidates':candidate,'public_paths':aliases,'item_kind':kind,'category':category,'owner_type':owner,'trait':trait,'trait_canonical_path':'::'.join(trpath) if trpath else None,'name':i.get('name'),'signature':sg,'impl_generics':im.get('generics'),'item_generics':i['inner'][kind].get('generics') if isinstance(i['inner'][kind],dict) else None,'raw_item':i['inner'],'source_span':i.get('span'),'defining_crate':d['external_crates'].get(str(i['crate_id']),{}).get('name',name if i['crate_id']==0 else str(i['crate_id'])),'feature_configuration':mode,'features_source':'Cargo.toml; compiler-resolved configuration documented in logs','target':'x86_64-unknown-linux-gnu','origin':origin,'attrs':i.get('attrs',[]),'safety_preconditions':'Unsafe API: inspect documented validity/aliasing/initialization obligations before proving' if sg and sg['raw']['header']['is_unsafe'] else 'Well-typed inputs; callback and trait bounds retained in raw signature','intended_contract':'Existing theorem describes exact behavioral contract' if proof else 'Unproved: establish documented output/state/error/panic/termination behavior over stated generic and safety domain; contract design remains required','existing_proof':proof,'existing_proof_domain':('default features on x86_64; exact published theorem precondition; all/minimal configuration bodies not separately verified' if proof else None),'coverage':cov,'remaining_obligations':[] if proof or excluded else ['Behavioral specification and body proof required; do not treat this inventory as verification'],'obstacle':None if proof or excluded else ('External inherited/dependency body must be separately connected to verified contracts' if origin.startswith('external') or category=='inherited-default' else 'Not yet investigated for complete body proof')}
  if extra:e.update(extra)
  key=(identity,category)
  if key not in seen:entries.append(e);seen.add(key)
 for iid,pp in sorted(exports.items()):
  i=idx[iid];k,x=next(iter(i['inner'].items()));canonical=sorted(pp)[0]
  if k=='module':continue
  if k=='function':add(i,canonical,'callable',public_paths=pp)
  else:add(i,canonical,'structural',public_paths=pp)
  if k in ['struct','enum','union','trait']:
   # Public fields, variants, custom trait members remain visible structural/callable APIs.
   children=[]
   if k=='enum':children=x['variants']
   if k=='struct':
    sk=x.get('kind',{});children=sk.get('plain',{}).get('fields',sk.get('tuple',[])) if isinstance(sk,dict) else []
   if k=='union':children=x.get('fields',[])
   if k=='trait':children=x.get('items',[])
   for cid in children:
    ch=idx.get(str(cid))
    if ch:
     ck=next(iter(ch['inner']));add(ch,canonical+'::'+str(ch['name']),'callable' if ck=='function' else 'structural',owner=canonical,public_paths={p+'::'+str(ch['name']) for p in pp},origin='custom trait default' if ck=='function' and ch['inner'][ck]['has_body'] else 'custom trait required' if ck=='function' else 'crate manual')
     if ck=='variant':
      vk=ch['inner']['variant']['kind'];fields=(vk.get('tuple',[]) or vk.get('struct',{}).get('fields',[])) if isinstance(vk,dict) else []
      for fid in fields:
       field=idx.get(str(fid))
       if field:add(field,canonical+'::'+str(ch['name'])+'::'+str(field['name']),'structural',owner=canonical+'::'+str(ch['name']),public_paths={p+'::'+str(ch['name'])+'::'+str(field['name']) for p in pp})
   for impid in x.get('impls',x.get('implementations',[])):
    it=idx.get(str(impid));im=it and it['inner'].get('impl')
    if not im:continue
    im=dict(im);im['_attrs']=it['attrs'];im['_span']=it['span'];own=render(im['for']);tr=im.get('trait');trait=tr['path']+args(tr.get('args')) if tr else None
    prefix=('<'+own+' as '+trait+'>' if trait else own)
    owneraliases={p for p in pp}
    # An impl is a public relationship even when it supplies no callable methods.
    add(it,prefix+' [impl relationship for '+canonical+']','implementation',owner=own,impl=im,public_paths=[],extra={'attached_public_type':canonical,'raw_impl':{k:v for k,v in im.items() if not k.startswith('_')}})
    for mid in im['items']:
     mi=idx.get(str(mid))
     if not mi:continue
     mk=next(iter(mi['inner']));vis=mi['visibility']
     if not tr and vis!='public':continue
     p={('<'+p+' as '+trait+'>::'+str(mi['name'])) if tr else p+'::'+str(mi['name']) for p in owneraliases}
     # Generic impl instantiations retain full identity, while server candidates erase owner args.
     add(mi,prefix+'::'+str(mi['name']),'callable' if mk=='function' else 'structural',owner=own,impl=im,public_paths=p,extra={'attached_public_type':canonical})
    for meth in im['provided_trait_methods']:
     pseudo={'name':meth,'inner':{'inherited_method':{}},'attrs':[],'span':None,'crate_id':tr and paths.get(str(tr['id']),{}).get('crate_id',0) or 0}
     add(pseudo,prefix+'::'+meth+' [inherited default]','inherited-default',owner=own,impl=im,public_paths={'<'+p+' as '+trait+'>::'+meth for p in owneraliases},origin='inherited default',extra={'attached_public_type':canonical,'signature_resolution':'Rustdoc provides inherited method names but omits external trait signatures; retain trait identity and resolve from dependency source before body proof'})
 return {'crate':name,'version':ver,'configuration':mode,'rustdoc_format_version':d['format_version'],'includes_private':d['includes_private'],'document_hidden_items':True,'entry_count':len(entries),'counts':dict(collections.Counter((e['category']+' | '+e['coverage']) for e in entries)),'entries':entries}
summary=[]
for name,ver in VERS.items():
 configs={mode:analyze(name,ver,mode) for mode in ['default','all','minimal']}
 for mode,c in configs.items():(ROOT/name/f'inventory-{mode}.json').write_text(json.dumps(c,indent=2)+'\n')
 ids={m:{e['rust_api_identity'] for e in c['entries']} for m,c in configs.items()}
 delta={'all_added_over_default':sorted(ids['all']-ids['default']),'default_removed_from_all':sorted(ids['default']-ids['all']),'default_added_over_minimal':sorted(ids['default']-ids['minimal']),'minimal_only':sorted(ids['minimal']-ids['default'])}
 (ROOT/name/'feature-differences.json').write_text(json.dumps(delta,indent=2)+'\n')
 union={}
 for mode,c in configs.items():
  for e in c['entries']:
   if e['local_id'] not in union:union[e['local_id']]={**e,'configurations_observed':[]}
   union[e['local_id']]['configurations_observed'].append(mode)
 result={'crate':name,'version':ver,'configuration_note':'default/all/minimal are three observed configurations, not an exhaustive proof of all feature/target combinations','entries':list(union.values()),'feature_differences':delta}
 (ROOT/name/'coverage-gap.json').write_text(json.dumps(result,indent=2)+'\n')
 counts=collections.Counter(e['category']+' | '+e['coverage'] for e in union.values());origins=collections.Counter(e['origin'] for e in union.values())
 lines=[f'# {name} {ver}: public API coverage inventory','', 'Original-source nightly rustdoc JSON (including hidden public items) and compiler-expanded source were generated for default, all-features and no-default-features on x86_64. These three configurations do not exhaust all feature/platform combinations. Source and manifests are unchanged.','', '## Counts','']
 lines += [f'- {k}: {v}' for k,v in sorted(counts.items())]
 lines+=['','## Origins','']+[f'- {k}: {v}' for k,v in sorted(origins.items())]
 lines+=['','## Feature differences','']+[f'- {k}: {len(v)} distinct identities' for k,v in delta.items()]
 lines+=['','## Every API entry','', '| Rust identity | Category / origin | Configurations | Coverage |','|---|---|---|---|']
 for e in sorted(union.values(),key=lambda e:e['rust_api_identity']):lines.append('| '+e['rust_api_identity'].replace('|','\\|')+' | '+e['category']+' / '+e['origin']+' | '+','.join(e['configurations_observed'])+' | '+e['coverage']+' |')
 lines+=['','## Remaining obligations','', 'Every unproved/manual/nonstandard-derived callable remains required. External blanket implementations, inherited defaults, synthetic auto traits and public custom-trait requirements are retained as dependency/open boundaries, not silently counted as proofs. Derived standard implementations alone are excluded. Structural APIs require representation/invariant review separately from callable-body counts. Raw Rust generic/signature/impl information is preserved in the JSON.','', 'Inherited-default signatures are supplemented by enrich-signatures.py from the exact toolchain/dependency rustdoc JSON; check inherited-signature-resolution.json for completeness. Generic substitutions remain explicit in owner/trait arguments. Stable IDs distinguish specialized impls. Server identities may erase generics and must not merge distinct Rust implementations.','', 'Existing proofs cover only their recorded feature/input domains. A feature-union API match is not evidence that all-feature generated bodies were proved.','']
 (ROOT/name/'coverage-gap.md').write_text('\n'.join(lines))
 summary.append({'crate':name,'version':ver,'counts':dict(counts),'origins':dict(origins),'feature_difference_counts':{k:len(v) for k,v in delta.items()}})
(ROOT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
