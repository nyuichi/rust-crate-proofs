from pathlib import Path
import re
p=Path(__file__).resolve().parents[1]
f=p/'proofs/BitField/FunsExternal_Template.lean';s=f.read_text()
types=['U8','U16','U32','U64','U128','Usize','I8','I16','I32','I64','I128','Isize']
assert set(re.findall(r'axiom core.mem.size_of(\w+) : Result Std.Usize',s))==set(types)
assert len(re.findall(r'\baxiom\b',s))==12
for ty in types:
 bits=64 if ty.lower().endswith('size') else int(ty[1:])
 s=s.replace(f'axiom core.mem.size_of{ty} : Result Std.Usize',f'def core.mem.size_of{ty} : Result Std.Usize := .ok {bits//8}#usize')
assert 'axiom ' not in s
(p/'proofs/BitField/FunsExternal.lean').write_text(s)
print('Provided twelve separately monomorphized integer layout models for x86_64')
