from pathlib import Path
import json,re,hashlib,collections,shutil,tomllib
OUT=Path(__file__).parent; B=Path('/workspace/full-api/bits'); E=Path('/workspace/full-api/enum/ascii'); SYS=Path('/workspace/proof-tools/rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library')
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def stable(x):return json.dumps(x,sort_keys=True,separators=(',',':'))
def mask(s):
 a=list(s);i=0;n=len(s)
 while i<n:
  j=None
  if s.startswith('//',i):j=s.find('\n',i);j=n if j<0 else j
  elif s.startswith('/*',i):
   k=i+2;dep=1
   while k<n and dep:
    if s.startswith('/*',k):dep+=1;k+=2
    elif s.startswith('*/',k):dep-=1;k+=2
    else:k+=1
   j=k
  else:
   raw=re.compile(r'(?:b|c)?r(#{0,255})"').match(s,i)
   if raw:
    close='"'+raw[1];k=s.find(close,i+len(raw[0]));j=n if k<0 else k+len(close)
   elif s[i]=='"':
    k=i+1
    while k<n:
     if s[k]=='\\':k+=2
     elif s[k]=='"':k+=1;break
     else:k+=1
    j=k
   elif s[i]=="'":
    m=re.compile(r"'(?:\\u\{[0-9a-fA-F_]+\}|\\x[0-9a-fA-F]{2}|\\.|[^'\\\n])'").match(s,i)
    if m:j=i+len(m[0])
  if j is not None:
   for k in range(i,min(j,n)):
    if a[k]!='\n':a[k]=' '
   i=j
  else:i+=1
 return ''.join(a)
def closing(m,at):
 dep=1;j=at+1
 while j<len(m) and dep:
  if m[j]=='{':dep+=1
  elif m[j]=='}':dep-=1
  j+=1
 return j
traitcache={}
def external_method(path,name):
 # Full canonical path matters: similarly named private/bound traits remain distinct.
 k=(path,name)
 if k in traitcache:return traitcache[k]
 seg=path.split('::');trait=seg[-1].split('<')[0];parts=seg[:-1];candidates=[]
 if parts:
  base=SYS/parts[0]/'src';mod=parts[1:];candidates=[base.joinpath(*mod).with_suffix('.rs'),base.joinpath(*mod)/'mod.rs']
 result={'canonical_trait':path,'method':name,'resolved':False,'reason':'Dependency source/signature not resolved; never assumed callable without bounds/gates'}
 for f in candidates:
  if not f.is_file():continue
  s=f.read_text();m=mask(s);t=re.search(r'\btrait\s+'+re.escape(trait)+r'\b',m)
  if not t:continue
  start=m.find('{',t.end());end=closing(m,start);body=m[start+1:end-1];offset=start+1;depth=0;pos=0;previous=offset;methods=[]
  # Scan function keywords only at trait-body depth0.
  while pos<len(body):
   c=body[pos]
   if c=='{':depth+=1;pos+=1;continue
   if c=='}':depth-=1;pos+=1;continue
   hit=re.compile(r'\bfn\s+(\w+)\b').match(body,pos) if depth==0 else None
   if not hit:pos+=1;continue
   fs=offset+pos;fname=hit[1];z=fs+len(hit[0]);par=br=0
   while z<end:
    ch=m[z]
    if ch=='(':par+=1
    elif ch==')':par-=1
    elif ch=='[':br+=1
    elif ch==']':br-=1
    elif ch in '{;' and par==br==0:break
    z+=1
   attrs=re.findall(r'#\[(?:[^\]]|\](?!\s*#))*?\]',s[previous:fs])
   # Keep stable and nightly/const-nightly separate. Attribute text source retained.
   prefix=s[previous:fs];gates=re.findall(r'#\[unstable\([^\]]*\)\]',prefix,re.S);constgates=re.findall(r'#\[rustc_const_unstable\([^\]]*\)\]',prefix,re.S)
   header=s[fs:z].strip();me={'resolved':True,'canonical_trait':path,'method':fname,'source_file':str(f),'source_sha256':digest(f),'begin_line':s.count('\n',0,fs)+1,'signature_with_type_binders_and_extra_bounds':header,'runtime_nightly_gates':gates,'const_only_nightly_gates':constgates,'deprecated_attrs':re.findall(r'#\[deprecated[^\]]*\]',prefix,re.S),'cfg_attrs':re.findall(r'#\[cfg[^\]]*\]',prefix,re.S),'has_default_body':m[z:z+1]=='{','owner_substitution_assumed':False,'module_visibility_and_inherited_cfg_gate_resolution':'not inferred; conditional boundary retained'}
   methods.append(me);method_end=closing(m,z) if m[z:z+1]=='{' else z+1;previous=method_end;pos=method_end-offset
  hits=[v for v in methods if v['method']==name]
  if len(hits)==1:result=hits[0]
  elif hits:result={'resolved':False,'canonical_trait':path,'method':name,'reason':'Ambiguous cfg/macro source declarations','candidates':hits}
  else:result={'resolved':False,'canonical_trait':path,'method':name,'reason':'Trait source located but method may be macro-generated','source_file':str(f),'source_sha256':digest(f)}
  break
 traitcache[k]=result;return result

