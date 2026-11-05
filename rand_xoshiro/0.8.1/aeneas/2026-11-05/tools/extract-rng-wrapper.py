from pathlib import Path
import os,json,subprocess,hashlib,re,tomllib
root=Path(__file__).resolve().parents[1];wrapper=root/'evidence/rng-selection-wrapper'
entries=json.loads((wrapper/'api-bindings.json').read_text())
expected='#![no_std]\n'+'\n'.join('pub fn '+e['wrapper']+'(s:&mut rand_xoshiro::'+e['type']+')->u'+str(e['bits'])+'{<rand_xoshiro::'+e['type']+' as rand_xoshiro::rand_core::Rng>::next_u'+str(e['bits'])+'(s)}' for e in entries)+'\n'
assert (wrapper/'src/lib.rs').read_text()==expected,'Selector source is not the exact original-API forwarding body'
manifest=tomllib.loads((root/'Cargo.toml').read_text());assert (manifest['package']['name'],manifest['package']['version'])==('rand_xoshiro','0.8.1')
selector=tomllib.loads((wrapper/'Cargo.toml').read_text());assert (wrapper/selector['dependencies']['rand_xoshiro']['path']).resolve()==root
lock=tomllib.loads((wrapper/'Cargo.lock').read_text()); dep=next(p for p in lock['package'] if p['name']=='rand_core')
assert dep['version']=='0.10.1' and dep['checksum']==hashlib.sha256((wrapper/'rand_core-0.10.1.crate').read_bytes()).hexdigest()
os.chdir(wrapper)
env=dict(os.environ,CARGO_NET_OFFLINE='true',RUSTFLAGS='--remap-path-prefix='+str(root)+'=upstream --remap-path-prefix='+str(wrapper)+'=selection')
commands=[]
for feature in ['default','serde']:
 out=wrapper/feature;out.mkdir(exist_ok=True)
 args=['charon','cargo','--preset','aeneas','--sysroot','default','--include','rand_xoshiro','--duplicate-defaulted-methods','--remove-unused-self-clauses','--mir','optimized','--rustc-arg=-Zmir-opt-level=4','--rustc-arg=-Zmir-enable-passes=+Inline','--rustc-arg=-Zdump-mir=Inline','--rustc-arg=-Zdump-mir-dir='+str(out/'mir'),'--dest-file',str(out/'xoshiro_rng_api_roots.llbc')]
 if feature=='serde':args+=['--','--features','serde']
 with (out/'charon.log').open('w') as log:subprocess.run(args,check=True,env=env,stdout=log,stderr=subprocess.STDOUT)
 with (out/'aeneas.log').open('w') as log:subprocess.run(['aeneas','-backend','lean','-emit-json','-split-files','-abort-on-error','-dest',str(out/'generated'),str(out/'xoshiro_rng_api_roots.llbc')],check=True,stdout=log,stderr=subprocess.STDOUT)
 j=json.loads((out/'xoshiro_rng_api_roots.llbc').read_text());assert not j['has_errors'];assert j['translated']['target_information'][0]['key']=='x86_64-unknown-linux-gnu'
 texts='\n'.join(p.read_text() for p in (out/'generated').glob('*.lean'));assert not re.search(r'\b(sorry|axiom)\b',texts),'Unknown external in selected program'
 for e in entries:
  mir=list((out/'mir').glob('*.'+e['wrapper']+'.*.Inline.after.mir'));assert len(mir)==1,(e,mir)
  text=mir[0].read_text();needle='inlined <rand_xoshiro::'+e['type']+' as rand_xoshiro::rand_core::Rng>::next_u'+str(e['bits'])
  assert needle in text,(e,text)
 commands.append({'feature':feature,'charon_command':args,'selector_source_sha256':hashlib.sha256(expected.encode()).hexdigest(),'guard_result':'exact original-API call, typed receiver/return and compiler inlined scopes confirmed'})
for name in ['Types.lean','Funs.lean']:
 a=(wrapper/'default/generated'/name).read_bytes();b=(wrapper/'serde/generated'/name).read_bytes();assert a==b,'Feature-specific selected program differs'
 (root/'proofs/XoshiroRngApiRoots'/name).write_bytes(a)
(root/'proofs/XoshiroRngApiRoots/translation.json').write_bytes((wrapper/'default/generated/translation.json').read_bytes())
(wrapper/'compiler-provenance.json').write_text(json.dumps(commands,indent=2))
print('All twenty-five external Rng original-API forwarding selectors: identity/MIR/target/feature/no-external guards passed')
