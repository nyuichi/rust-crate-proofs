#!/usr/bin/env python3
"""Fill only the three requested std wrapping operator model declarations."""
from pathlib import Path
import re
root=Path(__file__).resolve().parent.parent
p=root/'proofs/RandXorshift/FunsExternal_Template.lean'
s=p.read_text()
models={
'core.num.wrapping.WrappingU32.Insts.CoreOpsBitShlUsizeWrappingU32.shl': '''(x : core.num.wrapping.Wrapping Std.U32) (s : Std.Usize) :
  Result (core.num.wrapping.Wrapping Std.U32) :=
  .ok (U32.wrapping_shl x ⟨BitVec.ofNat 32 s.val⟩)''',
'core.num.wrapping.WrappingU32.Insts.CoreOpsBitShrUsizeWrappingU32.shr': '''(x : core.num.wrapping.Wrapping Std.U32) (s : Std.Usize) :
  Result (core.num.wrapping.Wrapping Std.U32) :=
  .ok (U32.wrapping_shr x ⟨BitVec.ofNat 32 s.val⟩)''',
'core.num.wrapping.WrappingU32.Insts.CoreOpsBitBitXorWrappingU32WrappingU32.bitxor': '''(x y : core.num.wrapping.Wrapping Std.U32) :
  Result (core.num.wrapping.Wrapping Std.U32) := .ok (x ^^^ y)'''}
for name,body in models.items():
 pattern=r'axiom\s+'+re.escape(name)+r'\s+:.*?(?=\n\n/\*\*|\n\n/--|\Z)'
 s,n=re.subn(pattern,'def '+name+'\n  '+body,s,flags=re.S)
 assert n==1,(name,n)
assert not re.search(r'\b(?:axiom|sorry)\b',s)
(root/'proofs/RandXorshift/FunsExternal.lean').write_text(s)
