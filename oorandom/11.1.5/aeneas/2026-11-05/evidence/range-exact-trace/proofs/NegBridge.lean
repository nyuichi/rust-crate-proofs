import Oorandom.Funs
open Aeneas Aeneas.Std Result
open oorandom_range_original
namespace NegBridge
theorem neg_math {n : Nat} (x : BitVec n) :
 (-x).toNat = if x.toNat = 0 then 0 else 2^n - x.toNat := by
 by_cases hz : x.toNat = 0
 · simp [BitVec.toNat_neg, hz]
 · simp only [if_neg hz]
   apply BitVec.toNat_neg_of_pos
   change 0 < x.toNat
   omega
theorem u32_existing_sub (x : U32) : core.num.U32.wrapping_neg x = .ok (core.num.U32.wrapping_sub 0#u32 x) := by rfl
theorem u64_existing_sub (x : U64) : core.num.U64.wrapping_neg x = .ok (core.num.U64.wrapping_sub 0#u64 x) := by rfl
theorem u32_math (x : U32) : (⟨-x.bv⟩ : U32).val = if x.val = 0 then 0 else 2^32 - x.val := by
 exact neg_math x.bv
theorem u64_math (x : U64) : (⟨-x.bv⟩ : U64).val = if x.val = 0 then 0 else 2^64 - x.val := by
 exact neg_math x.bv
#print axioms u32_existing_sub
#print axioms u64_existing_sub
#print axioms u32_math
#print axioms u64_math
end NegBridge
