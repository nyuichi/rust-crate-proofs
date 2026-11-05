from pathlib import Path
import json,tomllib,hashlib,re,csv
OUT=Path(__file__).resolve().parent
ROOT=Path('/workspace/full-api/endian/endian66-publication-candidate/endian-type/0.2.0/aeneas/2026-11-05')
PROBE=Path('/workspace/full-api/enum/typed-self-scanner/endian66-after.log')
probe=[json.loads(line) for line in PROBE.read_text().splitlines() if line.startswith('[')];assert len(probe)==66
source=(ROOT/'src/lib.rs').read_bytes();source_sha=hashlib.sha256(source).hexdigest();manifest_sha=hashlib.sha256((ROOT/'Cargo.toml').read_bytes()).hexdigest()
config=tomllib.loads((ROOT/'proofs.toml').read_text());assert len(config['tool']['proofs'])==66
base_json=json.loads((ROOT/'evidence/endian_type.llbc').read_text())['translated'];integer_json=json.loads((ROOT/'evidence/integer-models/endian_verification_driver.llbc').read_text())['translated']
base_manifest=json.loads((ROOT/'proofs/Generated/translation.json').read_text());integer_manifest=json.loads((ROOT/'proofs/EndianVerificationDriver/translation.json').read_text());binding=json.loads((ROOT/'evidence/integer-models/from-proof-bindings.json').read_text());bind_by_theorem={b['theorem']:b for b in binding}

def span_record(t,s):
 data=s['Untagged'];result={}
 for field in ['data','generated_from_span']:
  sp=data.get(field)
  if sp is None:result[field]=None;continue
  file=next(f for f in t['files'] if f['id']==sp['file_id'])
  assert file['crate_name']=='endian_type',(field,file['name'])
  if file['contents'] is not None:assert file['contents'].encode()==source
  result[field]={'file_name':file['name'],'file_crate':file['crate_name'],'source_contents_embedded_in_LLBC':file['contents'] is not None,'file_contents_sha256':hashlib.sha256(file['contents'].encode()).hexdigest() if file['contents'] is not None else None,'separate_archive_source_hash':source_sha,'source_origin_evidence':'Direct source is embedded for generic roots; dependency spans are compiler-remapped Virtual paths, with original archive/source hashes and exact path-dependency/compiler command preserved.','begin_line':sp['beg']['line'],'begin_column_zero_based':sp['beg']['col'],'end_line':sp['end']['line'],'end_column_zero_based':sp['end']['col']}
 return result

def body_span(start):
 lines=source.decode().splitlines();balance=0;seen=False
 for i in range(start-1,len(lines)):
  balance+=lines[i].count('{')-lines[i].count('}');seen=seen or '{' in lines[i]
  if seen and balance==0:return {'file':'src/lib.rs','begin_line':start,'end_line':i+1,'derivation':'brace-delimited original macro function text, distinct from compiler signature/invocation spans','text':'\n'.join(lines[start-1:i+1])}
 raise AssertionError(start)

