import SignedVariableProofs
open Aeneas Aeneas.Std Result Std.WP
open signed_variable_roots.byteorder_lite
namespace SignedWrites
private theorem required_fits (n : U64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) :
    requiredBytes64 n ≤ count.val ↔ n.val < 2^(8*count.val) := by
  let y := UScalar.bv n ||| UScalar.bv (1#u64)
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
  have hylog : Nat.log 2 y.toNat < 64 := by
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
  change 8 - y.leadingZeros / 8 ≤ count.val ↔ _
  rw [BitVec.leadingZeros, if_neg hyzero]
  simp only [UScalarTy.U64_numBits_eq] at *
  omega

def truncatedWord (n : I64) (count : Usize) : U64 :=
  ⟨((IScalar.bv n) <<< ((8-count.val)*8) >>> ((8-count.val)*8)).cast (by simp)⟩

theorem truncatedWord_val (n : I64) (count : Usize) (maximum : count.val ≤ 8) :
    (truncatedWord n count).val = (IScalar.bv n).toNat % 2^(8*count.val) := by
  change ((IScalar.bv n) <<< ((8-count.val)*8) >>> ((8-count.val)*8)).toNat = _
  rw [truncated_value _ _ (by omega)]
  congr 2
  omega

theorem be_write_int_spec (buf : Slice U8) (n : I64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (enough : count.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_be_bytes (truncatedWord n count)).val.drop (8-count.val)) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int
  step with unextend_sign_spec
  have hi : i = truncatedWord n count :=
    UScalar.eq_of_val_eq (i_post.trans (truncatedWord_val n count maximum).symm)
  have fit : requiredBytes64 i ≤ count.val := by
    apply (required_fits i count minimum maximum).2
    rw [i_post]
    exact Nat.mod_lt _ (Nat.two_pow_pos _)
  rw [hi] at fit ⊢
  exact be_write_uint_spec buf (truncatedWord n count) count fit maximum enough
theorem le_write_int_spec (buf : Slice U8) (n : I64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (enough : count.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_le_bytes (truncatedWord n count)).val.take count.val) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int
  step with unextend_sign_spec
  have hi : i = truncatedWord n count :=
    UScalar.eq_of_val_eq (i_post.trans (truncatedWord_val n count maximum).symm)
  have fit : requiredBytes64 i ≤ count.val := by
    apply (required_fits i count minimum maximum).2
    rw [i_post]
    exact Nat.mod_lt _ (Nat.two_pow_pos _)
  rw [hi] at fit ⊢
  exact le_write_uint_spec buf (truncatedWord n count) count fit maximum enough
end SignedWrites
#print axioms SignedWrites.be_write_int_spec

#print axioms SignedWrites.le_write_int_spec
