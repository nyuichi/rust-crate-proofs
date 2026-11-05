import Proofs
open Aeneas Aeneas.Std
open rand_xorshift
namespace Nonzero
 theorem right_xor_zero (t : BitVec 32) (h : t ^^^ (t >>> 8) = 0) : t = 0 := by
   have h8 : t = t >>> 8 := BitVec.xor_eq_zero_iff.mp h
   have h16 : t = t >>> 16 := by
     calc t = t >>> 8 := h8
          _ = (t >>> 8) >>> 8 := congrArg (fun v : BitVec 32 => v >>> 8) h8
          _ = t >>> 16 := (BitVec.shiftRight_add t 8 8).symm
   calc t = t >>> 16 := h16
        _ = (t >>> 16) >>> 16 := congrArg (fun v : BitVec 32 => v >>> 16) h16
        _ = t >>> 32 := (BitVec.shiftRight_add t 16 16).symm
        _ = 0 := BitVec.ushiftRight_eq_zero (by decide)

 theorem left_xor_zero (x : BitVec 32) (h : x ^^^ (x <<< 11) = 0) : x = 0 := by
   have h11 : x = x <<< 11 := BitVec.xor_eq_zero_iff.mp h
   have h22 : x = x <<< 22 := by
     calc x = x <<< 11 := h11
          _ = (x <<< 11) <<< 11 := congrArg (fun v : BitVec 32 => v <<< 11) h11
          _ = x <<< 22 := (BitVec.shiftLeft_add x 11 11).symm
   calc x = x <<< 11 := h11
        _ = (x <<< 22) <<< 11 := congrArg (fun v : BitVec 32 => v <<< 11) h22
        _ = x <<< 33 := (BitVec.shiftLeft_add x 22 11).symm
        _ = 0 := BitVec.shiftLeft_eq_zero (by decide)

 theorem next_model_nonzero (x y z w : BitVec 32)
    (h : x ≠ 0 ∨ y ≠ 0 ∨ z ≠ 0 ∨ w ≠ 0) :
    y ≠ 0 ∨ z ≠ 0 ∨ w ≠ 0 ∨ Proofs.nextWord x w ≠ 0 := by
   by_contra hn
   push_neg at hn
   rcases hn with ⟨hy,hz,hw,hr⟩
   subst y
   subst z
   subst w
   have hx : x ≠ 0 := by simpa using h
   apply hx
   apply left_xor_zero
   apply right_xor_zero
   simpa [Proofs.nextWord] using hr

 def stateNonzero (s : XorShiftRng) : Prop :=
   s.x.bv ≠ 0 ∨ s.y.bv ≠ 0 ∨ s.z.bv ≠ 0 ∨ s.w.bv ≠ 0
 theorem original_next_preserves_nonzero (s : XorShiftRng) (h : stateNonzero s) :
   ∃ word state, XorShiftRng.Insts.Rand_coreTryRngNever.try_next_u32 s =
       .ok (.Ok word,state) ∧ stateNonzero state := by
   refine ⟨_,_,Proofs.try_next_u32_spec s,?_⟩
   exact next_model_nonzero s.x.bv s.y.bv s.z.bv s.w.bv h
end Nonzero
#print axioms Nonzero.next_model_nonzero
#print axioms Nonzero.original_next_preserves_nonzero
