import Signed128WriteHelper
open Aeneas Aeneas.Std Result Std.WP
open lite_signed128_roots.byteorder_lite
open Signed128Support
namespace Signed128Writes
private theorem required_fits (n : U128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) :
    requiredBytes128 n ≤ count.val ↔ n.val < 2^(8*count.val) := by
  let y := UScalar.bv n ||| UScalar.bv (1#u128)
  have hy : y.toNat = n.val ||| 1 := by
    simp only [y, BitVec.toNat_or]
    rfl
  have hypos : y.toNat ≠ 0 := by
    have h : 1 ≤ n.val ||| 1 := Nat.right_le_or
    omega
  have hyzero : y ≠ 0 := by
    intro h
    apply hypos
    rw [h]
    rfl
  have hylog : Nat.log 2 y.toNat < 128 := by
    apply Nat.log_lt_of_lt_pow hypos
    exact y.isLt
  have hor : y.toNat < 2^(8*count.val) ↔ n.val < 2^(8*count.val) := by
    rw [hy]
    constructor
    · intro h
      exact lt_of_le_of_lt Nat.left_le_or h
    · intro h
      exact Nat.or_lt_two_pow h (Nat.one_lt_two_pow (by omega))
  have hl : Nat.log 2 y.toNat < 8*count.val ↔ y.toNat < 2^(8*count.val) :=
    Nat.log_lt_iff_lt_pow (by decide) hypos
  change 16 - y.leadingZeros / 8 ≤ count.val ↔ _
  rw [BitVec.leadingZeros, if_neg hyzero]
  simp only [UScalarTy.U128_numBits_eq] at *
  omega

def truncatedWord (n : I128) (count : Usize) : U128 :=
  ⟨((IScalar.bv n) <<< ((16-count.val)*8) >>> ((16-count.val)*8)).cast (by simp)⟩

theorem truncatedWord_val (n : I128) (count : Usize) (maximum : count.val ≤ 16) :
    (truncatedWord n count).val = (IScalar.bv n).toNat % 2^(8*count.val) := by
  change ((IScalar.bv n) <<< ((16-count.val)*8) >>> ((16-count.val)*8)).toNat = _
  rw [truncated_value _ _ (by omega)]
  congr 2
  omega

theorem be_write_int128_spec (buf : Slice U8) (n : I128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (enough : count.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_be_bytes (truncatedWord n count)).val.drop (16-count.val)) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int128
  step with unextend_sign128_spec
  have hi : i = truncatedWord n count :=
    UScalar.eq_of_val_eq (i_post.trans (truncatedWord_val n count maximum).symm)
  have fit : requiredBytes128 i ≤ count.val := by
    apply (required_fits i count minimum maximum).2
    rw [i_post]
    exact Nat.mod_lt _ (Nat.two_pow_pos _)
  rw [hi] at fit ⊢
  exact be_write_uint128_spec buf (truncatedWord n count) count fit maximum enough
theorem le_write_int128_spec (buf : Slice U8) (n : I128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (enough : count.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_le_bytes (truncatedWord n count)).val.take count.val) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int128
  step with unextend_sign128_spec
  have hi : i = truncatedWord n count :=
    UScalar.eq_of_val_eq (i_post.trans (truncatedWord_val n count maximum).symm)
  have fit : requiredBytes128 i ≤ count.val := by
    apply (required_fits i count minimum maximum).2
    rw [i_post]
    exact Nat.mod_lt _ (Nat.two_pow_pos _)
  rw [hi] at fit ⊢
  exact le_write_uint128_spec buf (truncatedWord n count) count fit maximum enough
end Signed128Writes
#print axioms Signed128Writes.be_write_int128_spec

#print axioms Signed128Writes.le_write_int128_spec