entries=[];recommended=[]
for index,proof in enumerate(config['tool']['proofs']):
 if index<6:
  old=proof['api'];owner=re.search(r'<endian_type::(\w+) as ',old).group(1);trait=old.split(' as ')[1].split('>::')[0];method=old.rsplit('::',1)[1]
  rust=f'<endian_type::{owner}<T> as {trait}>::{method}';key=rust
  matches=[f for f in base_manifest['functions'] if f['rust_name'].endswith('::'+method) and ' for endian_type::'+owner+'<T>' in f['rust_name']];assert len(matches)==1;fn=matches[0];t=base_json
  declaration=next(f for f in t['fun_decls'] if f and f['def_id']==fn['def_id'])
  macro_start={'bitand':18,'bitor':30,'bitxor':42}[method];origin='original macro body extracted directly with official method roots';body_local=True
  input_ty=f'endian_type::{owner}<T>';output_ty=f'endian_type::{owner}<<T as {trait}>::Output>';self_ownership='local_generic';bounds=f'T: {trait}<Rhs=T>; default Rhs for the wrapper is Self'
 else:
  b=bind_by_theorem[proof['theorem']];rust=b['rust_api_identity'];key=rust.replace('core::convert::From','From');input_ty=b['input_rust_type'];output_ty=b['output_rust_type'];method='from';bounds='Concrete primitive specialization; no generic type parameter is erased'
  fn=next(f for f in integer_manifest['functions'] if f['lean_name']==b['lean_function']);t=integer_json;declaration=next(f for f in t['fun_decls'] if f and f['def_id']==fn['def_id'])
  family=proof['theorem'].removeprefix('FromProofs.').rsplit('_',2)[0]
  # Recover family from the actual theorem labels, preserving signed/size types.
  for prefix,start in [('be_from_',90),('le_from_',153),('be_into_',83),('le_into_',146),('le_to_be_',97),('be_to_le_',160)]:
   if proof['theorem'].removeprefix('FromProofs.').startswith(prefix):macro_start=start;break
  else:raise AssertionError(proof)
  self_ownership='foreign_primitive' if output_ty in ['u16','u32','u64','u128','usize','i16','i32','i64','i128','isize'] else 'local_concrete_wrapper'
  origin='original dependency macro body reached through pure caller driver; original body is transparent and is the theorem target';body_local=False
  assert not fn['is_local'] and not fn['is_opaque']
 assert declaration['item_meta']['is_local']==body_local and declaration['item_meta']['opacity']=='Transparent' and declaration.get('body') is not None
 assert declaration['item_meta']['attr_info']['public']
 assert key in probe[index],(index,key,probe[index])
 if self_ownership=='foreign_primitive':assert key.startswith('<'+output_ty+' as From<')
 def owner_of(path):return path[1:].split(' as ',1)[0]
 expected_owner=owner_of(key)
 legacy=[p for p in probe[index] if owner_of(p)!=expected_owner]
 # New recorded bindings use exactly one source-observed path; do not send
 # erased aliases that the server could prefer merely because they already exist.
 predicted=hashlib.sha256(json.dumps(['recorded-api-v1','endian-type','0.2.0',key],ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
 proof_file=ROOT/'proofs'/(proof['module'].replace('.','/')+'.lean');proof_lines=proof_file.read_text().splitlines();theorem_short=proof['theorem'].rsplit('.',1)[1];decl_line=next(i+1 for i,line in enumerate(proof_lines) if re.match(r'theorem '+re.escape(theorem_short)+r'\b',line))
 s=span_record(t,declaration['item_meta']['span'])
 row={'contract_index':index,'exact_rust_api_identity':rust,'recommended_recorded_source_api_key':key,'record_api_paths_exactly':[key],'trait_resolution':{'public_resolved_trait':'core::convert::From' if index>=6 else trait,'source_scanner_trait_spelling':'From (standard prelude)' if index>=6 else trait,'equivalence_does_not_erase_type_arguments':True},'self_type':expected_owner,'self_ownership':self_ownership,'input_rust_type':input_ty,'output_rust_type':output_ty,'generic_or_specialization_bounds':bounds,'configured_candidate_api_before_mapping':proof['api'],'accepted_exact_key_observed_in_compiler_expanded_source_probe':True,'source_probe_candidates':probe[index],'erased_self_aliases_to_exclude_from_run':legacy,'proof_module':proof['module'],'proof_theorem':proof['theorem'],'proof_theorem_decl_line':decl_line,'proof_file_relative':'proofs/'+proof['module'].replace('.','/')+'.lean','proof_file_sha256':hashlib.sha256(proof_file.read_bytes()).hexdigest(),'proof_precondition':proof['precondition'],'generated_original_function':fn['lean_name'],'generated_original_rust_name':fn['rust_name'],'charon_def_id':fn['def_id'],'actual_body_origin':origin,'actual_original_function_is_local_to_extraction_root':body_local,'actual_original_function_is_opaque':False,'actual_original_function_is_FFI_extern':declaration['item_meta']['is_extern'],'charon_signature_ast':declaration['signature'],'actual_original_function_body_present':True,'compiler_spans':s,'original_macro_function_body_span':body_span(macro_start),'original_source_sha256':source_sha,'original_cargo_manifest_sha256':manifest_sha,'target':'x86_64-unknown-linux-gnu','source_cargo_package':'endian-type 0.2.0','server_binding':{'not_requested_by_this_audit':True,'actual_api_item_id':None,'if_exact_key_absent_recorded_id':predicted,'predicted_id_is_not_a_live_server_observation':True,'origin_if_missing':'recorded','existing_matching_exact_rustdoc_or_recorded_id_may_be_reused':True,'never_reuse_erased_or_foreign-self-corrupted_key':True}}
 entries.append(row);recommended.append((proof['api'],key))
assert len(entries)==66 and len({x['recommended_recorded_source_api_key'] for x in entries})==66
assert sum(x['self_ownership']=='foreign_primitive' for x in entries)==20
assert sum(x['self_ownership']=='local_generic' for x in entries)==6
out={'crate':'endian-type','version':'0.2.0','source_snapshot':str(ROOT),'source_sha256':source_sha,'manifest_sha256':manifest_sha,'entries':entries,'recording_policy':'Only one exact typed source path per contract. Existing erased aliases are excluded. Publisher alone performs official run preview/upload/publication; no API requests were made by this audit.','official_server_source_revision':'0a39304e702b196e6dff917bbf17c63dc01abcc6','already_imported_release_evidence':'/workspace/batch-ten/service-limit-details.json: endian-type 0.2.0 ready 200','not_a_new_import_request':True}
(OUT/'bindings-66.json').write_text(json.dumps(out,indent=2)+'\n')
with (OUT/'bindings-66.tsv').open('w') as f:
 writer=csv.writer(f,delimiter='\t');writer.writerow(['index','Rust exact identity','recorded source key','self ownership','macro invocation line','macro body lines','theorem','generated actual function'])
 for e in entries:
  invocation=e['compiler_spans']['data'];body=e['original_macro_function_body_span'];writer.writerow([e['contract_index'],e['exact_rust_api_identity'],e['recommended_recorded_source_api_key'],e['self_ownership'],invocation['begin_line'],str(body['begin_line'])+'-'+str(body['end_line']),e['proof_theorem'],e['generated_original_function']])
config_text=(ROOT/'proofs.toml').read_text()
for old,key in recommended:
 needle='api = '+json.dumps(old);assert config_text.count(needle)==1;config_text=config_text.replace(needle,'api = '+json.dumps(key),1)
(OUT/'proofs-source-keys.toml').write_text(config_text);assert len(tomllib.loads(config_text)['tool']['proofs'])==66
(OUT/'summary.json').write_text(json.dumps({'contracts':66,'unique_exact_recorded_keys':66,'generic_bitwise_owners_preserve_T':6,'concrete_integer_From_implementations':60,'foreign_primitive_self_implementations_preserved':20,'actual_original_bodies_present':66,'embedded_direct_LLBC_source_matches':6,'dependency_source_bodies_use_archive_hash_guarded_compiler_virtual_spans':60,'source_probe_rows':66,'source_scanner_exact_key_matches':66,'live_server_requests_by_this_audit':0,'shared_CLI_files_modified_by_this_audit':0},indent=2)+'\n')
print('66 source-verified typed keys; primitive self20; generic self6; actual original bodies66; no live API calls')
