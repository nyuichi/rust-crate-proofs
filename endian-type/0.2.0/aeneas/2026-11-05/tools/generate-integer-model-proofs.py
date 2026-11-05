from pathlib import Path
import re
r=Path(__file__).resolve().parents[1];p=r/'proofs'
text=(p/'ModelProofs.lean').read_text();prefix=text[:text.index('theorem u16_swap_model')].rstrip();base=(r/'tools/fixed-swap-proof-template.txt').read_text().strip();assert base.count('theorem ')==1
# Keep the independently structured array->bitvector proof as the common leaf.
(r/'tools/fixed-swap-proof-template.txt').write_text(base)
lines=[prefix];names=[]
for t,n in [('U16',16),('U32',32),('U64',64),('U128',128),('I16',16),('I32',32),('I64',64),('I128',128)]:
 block=base.replace('U16',t).replace('u16',t.lower());block=re.sub(r'\b16\b',str(n),block);lines.append(block);names.append(t.lower()+'_swap_model')
 for op in ['to_be','from_be']:
  name=t.lower()+'_'+op+'_model';lines.append(f'''theorem {name} (x : {t}) :
    core.num.{t}.{op} x = .ok (⟨byteReverse x.bv⟩ : {t}) := by
  simpa only [core.num.{t}.{op}, core.num.{t}.swap_bytes] using {t.lower()}_swap_model x
''');names.append(name)
 for op in ['to_le','from_le']:
  name=t.lower()+'_'+op+'_model';lines.append(f'theorem {name} (x : {t}) : core.num.{t}.{op} x = .ok x := by rfl\n');names.append(name)
for t,baseT,cast in [('Usize','U64','UScalar'),('Isize','I64','IScalar')]:
 rev='reverse'+t;lines.append(f'''/-- 64-bit pointer-sized bit-pattern interpretation; public theorems require h64. -/
def {rev} (x : {t}) : {t} :=
  {cast}.cast .{t} (⟨byteReverse ({cast}.cast .{baseT} x).bv⟩ : {baseT})

theorem {t.lower()}_swap_model (x : {t}) (h64 : System.Platform.numBits = 64) :
    core.num.{t}.swap_bytes x = .ok ({rev} x) := by
  simp only [core.num.{t}.swap_bytes, h64, ↓reduceIte]
  have hv : core.num.{baseT}.from_le_bytes (core.num.{baseT}.to_be_bytes ({cast}.cast .{baseT} x)) =
      (⟨byteReverse ({cast}.cast .{baseT} x).bv⟩ : {baseT}) :=
    Result.ok_injective ({baseT.lower()}_swap_model ({cast}.cast .{baseT} x))
  exact congrArg (fun v => (.ok ({cast}.cast .{t} v) : Result {t})) hv
''');names.append(t.lower()+'_swap_model')
 for op in ['to_be','from_be']:
  name=t.lower()+'_'+op+'_model';lines.append(f'''theorem {name} (x : {t}) (h64 : System.Platform.numBits = 64) :
    core.num.{t}.{op} x = .ok ({rev} x) := by
  simpa only [core.num.{t}.{op}, core.num.{t}.swap_bytes] using {t.lower()}_swap_model x h64
''');names.append(name)
 for op in ['to_le','from_le']:
  name=t.lower()+'_'+op+'_model';lines.append(f'theorem {name} (x : {t}) : core.num.{t}.{op} x = .ok x := by rfl\n');names.append(name)
lines+=['end ModelProofs']+['#print axioms ModelProofs.'+name for name in names]
(p/'ModelProofs.lean').write_text('\n'.join(lines)+'\n')
assert len(names)==50
