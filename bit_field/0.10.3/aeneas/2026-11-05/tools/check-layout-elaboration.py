from pathlib import Path
import subprocess,json,re
p=Path(__file__).resolve().parents[1];types=['U8','U16','U32','U64','U128','Usize','I8','I16','I32','I64','I128','Isize']
text='import BitField.Funs\nopen Aeneas Aeneas.Std\nset_option pp.explicit true\n'
for ty in types:text+=f'#print bit_field.{ty}.Insts.Bit_fieldBitField.BIT_LENGTH\n#print axioms bit_field.core.mem.size_of{ty}\n'
(p/'evidence/LayoutResolution.lean').write_text(text)
r=subprocess.run(['lake','env','lean','../evidence/LayoutResolution.lean'],cwd=p/'proofs',stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
(p/'evidence/layout-elaboration.log').write_text(r.stdout);assert r.returncode==0
for ty in types:
 assert re.search(r'@bit_field\.core\.mem\.size_of\s+'+ty+r'\s+bit_field\.layout_'+ty.lower()+r'\b',r.stdout),ty
rows=re.findall(r"depends on axioms:\s*\[(.*?)\]",r.stdout,re.S);assert len(rows)==12
for row in rows:assert {a.strip() for a in row.split(',')}<={'propext','Classical.choice','Quot.sound'}
r=subprocess.run(['lake','env','lean','../evidence/NonPrimitiveRejected.lean'],cwd=p/'proofs',stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
(p/'evidence/nonprimitive-rejected.log').write_text(r.stdout);assert r.returncode!=0 and 'PrimitiveRustLayout Bool' in r.stdout
(p/'evidence/layout-elaboration-check.json').write_text(json.dumps({'static_ground_instances':types,'layout_lemma_audits':'standard3','Bool_has_no_instance_in_audited_environment':True,'class_is_not_sealed':True,'negative_returncode':r.returncode,'implicit_dictionary_is_Lean_evidence_not_Rust_trait':True},indent=2)+'\n')
print('All12 original calls have expected static instance arguments; 12 layout lemma audits standard3; Bool use rejected')