def afp(x):return stable([x.get('owner_canonical_type'),x.get('trait_canonical_with_arguments'),x.get('impl_generics'),x.get('attached_public_type')])
def attrsderived(a):return 'automatically_derived' in stable(a)
def fnbody(d):return bool(d.get('has_body')) if isinstance(d,dict) else False

def ascii_cfg(cfg):
 inv=E/f'inventory-{cfg}.json';doc=E/cfg/'rustdoc.json';legacy=json.loads(inv.read_text());idx=json.loads(doc.read_text())['index'];entries=legacy['entries'];impls=collections.defaultdict(list)
 for x in entries:
  if x['category']=='implementation':impls[afp(x)].append(x)
 rows=[]
 for x in entries:
  cat=x['category'];kind='structural-data-operation-or-invariant' if cat=='structural' else 'impl-relation' if cat=='implementation' else 'callable';owners=impls.get(afp(x),[]);im=owners[0] if len(owners)==1 else None;rawim=im.get('raw_item',{}).get('impl',{}) if im else {};explicit={idx[str(i)]['name'] for i in rawim.get('items',[]) if str(i) in idx and 'function' in idx[str(i)]['inner']};shadow=cat=='inherited-default' and x['name'] in explicit;derived=attrsderived(im.get('attrs',[])) if im else attrsderived(x.get('attrs',[]));origin=x.get('origin');trait=x.get('trait_canonical_path') or '';std=trait.split('::')[0] in {'core','alloc','std'};excluded=(origin=='derived standard') or (derived and std)
  if cat=='inherited-default':kind='shadowed-provided-default' if shadow else 'excluded-derived-standard-impl-default' if excluded else 'inherited-default-effective-instance'
  elif cat=='callable':
   if excluded:kind='excluded-derived-standard-body'
   elif origin=='custom trait required':kind='trait-required-protocol'
   elif origin=='custom trait default':kind='local-trait-default-template'
   elif origin in ['external blanket','synthetic auto trait']:kind='external-conditional-body'
   else:kind='local-manual-or-macro-body'
  raw=x.get('signature',{}).get('raw') if isinstance(x.get('signature'),dict) else {};raw=raw or {};signature=x.get('signature');external=None
  if cat=='inherited-default' and not shadow and trait:external=external_method(trait,x['name'])
  structural=x.get('item_kind') if cat=='structural' else None
  row={'local_id':x['local_id'],'configuration':cfg,'rust_identity':x['rust_api_identity'],'canonical_rust_identity':x.get('canonical_rust_api_identity'),'server_candidates':x.get('server_identity_candidates',[]),'kind':kind,'legacy_category':cat,'origin':origin,'source_span':x.get('source_span'),'owner_with_type_arguments':x.get('owner_canonical_type'),'owner_raw_type':x.get('owner_raw_type'),'trait_with_arguments':x.get('trait_canonical_with_arguments'),'impl_type_binders_and_bounds':x.get('impl_generics'),'method_type_binders_and_extra_bounds':x.get('item_generics') or raw.get('generics'),'signature':signature,'raw_method_header':raw.get('header'),'default_unstable':raw.get('default_unstable'),'attrs':x.get('attrs',[]),'is_automatically_derived':derived,'derived_nonstandard':bool(derived and not std and trait),'shadowed_by_explicit_impl_member':shadow,'explicit_impl_function_names':sorted(explicit) if cat=='inherited-default' else None,'impl_relationship_matches':len(owners),'resolved_dependency_method':external,'structural_data_operation':structural,'data_definition':x.get('raw_item') if cat=='structural' else None,'legacy_coverage_unmodified':x.get('coverage'),'separate_legacy_body_evidence':{'existing_proof':x.get('existing_proof'),'domain':x.get('existing_proof_domain')} if x.get('existing_proof') else None,'proof_status':'unproved in this strict reconciliation unless explicitly bound to scoped published theorem','bound_satisfaction_not_assumed':origin in ['external blanket','inherited default','synthetic auto trait']}
  rows.append(row)
 return rows,inv,doc

