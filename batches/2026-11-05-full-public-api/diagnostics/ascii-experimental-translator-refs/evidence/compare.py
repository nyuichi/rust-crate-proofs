from pathlib import Path
import subprocess,hashlib,json,re
b=Path('/workspace/full-api/bits/ascii-extra');p=b/'from-field-two/translator-experiment';old='/workspace/aeneas-tools/aeneas/bin/aeneas';new='/tmp/ascii-rust-type-optin-build/src/_build/default/main.exe'
cases=[('ufcs',b/'from-field-two/ufcs-selector/evidence/ascii_field_selectors.llbc',[]),('own49',b/'candidate-next-three/evidence/ascii.llbc',['-filter-trait-methods']),('generic',b/'generic-char-forward/evidence/ascii.llbc',['-filter-trait-methods']),('ufcs-split',b/'from-field-two/ufcs-selector/evidence/ascii_field_selectors.llbc',['-split-files'])];results=[]
for name,llbc,flags in cases:
 out=[]
 for version,bin in [('old-default',old),('new-default',new)]:
  dest=p/version/name;dest.mkdir(parents=True,exist_ok=True)
  with (p/'evidence'/(name+'-'+version+'.log')).open('w') as log:subprocess.run([bin,'-backend','lean','-emit-json','-abort-on-error','-dest',str(dest),*flags,str(llbc)],check=True,stdout=log,stderr=subprocess.STDOUT)
  out.append({str(f.relative_to(dest)):hashlib.sha256(f.read_bytes()).hexdigest() for f in dest.rglob('*') if f.is_file()})
 assert {k:v for k,v in out[0].items() if k!='translation.json'}=={k:v for k,v in out[1].items() if k!='translation.json'},name
 oldmeta=json.loads((p/'old-default'/name/'translation.json').read_text());newmeta=json.loads((p/'new-default'/name/'translation.json').read_text());versions={'old':oldmeta.pop('aeneas_version'),'new':newmeta.pop('aeneas_version')};assert oldmeta==newmeta,name
 results.append({'case':name,'Lean_outputs_byte_identical':True,'all_files_byte_identical':False,'raw_files_old':out[0],'raw_files_new':out[1],'only_metadata_difference':{'aeneas_version':versions},'input_llbc_sha256':hashlib.sha256(llbc.read_bytes()).hexdigest()})
llbc=cases[0][1]
with (p/'evidence/ufcs-optin.log').open('w') as log:subprocess.run([new,'-backend','lean','-emit-json','-abort-on-error','-no-register-external-rust-types','-dest',str(p/'optin'),str(llbc)],check=True,stdout=log,stderr=subprocess.STDOUT)
a=(p/'new-default/ufcs/AsciiFieldSelectors.lean').read_text();on=(p/'optin/AsciiFieldSelectors.lean').read_text();expected=a.replace('@[discriminant u8, rust_type "ascii::ascii_char::AsciiChar"]','@[discriminant u8]').replace('@[rust_type "ascii::ascii_str::AsciiStr"]\n','');assert expected==on and a.count('rust_type')==2 and 'rust_type' not in on
assert (p/'new-default/ufcs/translation.json').read_bytes()==(p/'optin/translation.json').read_bytes();assert on.count('rust_fun')==2 and '@[discriminant u8]' in on
rust=(b/'source/src/ascii_char.rs').read_text().split('pub enum AsciiChar {',1)[1].split('\n}',1)[0];mapping=[(n,int(v)) for n,v in re.findall(r'^\s*(\w+) = (\d+),',rust,re.M)];lean=re.findall(r'^\| (\w+) : ascii.ascii_char.AsciiChar$',on,re.M);assert len(mapping)==128 and [n for n,v in mapping]==lean and [v for n,v in mapping]==list(range(128))
(p/'evidence/constructor-discriminant-mapping.json').write_text(json.dumps({'rust_repr':'u8','generated_discriminant_attribute':'discriminant u8 ordinal','all128_exact':True,'mapping':[{'rust':n,'lean':n,'discriminant':v} for n,v in mapping]},indent=2)+'\n')
(p/'evidence/comparison.json').write_text(json.dumps({'defaults_off_regressions':results,'optin_change':'Exactly external rust_type attributes for AsciiChar and AsciiStr omitted; entire remaining Lean text byte-identical','translation_metadata_identical':True,'constructors_discriminants_exact128':True,'no_models_added':True,'newbinary_sha256':hashlib.sha256(Path(new).read_bytes()).hexdigest(),'oldbinary_sha256':hashlib.sha256(Path(old).read_bytes()).hexdigest()},indent=2)+'\n');print('default4 Lean exact/version-only metadata delta, optin attr-only, exact128 mapping passed')
