from pathlib import Path
import json,hashlib,collections
p=Path(__file__).resolve().parent;artifact=p.parent/'candidate-array-bits';path=artifact/'evidence/bit_field.llbc';raw=path.read_bytes();d=json.loads(raw);x=d['translated'];refs=[];globals_used=[];operations=collections.Counter();bad=[]
def walk(v,path):
 if isinstance(v,dict):
  for k,w in v.items():
   if k in ['Index','Subslice','Repeat','ArrayToSlice','Unsize','Array','Field']:operations[k]+=1
   if k=='Fun' and isinstance(w,int):
    refs.append({'id':w,'path':path})
    if w>=len(x['fun_decls']) or x['fun_decls'][w] is None:bad.append({'kind':'Fun','id':w,'path':path})
   if k=='Global' and isinstance(w,dict) and 'id'in w:
    globals_used.append({'id':w['id'],'path':path})
    if w['id']>=len(x['global_decls']) or x['global_decls'][w['id']] is None:bad.append({'kind':'Global','id':w['id'],'path':path})
   walk(w,path+'/'+k)
 elif isinstance(v,list):
  for i,w in enumerate(v):walk(w,path+'/'+str(i))
for i,f in enumerate(x['fun_decls']):
 if f:walk(f['body'],'fun'+str(i))
for i,g in enumerate(x['global_decls']):
 if g:walk(g['value'],'global'+str(i))
assert not bad and not operations
assert {r['id'] for r in refs}==set(range(36,60))
assert {r['id'] for r in globals_used}==set(range(12))
array=json.loads((artifact/'evidence/array/bit_field.llbc').read_text())['translated']
result={'primitive_llbc_sha256':hashlib.sha256(raw).hexdigest(),'primitive_has_errors':d['has_errors'],'primitive_options':{k:x['options'][k] for k in ['preset','monomorphize','ops_to_function_calls','index_to_function_calls','no_gen_tuple_structs','no_typecheck']},'configuration_is_officially_unsupported':True,'all_executable_body_and_initializer_function_references':refs,'all_executable_body_global_references':globals_used,'dangling_executable_function_or_global_references':bad,'actual_index_array_repeat_unsize_field_operation_counts':dict(operations),'retained_names_for_removed_function_ids':list(range(11)),'retained_name_table_entries_are_not_actual_body_calls':True,'array_options':{k:array['options'][k] for k in ['preset','monomorphize','ops_to_function_calls','index_to_function_calls','no_gen_tuple_structs']},'interpretation':'No dangling executable function/global reference or ops/index lowering product was detected in these selected primitive bodies. This is a structural audit, not proof of support for the globally forbidden option combination or proof of Charon correctness. Existing Lean theorems remain kernel-valid; the Rust-extraction boundary requires disclosure or a supported replacement.'}
(p/'results.json').write_text(json.dumps(result,indent=2)+'\n');print('Primitive actual refs resolve; no executable ops/index lowering products; unsupported configuration remains an extraction limitation')