def bitstyle(crate):
 root=B/crate;inv=root/'inventory.json';d=json.loads(inv.read_text());doc=root/'default/rustdoc.json';idx=json.loads(doc.read_text())['index'];records=d['records'];impls={x['key']:x for x in records if x['category'] in ['local_trait_impl','excluded_derive_std_impl','external_blanket_impl','external_synthetic_impl']};rows=[]
 for x in records:
  cat=x['category'];im=impls.get(x.get('impl_key'));imd=im.get('definition',{}) if im else {};explicit={idx[str(i)]['name'] for i in imd.get('items',[]) if str(i) in idx and 'function' in idx[str(i)]['inner']};shadow=cat in ['inherited_local_default_function','inherited_external_default_function'] and x['name'] in explicit;origin=x.get('origin');definition=x.get('definition') or {};trait=x.get('trait') or '';derived=cat.startswith('excluded_derive_std') or attrsderived((im or x).get('attrs',[]));std=trait.split('::')[0] in {'core','alloc','std'};kind='structural-data-operation-or-invariant'
  if cat=='trait_declaration_function':kind='local-trait-default-template' if fnbody(definition) else 'trait-required-protocol'
  elif cat=='inherited_local_default_function':kind='shadowed-provided-default' if shadow else 'local-default-effective-instance'
  elif cat=='inherited_external_default_function':kind='shadowed-provided-default' if shadow else 'excluded-derived-standard-impl-default' if derived and std else 'inherited-default-effective-instance'
  elif cat in ['local_trait_impl_member_function','inherent_impl_member_function']:kind='local-manual-or-macro-body'
  elif cat=='excluded_derive_std_impl_member_function':kind='excluded-derived-standard-body'
  elif cat=='external_blanket_impl_member_function':kind='external-conditional-body'
  elif cat.endswith('impl'):kind='impl-relation'
  ext=external_method(trait,x['name']) if cat=='inherited_external_default_function' and not shadow and trait else None
  rows.append({'local_id':x['local_id'],'configuration':'default','observed_configs':x.get('feature_configs'),'rust_identity':x['rust_api_identity'],'canonical_rust_identity':x['rust_api_identity'],'server_candidates':[x.get('server_identity') or x['rust_api_identity']],'kind':kind,'legacy_category':cat,'origin':origin,'source_span':x.get('span'),'owner_with_type_arguments':x.get('receiver'),'trait_with_arguments':trait,'impl_type_binders_and_bounds':imd.get('generics'),'method_type_binders_and_extra_bounds':definition.get('generics'),'signature':definition.get('sig'),'raw_method_header':definition.get('header'),'default_unstable':definition.get('default_unstable'),'attrs':x.get('attrs'),'is_automatically_derived':derived,'derived_nonstandard':bool(derived and not std and trait),'shadowed_by_explicit_impl_member':shadow,'explicit_impl_function_names':sorted(explicit) if 'inherited_' in cat else None,'resolved_dependency_method':ext,'structural_data_operation':cat if kind=='structural-data-operation-or-invariant' else None,'data_definition':definition if kind=='structural-data-operation-or-invariant' else None,'legacy_coverage_unmodified':x.get('coverage'),'proof_status':'unproved in this strict reconciliation unless explicitly bound to scoped published theorem','bound_satisfaction_not_assumed':'external' in cat})
 return rows,inv,doc

