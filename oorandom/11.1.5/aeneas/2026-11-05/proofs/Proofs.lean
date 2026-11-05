import Oorandom
open Aeneas Aeneas.Std Result
namespace Proofs
/-- PCG-XSH-RR output permutation, using logical 64-bit shifts, low 32 bits,
    and a 32-bit rotate. -/
def pcgOutput (s : U64) : U32 :=
  let x : U64 := ⟨((s.bv >>> 18) ^^^ s.bv) >>> 27⟩
  let r : U64 := ⟨s.bv >>> 59⟩
  (UScalar.cast .U32 x).rotate_right (UScalar.cast .U32 r)
def pcgState (s i : U64) : U64 :=
  (s.wrapping_mul 6364136223846793005#u64).wrapping_add i

theorem rand_u32_spec (s : oorandom.Rand32) :
    oorandom.Rand32.rand_u32 s =
      .ok (pcgOutput s.state, { state := pcgState s.state s.inc, inc := s.inc }) := by
  simp only [oorandom.Rand32.rand_u32, oorandom.Rand32.MULTIPLIER,
    HShiftRight.hShiftRight, UScalar.shiftRight_IScalar, UScalar.shiftRight,
    pcgOutput, pcgState, U64.wrapping_mul, U64.wrapping_add,
    core.num.U64.wrapping_mul, core.num.U64.wrapping_add, core.num.U32.rotate_right,
    UScalar.xor]
  dsimp [IScalar.toNat, lift, Bind.bind, HXor.hXor, UScalar.xor,
    UScalar.wrapping_mul, UScalar.wrapping_add]
  simp only [bind_ok]
  rfl
end Proofs
#print axioms Proofs.rand_u32_spec
