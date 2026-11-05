import NatRangeArithmetic
import Aeneas.Std.Scalar.CoreConvertNum
import Aeneas.Std.Scalar.WrappingOps.Mul
import Aeneas.Std.Scalar.Ops.Mul
open Aeneas Aeneas.Std Result
namespace RangeArithmetic
set_option maxHeartbeats 400000

theorem product32 (w s : U32) (positive : 0 < s.val) :
 w.val*s.val < 2^32*s.val ∧ w.val*s.val < 2^64 ∧
 (w.val*s.val)/2^32 < s.val := by
 obtain ⟨hwidth,hwide,hhigh⟩ := NatRangeArithmetic.product_bound (2^32) w.val s.val (Nat.two_pow_pos 32) positive w.hBounds s.hBounds
 refine ⟨hwidth,?_,hhigh⟩
 simpa only [← Nat.pow_add] using hwide

theorem product64 (w s : U64) (positive : 0 < s.val) :
 w.val*s.val < 2^64*s.val ∧ w.val*s.val < 2^128 ∧
 (w.val*s.val)/2^64 < s.val := by
 obtain ⟨hwidth,hwide,hhigh⟩ := NatRangeArithmetic.product_bound (2^64) w.val s.val (Nat.two_pow_pos 64) positive w.hBounds s.hBounds
 refine ⟨hwidth,?_,hhigh⟩
 simpa only [← Nat.pow_add] using hwide

theorem wrapping_product32 (w s : U32) (positive : 0 < s.val) :
 (core.num.U64.wrapping_mul (core.convert.num.FromU64U32.from w)
   (core.convert.num.FromU64U32.from s)).val = w.val*s.val := by
 simp only [core.num.U64.wrapping_mul_val_eq,core.convert.num.FromU64U32.from_val_eq,UScalar.size_def,UScalarTy.U64_numBits_eq]
 exact Nat.mod_eq_of_lt (product32 w s positive).2.1

theorem offset32 (start finish : U32) (valid : start.val < finish.val) (q : Nat)
 (high : q < finish.val-start.val) : q + start.val < finish.val ∧ q < 2^32 ∧ q+start.val < 2^32 :=
 NatRangeArithmetic.offset_bound (2^32) start.val finish.val q valid finish.hBounds high

theorem offset64 (start finish : U64) (valid : start.val < finish.val) (q : Nat)
 (high : q < finish.val-start.val) : q + start.val < finish.val ∧ q < 2^64 ∧ q+start.val < 2^64 :=
 NatRangeArithmetic.offset_bound (2^64) start.val finish.val q valid finish.hBounds high
theorem wide_product32 (w s : U32) (positive : 0 < s.val) :
 (core.convert.num.FromU64U32.from w) * (core.convert.num.FromU64U32.from s)
 ⦃ m => m.val = w.val*s.val ⦄ := by
 have hm : w.val*s.val ≤ UScalar.max .U64 := by
  simpa only [UScalar.max,UScalarTy.U64_numBits_eq, Nat.pred_eq_sub_one] using Nat.le_pred_of_lt (product32 w s positive).2.1
 refine WP.spec_mono (UScalar.mul_spec (x:=core.convert.num.FromU64U32.from w)
  (y:=core.convert.num.FromU64U32.from s) (by simpa only [core.convert.num.FromU64U32.from_val_eq] using hm)) ?_
 intro m h
 simpa only [core.convert.num.FromU64U32.from_val_eq] using h

theorem wide_product64 (w s : U64) (positive : 0 < s.val) :
 (core.convert.num.FromU128U64.from w) * (core.convert.num.FromU128U64.from s)
 ⦃ m => m.val = w.val*s.val ⦄ := by
 have hm : w.val*s.val ≤ UScalar.max .U128 := by
  simpa only [UScalar.max,UScalarTy.U128_numBits_eq, Nat.pred_eq_sub_one] using Nat.le_pred_of_lt (product64 w s positive).2.1
 refine WP.spec_mono (UScalar.mul_spec (x:=core.convert.num.FromU128U64.from w)
  (y:=core.convert.num.FromU128U64.from s) (by simpa only [core.convert.num.FromU128U64.from_val_eq] using hm)) ?_
 intro m h
 simpa only [core.convert.num.FromU128U64.from_val_eq] using h

#print axioms product32
#print axioms product64
#print axioms wrapping_product32
#print axioms offset32
#print axioms offset64
end RangeArithmetic
