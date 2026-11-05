from pathlib import Path
import json,tomllib,collections,re,hashlib
b=Path('/workspace/full-api/prng');out=Path(__file__).resolve().parent;versions={'oorandom':'11.1.5','rand_pcg':'0.10.2','rand_xoshiro':'0.8.1','rand_xorshift':'0.5.0'}
allrows=[];summaries=[];shadows=[];issues=[];supp=[]
def norm_claim(api):
 m=re.match(r'^<(.+) as (.+)>::(\w+)$',api)
 if m:return m[1].rsplit('::',1)[-1],m[2].split('<')[0].rsplit('::',1)[-1],m[3]
 parts=api.rsplit('::',2);return parts[-2],None,parts[-1]
def norm_record(r):return r['owner'].rsplit('::',1)[-1],r['trait'].split('<')[0].rsplit('::',1)[-1]if r.get('trait')else None,r['method']
def add(c,method,types,id,domain,path,level='restricted_public_API_contract',configs=['default']):
 supp.append({'crate':c,'method':method,'types':types,'id':id,'domain':domain,'path':str(path),'level':level,'configurations':configs,'does_not_add_Aeneas_registration':True})
add('oorandom','rand_float',['Rand32','Rand64'],'oor-float-range2','Allpublicfrom_state state/inc; output0<=f<1 only; exactIEEE/state/distribution notproved',b/'kani-oorandom-float-range')
add('rand_pcg','from_seed',['Mcg128Xsl64'],'pcg-mcg-seed1','All16byteseeds; exactpublicstate=LE128|1; baselinecore0.10.0',b/'kani-pcg-mcg-seed',configs=['default','all-features'])
add('rand_pcg','fmt',['Lcg64Xsh32','Lcg128Xsl64','Lcg128CmDxsm64','Mcg128Xsl64'],'pcg-debug4-8925','Publicconstructorimages; defaultformat/64bytewriter acceptorfirstErr; exactredactedliteral/error',b/'kani-pcg-debug-rand-core-0.10.0')
types=['SplitMix64','Xoroshiro64Star','Xoroshiro64StarStar','Xoroshiro128Plus','Xoroshiro128PlusPlus','Xoroshiro128StarStar','Xoshiro128Plus','Xoshiro128PlusPlus','Xoshiro128StarStar','Xoshiro256Plus','Xoshiro256PlusPlus','Xoshiro256StarStar','Xoshiro512Plus','Xoshiro512PlusPlus','Xoshiro512StarStar']
add('rand_xoshiro','from_seed',types,'xoshiro-seed15','Everyfixedlengthseedinclzero; exactLEwords/fallback; originalfullsourceprefix+observer, notAeneas',b/'kani-xoshiro-seed',configs=['default','all-features'])
add('rand_xoshiro','state',types,'xoshiro-state15','Allinternalwordpatterns; everyLEoutputbyte; originalprefix/privateconstructors; corroboratesexistingAeneasstate15',b/'kani-xoshiro-state',configs=['default','all-features'])
add('rand_xoshiro','try_next_u64',['Xoroshiro64Star','Xoroshiro64StarStar','Xoshiro128Plus','Xoshiro128PlusPlus','Xoshiro128StarStar'],'xoshiro-composition5','Allinternalwordpatterns; independent2stepoutput/fullstate; originalprefixobservers',b/'kani-xoshiro-composition',configs=['default','all-features'])
x=b/'xorshift-coverage-reconciliation/coverage.json';xj=json.loads(x.read_text())
for r in xj['callables']:
 if r['configuration']=='default'and r.get('evidence')and r['method']!='try_next_u32':
  e=r['evidence'];add('rand_xorshift',r['method'],['XorShiftRng'],'xorshift-'+r['method'],e['domain']+'; '+e['guarantee'],e['path'])
