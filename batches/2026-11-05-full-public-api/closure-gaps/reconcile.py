from pathlib import Path
import json,collections,hashlib
root=Path(__file__).resolve().parents[1];out=Path(__file__).resolve().parent
def obligation(status,name):
 if status=='partial-Kani-integer-specializations-generic-unproved':return ['arbitrary Sized+Copy T: representation validity, initialization/padding, layout, provenance and borrowing','arbitrary admissible lengths and offsets; first size_of(T) bytes only','faithful original raw-pointer and raw-slice behavior; discharge public-reachability premises']
 if 'Debug' in status:return ['arbitrary stored value/T and general Formatter/writer/options remain unproved','no_std Debug not evaluated' if 'no-std' in status else 'current guarantee only Default/zero-or-one write_u8, fixed64 default-format writer']
 if 'external' in status:return ['original external implementation behavior, applicability bounds and standard-library version','compose local operation contracts where relevant; model composition alone is not std-body verification']
 if status.startswith('supplemental'):return ['connect official Clone default model to original Rust std body before promoting to full inherited API guarantee']
 if 'target-specific' in status:return ['maintain explicit 50 std-operation models and reviewed Rust/Lean bridge','little-endian x86_64 with pointer64 premises; no other-target claim']
 if 'generic-trait-model' in status:return ['underlying T operation uses assumed official trait dictionary/contract']
 if 'concrete-integer' in status:return ['retain ten concrete integer types, array domain and x86_64 target qualification; not generic raw-memory closure']
 return []
def blocker(status,name):
 if status=='partial-Kani-integer-specializations-generic-unproved':return 'official InterpPaths.ml:112 raw dereference unsupported' if name=='from_byte_slice' else 'official InterpExpressions.ml:1483 raw-address rvalue unsupported; general memory/layout/permissions model absent'
 if 'Debug' in status:return 'general Aeneas Formatter model discards output; Kani supplement has bounded u8/writer domain only'
 if 'external' in status or status.startswith('supplemental'):return 'not discharged by local crate-body proofs; separate standard-library/body correspondence boundary'
 return None
rows=[];summaries=[];inputs={}
for crate,configs in [('endian-type',['default','all']),('nohash-hasher',['default','all','no-default','default-release','all-release','no-default-release'])]:
 for config in configs:
  path=root/crate/config/'inventory.json';p=json.loads(path.read_text());inputs[str(path.relative_to(root))]=hashlib.sha256(path.read_bytes()).hexdigest();counts=collections.Counter()
  for a in p['callables']:
   origin=a['origin'];name=a['name'];identity=a['rust_api_identity']
   if crate=='endian-type':
    if origin=='derived_standard':status='excluded-derived-standard';evidence=None
    elif origin=='external_blanket':status='unproved-external-blanket';evidence=None
    elif name in ['bitand','bitor','bitxor']:status='Aeneas-original-generic-trait-model-contract';evidence='endian66-publication-candidate'
    elif name=='from':status='Aeneas-original-target-specific-explicit-std-op-models';evidence='integer-models'
    elif name in ['to_bytes','from_bytes']:status='Kani-original-concrete-integer-all-inputs';evidence='integer-byte-apis'
    elif name in ['as_byte_slice','as_byte_slice_mut','from_byte_slice']:status='partial-Kani-integer-specializations-generic-unproved';evidence='integer-slice-apis'
    else:raise AssertionError(identity)
   else:
    if origin=='crate_manual':
     if name=='fmt':status='partial-Kani-u8-Debug' if config in ['default','all','default-release','all-release'] else 'unproved-Debug-no-std-not-evaluated';evidence='nohash-debug-probe' if status.startswith('partial') else None
     else:status='Aeneas-original-body-std-and-no-std';evidence='nohash-publication-candidate'
    elif name=='clone_from' and a.get('owner_type')=='nohash_hasher::NoHashHasher<T>' and origin=='inherited_default':status='supplemental-official-Clone-default-model-composition-only';evidence='nohash-supplement'
    else:status='unproved-external-'+origin.removeprefix('external_');evidence=None
   counts[status]+=1
   rows.append({'crate':crate,'configuration':config,'local_id':a['local_id'],'rust_api_identity':identity,'origin':origin,'inventory_ref':str(path.relative_to(root)),'inventory_reference_preserves_full_bounds_signature_and_spans':True,'reconciled_status':status,'evidence_directory':str(root/evidence) if evidence else None,'source_span':a.get('source_span'),'impl_source_span':a.get('impl_source_span'),'target':a.get('target','x86_64-unknown-linux-gnu'),'obligations':obligation(status,name),'blocker':blocker(status,name),'generic_api_complete':False if ('partial' in status or 'unproved' in status or 'supplemental' in status) else None})
  summaries.append({'crate':crate,'configuration':config,'callables':len(p['callables']),'counts':dict(counts),'structural_items':dict(collections.Counter(a['kind'] for a in p['types_constants_and_associated_items']))})
assert [s for s in summaries if s['crate']=='endian-type'][0]['counts']=={'Aeneas-original-generic-trait-model-contract':6,'excluded-derived-standard':46,'Kani-original-concrete-integer-all-inputs':40,'Aeneas-original-target-specific-explicit-std-op-models':60,'partial-Kani-integer-specializations-generic-unproved':6,'unproved-external-blanket':16}
(out/'coverage-reconciliation.json').write_text(json.dumps({'scope':'local evidence reconciliation; not publication state or full API completion','input_sha256':inputs,'summaries':summaries,'rows':rows},indent=2)+'\n')
# External boundaries explicitly retained, not absorbed into local proof counts.
(out/'external-api-gaps.json').write_text(json.dumps([r for r in rows if r['origin'] in ['external_blanket','inherited_default','external_alias_target']],indent=2)+'\n')
print(json.dumps(summaries,indent=2))