SPECS={'ascii':(Path('/workspace/full-api/bits/ascii-extra/candidate-error-projections'),37),'bit_field':(B/'bit_field/candidate-supported-layout-v3',27),'byteorder':(B/'byteorder/candidate-read128',42),'byteorder-lite':(B/'byteorder-lite/candidate-signed128',64)}
allsummary={};manifest=[]
for crate,(artifact,expected) in SPECS.items():
 out=OUT/crate;out.mkdir(exist_ok=True);legacydir=OUT/'legacy'/crate;legacydir.mkdir(parents=True,exist_ok=True);configs=['default','all','minimal','alloc_only'] if crate=='ascii' else ['default'];allrows=[];config_summary={}
 for cfg in configs:
  rows,inv,doc=ascii_cfg(cfg) if crate=='ascii' else bitstyle(crate);allrows+=rows;shutil.copy2(inv,legacydir/inv.name);manifest.append({'path':str(inv),'snapshot':str(legacydir/inv.name),'sha256':digest(inv),'rustdoc_json':str(doc),'rustdoc_sha256':digest(doc)})
  config_summary[cfg]={'records':len(rows),'by_kind':dict(collections.Counter(r['kind'] for r in rows)),'derived_nonstandard_records':sum(r['derived_nonstandard'] for r in rows),'effective_inherited_with_std_source_signature':sum(bool(r['kind']=='inherited-default-effective-instance' and r['resolved_dependency_method'] and r['resolved_dependency_method']['resolved']) for r in rows),'effective_inherited_dependency_signature_unresolved':sum(bool(r['kind']=='inherited-default-effective-instance' and r['resolved_dependency_method'] and not r['resolved_dependency_method']['resolved']) for r in rows),'runtime_nightly_gated_dependency_methods':sum(bool(r['kind']=='inherited-default-effective-instance' and r['resolved_dependency_method'] and r['resolved_dependency_method'].get('runtime_nightly_gates')) for r in rows)}
 defaults=[r for r in allrows if r['configuration']=='default'];maps=tomllib.loads((artifact/'proofs.toml').read_text())['tool']['proofs'];assert len(maps)==expected;bindings=[]
 for m in maps:
  candidates=[r for r in defaults if m['api'] in r['server_candidates'] or m['api']==r['canonical_rust_identity'] or m['api']==r['rust_identity']];eff=[r for r in candidates if r['kind']!='shadowed-provided-default' and r['kind'] not in ['impl-relation','structural-data-operation-or-invariant']]
  assert len(eff)==1,(crate,m['api'],[(r['rust_identity'],r['kind']) for r in candidates]);r=eff[0];assert r['kind'] not in ['excluded-derived-standard-body','excluded-derived-standard-impl-default'],(crate,m['api']);r['proof_status']='scoped published body contract (configuration/domain preserved)';r['published_mapping']=m;bindings.append({'api_key_unchanged':m['api'],'effective_source_matches':1,'raw_legacy_matches':len(candidates),'shadowed_legacy_matches':len(candidates)-len(eff),'local_id':r['local_id'],'source_identity':r['canonical_rust_identity'],'source_span':r['source_span'],'origin':r['origin'],'kind':r['kind'],'type_binders':r['impl_type_binders_and_bounds'],'method_binders_and_bounds':r['method_type_binders_and_extra_bounds'],'mapping':m})
 noncallable_invariants=[]
 if crate=='bit_field':
  for rr in defaults:
   if rr['legacy_category']=='local_trait_impl_member_assoc_const':
    ty=rr['owner_with_type_arguments'];rr['noncallable_invariant_evidence']={'theorem':'Proofs.'+ty+'_length_spec','scope':'twelve guarded primitive layout dictionaries; x86_64 pointer width; no arbitrary Rust type layout guarantee'};noncallable_invariants.append(rr['noncallable_invariant_evidence'])
 unpublished=[]
 if crate=='ascii':
  packet=json.loads((B/'ascii-extra/typed-vector-bindings/binding-packet.json').read_text())
  for v in packet['rows']:
   matches=[r for r in defaults if r['rust_identity']==v['inventory_rust_identity'] and r['kind']=='local-manual-or-macro-body'];assert len(matches)==1;matches[0]['proof_status']='frozen41 typed proof; UNPUBLISHED pending exact source/remote binding';unpublished.append({'source_identity':v['canonical_rust_identity'],'typed_api_key':v['desired_fully_typed_server_key'],'source_matches':1,'publication_gate':'held; not part of published37','current_exact_typed_catalogue_rows':len(v['current_exact_typed_server_rows']),'local_id':matches[0]['local_id']})
 summary={'crate':crate,'published_scoped_api_count':expected,'published_source_unique':len(bindings),'published_api_keys_changed':0,'published_proof_artifact':str(artifact),'published_proofs_toml_sha256':digest(artifact/'proofs.toml'),'not_full_api_closure':True,'per_configuration':config_summary,'unpublished_frozen_typed_proofs_separate':unpublished,'known_noncallable_invariant_evidence_separate':noncallable_invariants,'local_instances_without_published_Lean_or_held41_mapping_default':sum(r['kind'] in ['local-manual-or-macro-body','local-default-effective-instance'] and not r['proof_status'].startswith(('scoped published','frozen41')) for r in defaults),'additional_local_Kani_body_evidence_beyond_published_Lean_default':sum(r['legacy_coverage_unmodified']=='proved-kani-body-local-unpublished' and not r['proof_status'].startswith(('scoped published','frozen41')) for r in defaults),'template_protocol_obligations_separate':sum(r['kind'] in ['local-trait-default-template','trait-required-protocol'] for r in defaults),'structural_data_records_separate':sum(r['kind']=='structural-data-operation-or-invariant' for r in defaults),'shadowed_provided_defaults_default':sum(r['kind']=='shadowed-provided-default' for r in defaults)}
 (out/'records.json').write_text(json.dumps({'scope':'Strict reconciliation; legacy snapshots unchanged; proof coverage comes only from explicit scoped published mappings','records':allrows},indent=2)+'\n');(out/'published-source-bindings.json').write_text(json.dumps(bindings,indent=2)+'\n');(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');allsummary[crate]=summary
(OUT/'legacy-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(OUT/'summary.json').write_text(json.dumps(allsummary,indent=2)+'\n');(OUT/'dependency-signature-resolution.json').write_text(json.dumps([v for v in traitcache.values()],indent=2)+'\n');print(json.dumps({k:{'published_unique':v['published_source_unique'],'shadow_default':v['shadowed_provided_defaults_default'],'unproved_local':v['local_instances_without_published_Lean_or_held41_mapping_default'],'std_signature_resolved':v['per_configuration']['default']['effective_inherited_with_std_source_signature'],'std_signature_unresolved':v['per_configuration']['default']['effective_inherited_dependency_signature_unresolved']} for k,v in allsummary.items()},indent=2))
