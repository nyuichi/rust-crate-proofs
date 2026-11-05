from pathlib import Path
import json,re,hashlib
p=Path(__file__).resolve().parents[1];g=p/'proofs/Byteorder'
llbc=json.loads((p/'evidence/native-identity/byteorder.llbc').read_text());o=llbc['translated']['options']
assert o['preset']=='Aeneas' and not o['monomorphize'] and not llbc['has_errors']
targets=llbc['translated']['target_information'];assert len(targets)==1 and targets[0]['key']=='x86_64-unknown-linux-gnu' and targets[0]['value']['is_little_endian']

rows=json.loads((g/'translation.json').read_text())['functions'];actual=[f for f in rows if not f['is_opaque']];opaque=[f for f in rows if f['is_opaque']]
assert len(actual)==4 and len(opaque)==5
expected={f'byteorder::{{impl byteorder::ByteOrder for byteorder::LittleEndian}}::from_slice_{sgn}{bits}' for sgn in ['u'] for bits in [16,32,64,128]}
assert {f['rust_name'] for f in actual}==expected
for f in actual:
 assert f['is_local'] and (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve(),f
num=[f for f in opaque if re.fullmatch(r'core::num::\{u(16|32|64|128)\}::to_le',f['rust_name'])]
other=[f for f in opaque if f not in num]
assert len(num)==4 and len(other)==1 and other[0]['rust_name'].startswith('core::slice::iter::') and other[0]['rust_name'].endswith('::into_iter')
s=(g/'Funs.lean').read_text();decls=list(re.finditer(r'^def ([^\s]+)',s,re.M));assert len(decls)==4
for m in decls:
 st=s.index(':= do',m.start());end=s.find('\n/--',st)
 if end<0:end=s.index('\nend byteorder',st)
 assert re.sub(r'\s+',' ',s[st:end]).strip()==':= do ok numbers',m.group(1)
for f in opaque:assert f['lean_name'] not in s
assert 'FunsExternal_Template' not in s
assert not re.search(r'\baxiom\b',s) and not re.search(r'\baxiom\b',(g/'Types.lean').read_text())
provider='module\npublic import Aeneas\npublic import Byteorder.Types\n/- No external symbols are supplied. All original native-endian bodies return their input unchanged.\n   The unchanged official external template records unused declarations from discarded target branches. -/\n'
(g/'FunsExternal.lean').write_text(provider)
record={'actual_bodies':4,'all_actual_bodies_exact_identity':True,'opaque_symbols_retained_in_unimported_official_template':[{k:f[k] for k in ['rust_name','lean_name']} for f in opaque],'external_references_in_actual_funs':0,'provider_definitions':0,'target':'x86_64-unknown-linux-gnu little endian/default features/debug','opposite_endian_and_other_targets_unproved':True,'funs_sha256':hashlib.sha256((g/'Funs.lean').read_bytes()).hexdigest(),'template_sha256':hashlib.sha256((g/'FunsExternal_Template.lean').read_bytes()).hexdigest(),'provider_sha256':hashlib.sha256((g/'FunsExternal.lean').read_bytes()).hexdigest()}
(p/'evidence/native-identity/use-guard.json').write_text(json.dumps(record,indent=2)+'\n')
print('Four actual original native bodies return their inputs; external use0/provider definitions0; official external template retained unimported')