for crate,ver in versions.items():
 j=json.loads((b/(crate+'.json')).read_text());package=Path('/workspace/publish-full-api')/crate/ver/'aeneas/2026-11-05';claims=tomllib.loads((package/'proofs.toml').read_text())['tool']['proofs'];claims_by={norm_claim(c['api']):c for c in claims};assert len(claims_by)==len(claims)
 depver='0.10.1'if crate=='rand_xoshiro'else'0.10.0';depdir='dependencies-0.10.1'if depver=='0.10.1'else'dependencies';dep=json.loads((b/f'rustdoc/{depdir}/x86_64-unknown-linux-gnu/doc/rand_core.json').read_text());seedtrait=next(v for v in dep['index'].values()if v['name']=='SeedableRng'and'trait'in v['inner']);defaults={dep['index'][str(i)]['name']:dep['index'][str(i)]for i in seedtrait['inner']['trait']['items']if dep['index'][str(i)]['inner'].get('function',{}).get('has_body')}
 for mode in j['configurations']:
  rs=[r for r in j['callables']if r['configuration']==mode];rawpath=b/f'rustdoc/{crate}/{mode}/x86_64-unknown-linux-gnu/doc/{crate}.json';doc=json.loads(rawpath.read_text());idx=doc['index'];features=['serde']if mode=='all-features'and'serde'in j['manifest_features']else[]
  for ir in j['trait_impls']:
   if ir['configuration']!=mode or not ir.get('trait','')or'SeedableRng'not in ir['trait']:continue
   im=idx[str(ir['rustdoc_id'])]['inner']['impl'];own={idx[str(i)]['name']:idx[str(i)]for i in im['items']}
   for name in sorted(defaults.keys()&own.keys()):
    shadows.append({'crate':crate,'version':ver,'configuration':mode,'owner':ir['owner'],'trait':ir['trait'],'method':name,'classification':'overridden_trait_default_not_an_additional_callable','active_override_origin':ir['origin'],'active_override_span':own[name]['span'],'shadowed_default_source':defaults[name]['span'],'trait_impl_id':ir['rustdoc_id'],'provided_defaults':im['provided_trait_methods'],'rustdoc':str(rawpath)})
  hits=[]
  for r in rs:
   row={'crate':crate,'version':ver,'configuration':mode,'enabled_local_features':features,'runtime_identity_observed':r['canonical_api'],'kind':r['kind'],'origin':r['origin'],'owner':r['owner'],'trait':r.get('trait'),'method':r['method'],'generic_signature':r.get('signature'),'impl_generics':r.get('impl_generics'),'raw_item':r.get('raw_item'),'source_span':r.get('source_span'),'rustdoc_path':str(rawpath),'machine_claim':None,'supplementary_tool_contracts':[]}
   if r['kind']=='tuple_struct_constructor':row['surface_class']='public_data_constructor_operation';row['counted_as_function_callable']=False
   else:row['surface_class']=r['origin'];row['counted_as_function_callable']=True
   row['user_std_derive_exclusion']=r['origin']=='derived_standard';row['nonstd_derive_never_excluded']=r['origin']=='derived_nonstandard'
   imps=[i for i in j['trait_impls']if i['configuration']==mode and i['owner']==r['owner']and i.get('trait')==r.get('trait')]
   rawimp=idx[str(imps[0]['rustdoc_id'])]['inner']['impl']if len(imps)==1 else None
   row['impl_binding_metadata']={'rustdoc_id':imps[0]['rustdoc_id'],'blanket_impl':rawimp.get('blanket_impl'),'provided_trait_methods':rawimp.get('provided_trait_methods'),'source_span':imps[0].get('source_span')}if rawimp else None
   selfparam=rawimp.get('blanket_impl',{}).get('generic')if rawimp and isinstance(rawimp.get('blanket_impl'),dict)else None
   params=(r.get('impl_generics')or{}).get('params',[]);remaining=[p['name']for p in params if'type'in p['kind']and p['name']!=selfparam];methodparams=(r.get('signature')or{}).get('generics',{}).get('params',[]);remaining += [p['name']for p in methodparams if'type'in p['kind']]
   row['self_substitution']={selfparam:r['owner']}if selfparam else {};row['remaining_type_parameters']=remaining
   if remaining:row['bound_class']='conditional_generic_bound_not_discharged';row['unconditional_concrete_callability']=False
   elif r.get('signature')is None and r['origin']=='inherited_default':row['bound_class']='external_default_signature_missing_in_old_inventory';row['unconditional_concrete_callability']='not_established_by_this_inventory'
   else:row['bound_class']='concrete_self_observed_or_declaration_bounds_retained';row['unconditional_concrete_callability']='compiler_observed_with_retained_predicates'
   if selfparam:row['runtime_identity_self_substituted_display']=re.sub(r'(?<![A-Za-z0-9_])'+re.escape(selfparam)+r'(?![A-Za-z0-9_])',r['owner'],r['canonical_api'])
   if selfparam:row['identity_caution']='Rustdocblanketparameter '+selfparam+'isSelf-substituted; observedgenericspellingmustnotbeinterpretedasarbitraryadditionaltypeargument'
   if r['origin']=='external_blanket'and(r.get('trait')or'').startswith('alloc::borrow::ToOwned'):row['cfg_caution']='allocgraphvisibilitydifference; NOTprovedlocalcfg(feature=serde)gate'
   if (r.get('trait')or'').endswith("Deserialize<'de>")and r['method']=='deserialize_in_place':row['external_default_source_supplement']='serde_core1.0.228/src/de/mod.rs::Deserialize::deserialize_in_place, exactsourceexcerptinserde-default-source.json; genericDpredicateunproved';row['bound_class']='conditional_generic_bound_not_discharged';row['remaining_type_parameters']=['D'];row['unconditional_concrete_callability']=False
   key=norm_record(r);claim=claims_by.get(key)
   applicable=not(crate=='rand_xorshift'and features)
   if claim and applicable and r['kind']=='function'and not row['user_std_derive_exclusion']:
    row['machine_claim']={'configured_exact_key':claim['api'],'module':claim['module'],'theorem':claim['theorem'],'precondition':claim['precondition'],'configuration_note':'Default/no-defaultemptyfeatureequivalenceisnotaseparateverificationrun'if mode=='no-default-features'else'configurationsasdocumentedinthepacket','packet':str(package)};hits.append(claim['api'])
   for s in supp:
    if s['crate']==crate and s['method']==r['method']and r['owner'].rsplit('::',1)[-1]in s['types']and mode in s['configurations']:row['supplementary_tool_contracts'].append(s['id'])
   row['no_automatic_completion_from_partial_supplements']=True
   allrows.append(row)
  if mode=='default':assert len(hits)==len(set(hits))==len(claims),(crate,len(hits),len(claims))
  functions=[r for r in rs if r['kind']=='function'];ctor=[r for r in rs if r['kind']!='function'];origins=collections.Counter(r['origin']for r in functions)
  summaries.append({'crate':crate,'version':ver,'configuration':mode,'old_total_entries':len(rs),'function_callable_entries':len(functions),'public_data_constructor_operations':len(ctor),'origin_counts_functions_only':dict(origins),'std_derived_excluded':origins['derived_standard'],'nonexcluded_function_entries':len(functions)-origins['derived_standard'],'machine_API_keys_matched_in_configuration':len(set(hits)),'default_unique_machine_claims':len(claims)if mode=='default'else None,'conditional_generic_entries':sum(x['crate']==crate and x['configuration']==mode and x['bound_class']=='conditional_generic_bound_not_discharged'for x in allrows),'no_claim_of_all_downstream_external_trait_completeness':True})
(out/'coverage.json').write_text(json.dumps({'scope':'4crateversions/x86_64/3rustdocgraphconfigurations; retainsconditionalexternalfamiliesandpublicdataconstructors; notallAPIproved','machine_unique_API_count':sum(x['default_unique_machine_claims'] or 0 for x in summaries),'summaries':summaries,'callables_and_data_operations':allrows,'supplementary_tool_contracts':supp,'overridden_trait_defaults':shadows},indent=2))
(out/'summary.json').write_text(json.dumps(summaries,indent=2));(out/'shadowed-defaults.json').write_text(json.dumps(shadows,indent=2));print('rows',len(allrows),'shadows',len(shadows),'machine',sum(x['default_unique_machine_claims'] or 0 for x in summaries))
