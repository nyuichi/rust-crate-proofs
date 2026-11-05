import RandXoshiro
open Aeneas Aeneas.Std Result
namespace Proofs
def state4 (a b c d : U32) : rand_xoshiro.xoshiro128plus.Xoshiro128Plus :=
  { s := Array.from [a,b,c,d] (by simp) }
def xoshiroOutput (a d : U32) : U32 := a.wrapping_add d
def xoshiroState (a b c d : U32) : rand_xoshiro.xoshiro128plus.Xoshiro128Plus :=
  let c' := c ^^^ a
  let d' := d ^^^ b
  state4 (a ^^^ d') (b ^^^ c') (c' ^^^ (⟨b.bv <<< 9⟩ : U32)) (d'.rotate_left 11#u32)
theorem xoshiro_next_spec (a b c d : U32) :
  rand_xoshiro.xoshiro128plus.Xoshiro128Plus.Insts.Rand_coreTryRngNever.try_next_u32 (state4 a b c d) =
    .ok (.Ok (xoshiroOutput a d), xoshiroState a b c d) := by
  simp only [rand_xoshiro.xoshiro128plus.Xoshiro128Plus.Insts.Rand_coreTryRngNever.try_next_u32,
    state4, xoshiroOutput, xoshiroState, Array.index_usize, Array.update]
  simp [lift, core.num.U32.wrapping_add, U32.wrapping_add,
    HShiftLeft.hShiftLeft, UScalar.shiftLeft_IScalar, UScalar.shiftLeft, IScalar.toNat,
    core.num.U32.rotate_left, -BitVec.shiftLeft_eq]
/-- Every valid four-word state has the input representation used above. -/
theorem state4_surjective (s : rand_xoshiro.xoshiro128plus.Xoshiro128Plus) :
    ∃ a b c d, state4 a b c d = s := by
  have hlen : s.s.val.length = 4 := by simpa using s.s.property
  obtain ⟨a,b,c,d,h⟩ := List.length_eq_four.mp hlen
  refine ⟨a,b,c,d,?_⟩
  cases s with
  | mk arr =>
    unfold state4
    congr 1
    apply Aeneas.Std.Array.ext
    simpa using h.symm
end Proofs
#print axioms Proofs.xoshiro_next_spec
#print axioms Proofs.state4_surjective
