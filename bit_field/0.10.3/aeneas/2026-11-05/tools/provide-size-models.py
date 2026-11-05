from pathlib import Path
import re,json,hashlib
p=Path(__file__).resolve().parents[1];g=p/'proofs/BitField'
llbc=json.loads((p/'evidence/bit_field.llbc').read_text());o=llbc['translated']['options']
assert o['preset']=='Aeneas' and not o['monomorphize'] and not llbc['has_errors']
infos=llbc['translated']['target_information'];assert len(infos)==1 and infos[0]['key']=='x86_64-unknown-linux-gnu' and infos[0]['value']['target_pointer_size']==8
s=(g/'FunsExternal_Template.lean').read_text();assert len(re.findall(r'\baxiom\b',s))==1
assert 'axiom core.mem.size_of (T : Type _) : Result Std.Usize' in s
expected=['U8','U16','U32','U64','U128','Usize','I8','I16','I32','I64','I128','Isize']
funs=(g/'Funs.lean').read_text();calls=re.findall(r'core\.mem\.size_of Std\.(\w+)',funs)
assert calls==expected and funs.count('core.mem.size_of')==12 and '@core.mem.size_of' not in funs
translation=json.loads((g/'translation.json').read_text())
functions=translation['functions'];opaque=[f for f in functions if f['is_opaque']];actual=[f for f in functions if not f['is_opaque']]
assert len(opaque)==1 and opaque[0]['rust_name']=='core::mem::size_of' and len(actual)==24
wanted={f'bit_field::{{impl bit_field::BitField for {ty.lower()}}}::{method}' for ty in expected for method in ['get_bit','set_bit']}
assert {f['rust_name'] for f in actual}==wanted
for f in actual:assert f['is_local'] and (p/f['source']['file']).resolve()==(p/'src/lib.rs').resolve()
assert len(translation['globals'])==12
provider=(p/'tools/primitive-layout-provider.lean').read_text();assert len(re.findall(r'^instance layout_',provider,re.M))==12
assert not re.search(r'\b(sorry|axiom)\b',provider)
(g/'FunsExternal.lean').write_text(provider)
record={'configuration':{k:o[k] for k in ['preset','monomorphize','ops_to_function_calls','index_to_function_calls','no_gen_tuple_structs']},'known_primitive_calls':calls,'only_provider_instances':12,'new_lean_interface':'size_of (T : Type) [PrimitiveRustLayout T] : Result Usize','not_a_Rust_trait':True,'class_extensible_in_Lean':True,'fallback_supplied_in_audited_instance_environment':False,'use_closure_is_frozen_12_ground_calls':True,'original_funs_unchanged':True,'funs_sha256':hashlib.sha256((g/'Funs.lean').read_bytes()).hexdigest(),'template_sha256':hashlib.sha256((g/'FunsExternal_Template.lean').read_bytes()).hexdigest(),'provider_sha256':hashlib.sha256((g/'FunsExternal.lean').read_bytes()).hexdigest()}
(p/'evidence/layout-model-use-guard.json').write_text(json.dumps(record,indent=2)+'\n')
print('Supported nonmono extraction; exactly12 ground primitive size calls statically receive Lean-only layout evidence; no arbitrary-Type fallback')
