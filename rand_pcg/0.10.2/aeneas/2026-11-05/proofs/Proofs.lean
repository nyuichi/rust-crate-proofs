import RandPcg
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

theorem try_next_u32_spec (s : rand_pcg.pcg64.Lcg64Xsh32) :
    rand_pcg.pcg64.Lcg64Xsh32.Insts.Rand_coreTryRngNever.try_next_u32 s =
      .ok (.Ok (pcgOutput s.state),
        { state := pcgState s.state s.increment, increment := s.increment }) := by
  simp only [rand_pcg.pcg64.Lcg64Xsh32.Insts.Rand_coreTryRngNever.try_next_u32,
    rand_pcg.pcg64.Lcg64Xsh32.step, rand_pcg.pcg64.MULTIPLIER,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.ROTATE,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.XSHIFT,
    rand_pcg.pcg64.TryRngLcg64Xsh32Never.try_next_u32.SPARE,
    HShiftRight.hShiftRight, UScalar.shiftRight_UScalar, UScalar.shiftRight,
    pcgOutput, pcgState, U64.wrapping_mul, U64.wrapping_add,
    core.num.U64.wrapping_mul, core.num.U64.wrapping_add, core.num.U32.rotate_right]
  dsimp [lift, Bind.bind, HXor.hXor, UScalar.xor,
    UScalar.wrapping_mul, UScalar.wrapping_add]
  simp only [bind_ok]
  rfl
end Proofs
#print axioms Proofs.try_next_u32_spec
