import U24WriteSupport
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite_u24_roots.byteorder_lite
namespace U24Writes
private theorem u32_cast_val (n : U32) : (UScalar.cast .U64 n).val = n.val := by
  rw [UScalar.cast_val_eq]
  have hn := n.hmax
  apply Nat.mod_eq_of_lt
  scalar_tac

theorem fits24 (n : U32) : requiredBytes64 (UScalar.cast .U64 n) ≤ 3 ↔ n.val < 2^24 := by
  let x := UScalar.bv (UScalar.cast .U64 n)
  let y := x ||| UScalar.bv (1#u64)
  have hy : y.toNat = n.val ||| 1 := by
    simp only [y, x, BitVec.toNat_or]
    change (UScalar.cast .U64 n).val ||| 1 = _
    rw [u32_cast_val]
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
  have hor : y.toNat < 2^24 ↔ n.val < 2^24 := by
    rw [hy]
    constructor
    · intro h
      exact lt_of_le_of_lt Nat.left_le_or h
    · intro h
      exact Nat.or_lt_two_pow h (by decide)
  have hl : Nat.log 2 y.toNat < 24 ↔ y.toNat < 2^24 :=
    Nat.log_lt_iff_lt_pow (by decide) hypos
  change 8 - y.leadingZeros / 8 ≤ 3 ↔ _
  rw [BitVec.leadingZeros, if_neg hyzero]
  simp only [UScalarTy.U64_numBits_eq] at *
  omega
end U24Writes
#print axioms U24Writes.fits24

namespace U24Writes
theorem be_write_u24_spec (buf : Slice U8) (n : U32)
    (fit : n.val < 2^24) (enough : 3 ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_be_bytes (UScalar.cast .U64 n)).val.drop 5) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u24
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    (be_write_uint_spec buf (UScalar.cast .U64 n) (3#usize)
      ((fits24 n).2 fit) (by decide) enough)

theorem be_write_u24_invalid (buf : Slice U8) (n : U32)
    (bad : ¬ (n.val < 2^24 ∧ 3 ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u24
  have hbad : ¬ (requiredBytes64 (UScalar.cast .U64 n) ≤ (3#usize).val ∧
      (3#usize).val ≤ 8 ∧ (3#usize).val ≤ buf.length) := by
    intro h
    exact bad ⟨(fits24 n).1 h.1, h.2.2⟩
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    (be_write_uint_invalid buf (UScalar.cast .U64 n) (3#usize) hbad)
theorem le_write_u24_spec (buf : Slice U8) (n : U32)
    (fit : n.val < 2^24) (enough : 3 ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_le_bytes (UScalar.cast .U64 n)).val.take 3) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u24
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    (le_write_uint_spec buf (UScalar.cast .U64 n) (3#usize)
      ((fits24 n).2 fit) (by decide) enough)

theorem le_write_u24_invalid (buf : Slice U8) (n : U32)
    (bad : ¬ (n.val < 2^24 ∧ 3 ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u24
  have hbad : ¬ (requiredBytes64 (UScalar.cast .U64 n) ≤ (3#usize).val ∧
      (3#usize).val ≤ 8 ∧ (3#usize).val ≤ buf.length) := by
    intro h
    exact bad ⟨(fits24 n).1 h.1, h.2.2⟩
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    (le_write_uint_invalid buf (UScalar.cast .U64 n) (3#usize) hbad)
end U24Writes
#print axioms U24Writes.be_write_u24_spec
#print axioms U24Writes.be_write_u24_invalid
#print axioms U24Writes.le_write_u24_spec
#print axioms U24Writes.le_write_u24_invalid

namespace U24Writes
theorem be_write_u24_contract (buf : Slice U8) (n : U32) :
    ((n.val < 2^24 ∧ 3 ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_be_bytes (UScalar.cast .U64 n)).val.drop 5) ⦄) ∧
    (¬ (n.val < 2^24 ∧ 3 ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n = .fail .assertionFailure) := by
  exact ⟨fun h => be_write_u24_spec buf n h.1 h.2, fun h => be_write_u24_invalid buf n h⟩
theorem le_write_u24_contract (buf : Slice U8) (n : U32) :
    ((n.val < 2^24 ∧ 3 ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_le_bytes (UScalar.cast .U64 n)).val.take 3) ⦄) ∧
    (¬ (n.val < 2^24 ∧ 3 ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_u24 buf n = .fail .assertionFailure) := by
  exact ⟨fun h => le_write_u24_spec buf n h.1 h.2, fun h => le_write_u24_invalid buf n h⟩
end U24Writes
#print axioms U24Writes.be_write_u24_contract
#print axioms U24Writes.le_write_u24_contract
