import ReflexiveProofs
import RandXorshift.Funs

open Aeneas Aeneas.Std Result
open rand_xorshift

namespace Proofs

/-- Marsaglia's xorshift128 transition over four 32-bit words. -/
def nextWord (x w : BitVec 32) : BitVec 32 :=
  let t := x ^^^ (x <<< 11)
  w ^^^ (w >>> 19) ^^^ (t ^^^ (t >>> 8))

/-- The original next_u32 body returns the new fourth word and shifts the
    first three words, under the supplied standard-library wrapping models. -/
theorem try_next_u32_spec (s : XorShiftRng) :
    XorShiftRng.Insts.Rand_coreTryRngNever.try_next_u32 s =
      .ok (.Ok ⟨nextWord s.x.bv s.w.bv⟩,
        { x := s.y, y := s.z, z := s.w, w := ⟨nextWord s.x.bv s.w.bv⟩ }) := by
  simp [XorShiftRng.Insts.Rand_coreTryRngNever.try_next_u32,
    core.num.wrapping.WrappingU32.Insts.CoreOpsBitShlUsizeWrappingU32.shl,
    core.num.wrapping.WrappingU32.Insts.CoreOpsBitShrUsizeWrappingU32.shr,
    core.num.wrapping.WrappingU32.Insts.CoreOpsBitBitXorWrappingU32WrappingU32.bitxor,
    U32.wrapping_shl, U32.wrapping_shr, UScalar.wrapping_shl,
    UScalar.wrapping_shr, nextWord]
  have scalar_ext (a b : Std.U32) (h : a.bv = b.bv) : a = b := by
    cases a; cases b; cases h; rfl
  apply scalar_ext
  simp only [UScalar.bv_xor]
  rfl

end Proofs
#print axioms Proofs.try_next_u32_spec
