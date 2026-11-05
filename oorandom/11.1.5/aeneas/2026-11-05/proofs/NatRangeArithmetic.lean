import Mathlib.Data.Nat.Basic
namespace NatRangeArithmetic
theorem product_bound (M w s : Nat) (mp : 0 < M) (sp : 0 < s) (wb : w < M) (sb : s < M) :
 w*s < M*s ∧ w*s < M*M ∧ (w*s)/M < s := by
 have p1 : w*s < M*s := Nat.mul_lt_mul_of_pos_right wb sp
 have p2 : M*s < M*M := Nat.mul_lt_mul_of_pos_left sb mp
 refine ⟨p1,lt_trans p1 p2,?_⟩
 apply (Nat.div_lt_iff_lt_mul mp).2
 simpa only [Nat.mul_comm] using p1

theorem offset_bound (M start finish high : Nat) (valid : start < finish)
 (fb : finish < M) (hb : high < finish-start) :
 high+start < finish ∧ high < M ∧ high+start < M := by
 have total : high+start < finish := by
  have h := Nat.add_lt_add_right hb start
  simpa only [Nat.sub_add_cancel (Nat.le_of_lt valid)] using h
 have high_lt_finish : high < finish := lt_of_lt_of_le hb (Nat.sub_le finish start)
 exact ⟨total,lt_trans high_lt_finish fb,lt_trans total fb⟩
#print axioms product_bound
#print axioms offset_bound
end NatRangeArithmetic
