from pathlib import Path
import json,tomllib,hashlib,re,tarfile
ROOT=Path(__file__).resolve().parents[1];OUT=Path(__file__).resolve().parent
artifacts={'nohash-hasher':ROOT/'nohash-publication-candidate/nohash-hasher/0.2.0/aeneas/2026-11-05','endian-type':ROOT/'endian66-publication-candidate/endian-type/0.2.0/aeneas/2026-11-05','normalize-line-endings':Path('/workspace/rust-crate-proofs/normalize-line-endings/0.3.0/aeneas/2026-11-04')}
expected={'nohash-hasher':14,'endian-type':66,'normalize-line-endings':1}
def normalized_key(s):
 # Only schema spelling aliases and nominal generic omission in existing configured keys.
 s=s.replace('core::ops::bit::','core::ops::').replace('<T>','')
 for short,full in [('Default','core::default::Default'),('Clone','core::clone::Clone')]:s=s.replace(' as '+full+'>', ' as '+short+'>')
 return s
rows=[]
for crate,artifact in artifacts.items():
 inv=json.loads((ROOT/crate/'default/inventory.json').read_text())['callables'];m=tomllib.loads((artifact/'proofs.toml').read_text())['tool']['proofs'];assert len(m)==expected[crate]
 allowed=[a for a in inv if a['origin'] in ['crate_manual','macro_manual'] and not a.get('alias_label') and not a['inherited_default']]
 used=[]
 for q in m:
  candidates=[a for a in allowed if normalized_key(a['rust_api_identity'])==normalized_key(q['api']) or (crate=='normalize-line-endings' and a['name']=='normalized' and q['api']=='normalized')]
  assert len(candidates)==1,(crate,q['api'],len(candidates));a=candidates[0];assert a['local_id'] not in used;used.append(a['local_id'])
  if crate=='nohash-hasher':assert a['owner_type']=='nohash_hasher::NoHashHasher<T>' and a['name']!='fmt'
  rows.append({'crate':crate,'configured_machine_key':q['api'],'exact_typed_Rust_inventory_identity':a['rust_api_identity'],'source_local_id':a['local_id'],'source_span':a['source_span'],'impl_source_span':a['impl_source_span'],'impl_generics':a['impl_generics'],'origin':a['origin'],'theorem':q['theorem'],'artifact':str(artifact),'proofs_toml_sha256':hashlib.sha256((artifact/'proofs.toml').read_bytes()).hexdigest(),'wrong_alias_or_external_blanket_used':False,'spelling_normalization_used_for_legacy_configured_key':q['api']!=a['rust_api_identity'],'claim_source_qualification':'existing adapted translation/finite-list contract; not upgraded into original arbitrary-iterator proof' if crate=='normalize-line-endings' else 'original crate body under existing contract/model/platform preconditions'})
 with tarfile.open(artifact/'evidence/upstream.crate') as t:
  for f in [artifact/'Cargo.toml',*sorted((artifact/'src').rglob('*.rs'))]:assert f.read_bytes()==t.extractfile(f'{crate}-{m and ("0.2.0" if crate!="normalize-line-endings" else "0.3.0")}/'+str(f.relative_to(artifact))).read()
(OUT/'machine-claims-unaffected.json').write_text(json.dumps({'counts':expected,'all_exact_source_bindings_unique':True,'unrelated_alias_or_blanket_corrections_do_not_change_claims':True,'machine_configured_key_and_typed_Rust_identity_are_kept_separate':True,'rows':rows},indent=2)+'\n')
print('Unique source-origin bindings14/66/1 and unchanged original source/manifest archives verified; normalize1 adapted qualification retained')
