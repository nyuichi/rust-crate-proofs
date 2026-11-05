import Signed128Support
open Aeneas Aeneas.Std Result Std.WP
open lite_signed128_roots.byteorder_lite
namespace Signed128Writes
theorem truncated_value (x : BitVec 128) (shift : Nat) (hs : shift ≤ 128) :
    (x <<< shift >>> shift).toNat = x.toNat % 2^(128-shift) := by
  simp only [BitVec.toNat_ushiftRight, BitVec.toNat_shiftLeft]
  rw [Nat.shiftLeft_eq]
  have hp : 2^128 = 2^(128-shift) * 2^shift := by
    rw [← Nat.pow_add]
    congr 1
    omega
  rw [hp, Nat.mul_mod_mul_right, Nat.shiftRight_eq_div_pow]
  exact Nat.mul_div_cancel _ (Nat.two_pow_pos shift)

theorem unextend_sign128_spec (n : I128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) :
    unextend_sign128 n count ⦃ r => r.val = (IScalar.bv n).toNat % 2^(8*count.val) ⦄ := by
  unfold unextend_sign128
  step
  step
  step
  step
  step
  have hi2 : (UScalar.bv i2) = (IScalar.bv i1) := by
    rw [i2_post, IScalar.hcast_bv_eq]
    simp
  have hs : shift.val ≤ 128 := by omega
  calc
    r.val = ((UScalar.bv i2) >>> shift.val).toNat := congrArg BitVec.toNat r_post1
    _ = ((IScalar.bv n) <<< shift.val >>> shift.val).toNat :=
      congrArg (fun v : BitVec 128 => (v >>> shift.val).toNat) (hi2.trans i1_post1)
    _ = (IScalar.bv n).toNat % 2^(128-shift.val) := truncated_value _ _ hs
    _ = _ := by congr 2; omega

end Signed128Writes
#print axioms Signed128Writes.unextend_sign128_spec
