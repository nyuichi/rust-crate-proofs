import SignedVariableWriteSupport
open Aeneas Aeneas.Std Result Std.WP
open signed_variable_roots.byteorder_lite
namespace SignedWrites
theorem truncated_value (x : BitVec 64) (shift : Nat) (hs : shift ≤ 64) :
    (x <<< shift >>> shift).toNat = x.toNat % 2^(64-shift) := by
  simp only [BitVec.toNat_ushiftRight, BitVec.toNat_shiftLeft]
  rw [Nat.shiftLeft_eq]
  have hp : 2^64 = 2^(64-shift) * 2^shift := by
    rw [← Nat.pow_add]
    congr 1
    omega
  rw [hp, Nat.mul_mod_mul_right, Nat.shiftRight_eq_div_pow]
  exact Nat.mul_div_cancel _ (Nat.two_pow_pos shift)

theorem unextend_sign_spec (n : I64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) :
    unextend_sign n count ⦃ r => r.val = (IScalar.bv n).toNat % 2^(8*count.val) ⦄ := by
  unfold unextend_sign
  step
  step
  step
  step
  step
  have hi2 : (UScalar.bv i2) = (IScalar.bv i1) := by
    rw [i2_post, IScalar.hcast_bv_eq]
    simp
  have hs : shift.val ≤ 64 := by omega
  calc
    r.val = ((UScalar.bv i2) >>> shift.val).toNat := congrArg BitVec.toNat r_post1
    _ = ((IScalar.bv n) <<< shift.val >>> shift.val).toNat :=
      congrArg (fun v : BitVec 64 => (v >>> shift.val).toNat) (hi2.trans i1_post1)
    _ = (IScalar.bv n).toNat % 2^(64-shift.val) := truncated_value _ _ hs
    _ = _ := by congr 2; omega

end SignedWrites
#print axioms SignedWrites.unextend_sign_spec
