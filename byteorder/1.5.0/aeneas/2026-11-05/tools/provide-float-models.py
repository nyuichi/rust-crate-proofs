from pathlib import Path
import re
p=Path(__file__).resolve().parents[1]
f=p/'proofs/ByteorderFloatRoots/FunsExternal_Template.lean';s=f.read_text()
externs=re.findall(r'axiom ([\w.]+) : F(32|64) → Result Std.U(32|64)',s)
assert set(externs)=={('core.f32.F32.to_bits','32','32'),('core.f64.F64.to_bits','64','64')},externs
assert s.count('axiom ')==2
prefix=''
for bits in [32,64]:
 prefix+=f'/-- IEEE bit-pattern representation only; no arithmetic model. -/\nabbrev F{bits} := BitVec {bits}\ndef F{bits}.toWord (n : F{bits}) : U{bits} := ⟨n.cast (by simp)⟩\ntheorem F{bits}.toWord_val (n : F{bits}) : (F{bits}.toWord n).val = n.toNat := by\n  simp only [F{bits}.toWord, UScalar.val, BitVec.toNat_cast]\n\n'
at=s.index('/-- [core::');s=s[:at]+prefix+s[at:]
for bits in [32,64]:s=s.replace(f'axiom core.f{bits}.F{bits}.to_bits : F{bits} → Result Std.U{bits}',f'def core.f{bits}.F{bits}.to_bits (n : F{bits}) : Result Std.U{bits} := .ok (F{bits}.toWord n)')
assert 'axiom ' not in s
(p/'proofs/ByteorderFloatRoots/FunsExternal.lean').write_text(s)
print('Only the two std to_bits externs and F32/F64 bit-pattern types are modeled')
