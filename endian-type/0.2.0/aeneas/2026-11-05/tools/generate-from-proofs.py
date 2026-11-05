from pathlib import Path
import json
r=Path(__file__).resolve().parents[1];p=r/'proofs';manifest=json.loads((p/'EndianVerificationDriver/translation.json').read_text());functions={f['rust_name']:f for f in manifest['functions']}
lines=['import ModelProofs','open Aeneas Aeneas.Std Result','open ModelProofs','namespace FromProofs'];bindings=[]
for ty in ['u16','u32','u64','u128','usize','i16','i32','i64','i128','isize']:
 T=ty[0].upper()+ty[1:]
 families=[('be_from',ty,f'endian_type::BigEndian<{ty}>','to_be'),('le_from',ty,f'endian_type::LittleEndian<{ty}>','to_le'),('be_into',f'endian_type::BigEndian<{ty}>',ty,'from_be'),('le_into',f'endian_type::LittleEndian<{ty}>',ty,'from_le'),('le_to_be',f'endian_type::LittleEndian<{ty}>',f'endian_type::BigEndian<{ty}>','swap_bytes'),('be_to_le',f'endian_type::BigEndian<{ty}>',f'endian_type::LittleEndian<{ty}>','swap_bytes')]
 for label,source,target,op in families:
  rust=f'endian_type::{{impl core::convert::From<{source}> for {target}}}::from';f=functions[rust];lean=f['lean_name'];name=label+'_'+ty+'_spec'
  sized=ty in ['usize','isize'];extra=' (h64 : System.Platform.numBits = 64)' if sized else ''
  if op in ['to_le','from_le']:expected='x'
  elif sized:expected=f'reverse{T} x'
  else:expected=f'(⟨byteReverse x.bv⟩ : {T})'
  model=ty+'_'+('swap' if op=='swap_bytes' else op)+'_model'
  call=model+' x'+(' h64' if sized and op not in ['to_le','from_le'] else '')
  lines += [f'/-- Original Rust body: {rust}. -/',f'theorem {name} (x : {T}){extra} :',f'    {lean} x = .ok ({expected}) := by',f'  simp only [{lean}, {call}, bind_ok]','']
  bindings.append({'rust_api_identity':f'<{target} as core::convert::From<{source}>>::from','original_rust_name':rust,'theorem':'FromProofs.'+name,'lean_function':lean,'input_rust_type':source,'output_rust_type':target,'primitive':ty,'model_operation':op,'is_original_body':True,'is_local_driver':False,'precondition':'x86_64 little-endian; all primitive input bit patterns; '+('System.Platform.numBits=64 explicitly required in Lean.' if sized else 'fixed-width scalar types.'),'source':f['source']})
lines+=['end FromProofs']+['#print axioms '+x['theorem'] for x in bindings]
(p/'FromProofs.lean').write_text('\n'.join(lines)+'\n');(r/'evidence/integer-models/from-proof-bindings.json').write_text(json.dumps(bindings,indent=2)+'\n')

assert len(bindings)==60
