from pathlib import Path
import json,hashlib,tarfile
b=Path('/workspace/full-api/prng');out=Path(__file__).resolve().parent;p=b/'artifacts-next3/rand_xoshiro/0.8.1/aeneas/2026-11-05';w=p/'evidence/rng-selection-wrapper'
sha=lambda data:hashlib.sha256(data).hexdigest()
archive=w/'rand_core-0.10.1.crate'
with tarfile.open(archive)as t:
 files={m.name:sha(t.extractfile(m).read())for m in t.getmembers()if m.isfile()};source=t.extractfile('rand_core-0.10.1/src/lib.rs').read()
assert sha(archive.read_bytes())=='63b8176103e19a2643978565ca18b50549f6101881c443590420e4dc998a3c69'
rows=[]
for mode in ['default','all-features']:
 raw=b/f'rustdoc/rand_xoshiro/{mode}/x86_64-unknown-linux-gnu/doc/rand_xoshiro.json';j=json.loads(raw.read_text());index=j['index']
 for e in json.loads((w/'api-bindings.json').read_text()):
  impls=[];own=[]
  for v in index.values():
   im=v['inner'].get('impl')
   if not im or im.get('for',{}).get('resolved_path',{}).get('path')!=e['type']:continue
   tr=im.get('trait')
   if not tr:continue
   path=j['paths'].get(str(tr['id']),{}).get('path')
   if path==['rand_core','Rng']:impls.append(v)
   if path==['rand_core','TryRng'] and im.get('blanket_impl')is None:own.append(v)
  assert len(impls)==len(own)==1,(e,len(impls),len(own))
  v=impls[0];im=v['inner']['impl'];oi=own[0]['inner']['impl'];assert im['blanket_impl']=={'generic':'R'}
  meth=[index[str(i)]for i in im['items']if index[str(i)]['name']=='next_u'+str(e['bits'])];err=[index[str(i)]for i in oi['items']if index[str(i)]['name']=='Error'];assert len(meth)==len(err)==1
  assert meth[0]['inner']['function']['sig']['output']=={'primitive':'u'+str(e['bits'])}
  assert err[0]['inner']['assoc_type']['type']=={'primitive':'never'}
  feature='serde'if mode=='all-features'else'default';mir=list((w/feature/'mir').glob('*.'+e['wrapper']+'.*.Inline.after.mir'));assert len(mir)==1
  text=mir[0].read_text();assert 'inlined <rand_xoshiro::'+e['type']+' as rand_xoshiro::rand_core::Rng>::next_u'+str(e['bits'])in text
  rows.append({'configuration':mode,'typed_api':e['api'],'runtime_generic_arguments':[],'receiver':'&mut rand_xoshiro::'+e['type'],'return':'u'+str(e['bits']),'callable_origin':'external_blanket_specialization','not_a_local_declaration':True,'rustdoc_json':str(raw),'rustdoc_sha256':sha(raw.read_bytes()),'trait_path':j['paths'][str(im['trait']['id'])],'impl_item':v,'method_item':meth[0],'local_tryrng_impl':own[0],'local_error_associated_type':err[0],'infallible_rustdoc_normalization':'primitive never; source explicitly Infallible, not an arbitrary associated-type relaxation','selector':e['wrapper'],'selector_source_sha256':sha((w/'src/lib.rs').read_bytes()),'mir':str(mir[0]),'mir_sha256':sha(mir[0].read_bytes())})
rootfiles={str(f.relative_to(p)):sha(f.read_bytes())for f in sorted((p/'src').rglob('*.rs'))};rootfiles['Cargo.toml']=sha((p/'Cargo.toml').read_bytes());rootfiles['Cargo.lock']=sha((p/'Cargo.lock').read_bytes())
(out/'existing-observations.json').write_text(json.dumps({'status':'read-only observations; registration schema proposal only','claim_count':25,'configurations':['default','serde'],'target':'x86_64-unknown-linux-gnu','dependency':{'crate':'rand_core','version':'0.10.1','archive_sha256':sha(archive.read_bytes()),'all_archive_file_hashes':files,'lib_rs_sha256':sha(source),'trait_lines':[49,63],'blanket_impl_lines':[65,89]},'original_target_source_hashes':rootfiles,'entries':rows,'coexistence_with_local44':'not compiled or audited together; remains an explicit integration obligation'},indent=2))
print('Captured 50 exact typed compiler/rustdoc/MIR observations, 25 methods x 2 configurations. No compiler/solver/API call.')
