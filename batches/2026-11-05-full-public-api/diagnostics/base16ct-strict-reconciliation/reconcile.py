from pathlib import Path
import json,collections,copy,hashlib,tomllib
ROOT=Path(__file__).resolve().parent.parent;OUT=Path(__file__).resolve().parent
corepath=Path('/workspace/proof-tools/rustup/toolchains/nightly-2026-09-17-x86_64-unknown-linux-gnu/share/doc/rust/json/core.json')
core=json.loads(corepath.read_text());traitmeta={}
for id,p in core['paths'].items():
 i=core['index'].get(id)
 if i and 'trait' in i['inner']:traitmeta['::'.join(p['path'])]={'stability':i.get('stability'),'visibility':i['visibility'],'attrs':i['attrs'],'methods':{core['index'][str(mid)]['name']:{'stability':core['index'][str(mid)].get('stability'),'attrs':core['index'][str(mid)]['attrs']} for mid in i['inner']['trait']['items'] if str(mid) in core['index'] and 'function' in core['index'][str(mid)]['inner']}}
del core
rows=[];shadows=[];summary=[];inputs={}
for config in ['default','all','minimal']:
 p=ROOT/f'inventory-{config}.json';inv=json.loads(p.read_text());d=json.loads((ROOT/config/'rustdoc.json').read_text());inputs[str(p)]=hashlib.sha256(p.read_bytes()).hexdigest();inputs[str(ROOT/config/'rustdoc.json')]=hashlib.sha256((ROOT/config/'rustdoc.json').read_bytes()).hexdigest()
 callable=[a for a in inv['entries'] if a['category'] in ['callable','inherited-default']];impls=[a for a in inv['entries'] if a['category']=='implementation'];co=collections.Counter()
 for a in callable:
  cands=[i for i in impls if i['owner_raw_type']==a['owner_raw_type'] and i['trait_canonical_with_arguments']==a['trait_canonical_with_arguments']]
  assert len(cands)<=1,(config,a['rust_api_identity'],len(cands));imp=cands[0] if cands else None
  rawimp=imp['raw_item']['impl'] if imp else None
  actualnames={d['index'][str(id)]['name'] for id in rawimp['items'] if str(id) in d['index'] and 'function' in d['index'][str(id)]['inner']} if rawimp else set()
  shadow=a['category']=='inherited-default' and a['name'] in actualnames
  derived=bool(imp and 'automatically_derived' in imp['attrs'])
  stdtrait=bool(a['trait_canonical_path'] and a['trait_canonical_path'].startswith(('core::','alloc::','std::')))
  nonstdderive=derived and not stdtrait
  inherited=a['category']=='inherited-default'
  trmeta=traitmeta.get(a['trait_canonical_path'],{})
  methodst=(a.get('raw_item_metadata')or{}).get('stability')
  # Original inventory kept only inner item data. Recover direct method stability
  # from the original rustdoc item by exact inner/span/name; inherited trait gate separately retained.
  rawmatches=[i for i in d['index'].values() if i['name']==a['name'] and i.get('span')==a['source_span'] and i['inner']==a['raw_item']]
  methodst=rawmatches[0].get('stability') if rawmatches else trmeta.get('methods',{}).get(a['name'],{}).get('stability') or methodst
  gates=[]
  for source,st in [('method',methodst),('trait',trmeta.get('stability'))]:
   if st and st.get('level')=='unstable':gates.append({'source':source,'feature':st['feature']})
  classification='shadowed_provided_default_not_executable_extra' if shadow else 'excluded_derived_standard_impl' if derived and stdtrait else 'requires_consumer_nightly_gate' if gates else 'signature_candidate_under_retained_bounds'
  co[classification]+=1
  owner=a['owner_canonical_type'];trait=a['trait_canonical_with_arguments'];corrected=a['canonical_rust_api_identity'];bindings={}
  if rawimp and isinstance(rawimp.get('blanket_impl'),dict) and 'generic' in rawimp['blanket_impl']:
   binder=rawimp['blanket_impl']['generic'];bindings[binder]=owner
   # The concrete examples affected are reflexive From/Borrow/BorrowMut.
   if trait and '<'+binder+'>' in trait:trait=trait.replace('<'+binder+'>','<'+owner+'>')
  if owner and trait:corrected=f'<{owner} as {trait}>::{a["name"]}'
  name=a['name'];evidence={'machine3':None,'boundedDisplay1910':None,'boundedCodec1484':None}
  machine=name in ['encoded_len','decoded_len'] and not owner or (name=='from' and owner=='core::fmt::Error' and a['origin']=='crate manual')
  if machine:evidence['machine3']={'scope':'original existing three default-feature mapped claims; encoded_len <=usizeMAX/2 remains required','direct_config_covered':config=='default','other_config_note':'source/config correspondence only; not a new feature-run claim' if config!='default' else None}
  if owner=='base16ct::Error' and a['trait_canonical_path']=='core::fmt::Display':evidence['boundedDisplay1910']={'packet':'/workspace/full-api/endian/base16ct-display-publisher-ready','direct_config_covered':config=='default','scope':'two Error variants, default {}, fixed64 writer accept/first-error; corrected literal23/21'}
  if a['rust_api_identity'] in ['base16ct::lower::decode','base16ct::upper::decode','base16ct::mixed::decode','base16ct::lower::encode','base16ct::upper::encode']:evidence['boundedCodec1484']={'packet':'/workspace/full-api/endian/base16ct-byte-codec-kani','direct_config_covered':config=='default','scope':'decode AsRef instantiated &[u8], src2/dst3, suffix1/2 only on Err; encode src1/dst4 plus roundtrip; not arbitrary AsRef or lengths'}
  r={'configuration':config,'raw_row':a,'raw_impl':imp,'classification':classification,'derived_standard_scope_basis':'specific implementation automatically_derived, not merely type derives other traits' if derived else None,'nonstandard_derived':nonstdderive,'shadowed_by_actual_method':shadow,'actual_override_names':sorted(actualnames) if shadow else None,'trait_metadata':trmeta,'effective_nightly_gates':gates,'corrected_identity_display':corrected,'blanket_template_binding_to_concrete_self':bindings,'generic_bounds':'raw impl/item/signature constraints retained; arbitrary instantiation satisfaction not asserted','evidence_columns':evidence,'full_API_completion':False}
  if a['name']=='type_id' and a['trait_canonical_path']=='core::error::Error':r['extra_argument_privacy_obligation']='private::Internal parameter construction/access not resolved; gated/internal helper is not advertised as ordinary callable'
  rows.append(r)
  if shadow:shadows.append(r)
 summary.append({'configuration':config,'all_ledger_records':inv['entry_count'],'raw_callable_and_inherited_records':len(callable),'strict_partition':dict(co),'distinct_callable_signatures_before_derive_exclusion':len(callable)-co['shadowed_provided_default_not_executable_extra'],'non_derived_candidates_including_gated':co['requires_consumer_nightly_gate']+co['signature_candidate_under_retained_bounds'],'structural_and_implementation_records':inv['entry_count']-len(callable),'nonstandard_derived_functions':sum(r['nonstandard_derived'] for r in rows if r['configuration']==config)})
for f,x in [('strict-rows.json',rows),('summary.json',summary),('shadow-exact-rows.json',shadows),('input-hashes.json',inputs),('trait-gate-source.json',traitmeta)]:
 (OUT/f).write_text(json.dumps(x,indent=2)+'\n')
print(json.dumps(summary,indent=2))
