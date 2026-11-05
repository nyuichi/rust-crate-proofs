import Signed128WriteNormal
open Aeneas Aeneas.Std Result Std.WP
open lite_signed128_roots.byteorder_lite
open Signed128Support
namespace Signed128Writes
private theorem returns {T : Type} {m : Result T} {post : T → Prop}
    (h : m ⦃ post ⦄) : ∃ value, m = .ok value ∧ post value := by
  cases m using Result.cases with
  | ret value => exact ⟨value, rfl, (spec_ok value).mp h⟩
  | vis effect k => simp only [spec_vis] at h
  | div => simp only [spec_div] at h

private def tail (n : I128) (shift : Usize) : Result U128 := do
  let i1 ← IScalar.shiftLeft_UScalar n shift
  let i2 ← lift (IScalar.hcast .U128 i1)
  UScalar.shiftRight_UScalar i2 shift
private def rest (n : I128) (i : Usize) : Result U128 := do
  let shift ← UScalar.mul i (8#usize)
  tail n shift

theorem unextend_large (n : I128) (count : Usize) (large : 16 < count.val) :
    unextend_sign128 n count = .fail .integerOverflow := by
  have hsub : UScalar.sub (16#usize) count = .fail .integerOverflow := by
    exact if_pos large
  change (UScalar.sub (16#usize) count >>= rest n) = _
  exact (congrArg (fun x : Result Usize => x >>= rest n) hsub).trans (by simp only [bind_tc_fail])

theorem unextend_zero (n : I128) : unextend_sign128 n (0#usize) = .fail .integerOverflow := by
  have hs : UScalar.sub (16#usize) (0#usize) = .ok (16#usize) := by
    have h : UScalar.sub (16#usize) (0#usize) ⦃ r => r = 16#usize ⦄ := by
      change ((16#usize) - (0#usize) : Result Usize) ⦃ r => r = 16#usize ⦄
      step
      apply UScalar.eq_of_val_eq
      scalar_tac
    obtain ⟨r, hr, heq⟩ := returns h
    exact heq ▸ hr
  have hm : UScalar.mul (16#usize) (8#usize) = .ok (128#usize) := by
    have h : UScalar.mul (16#usize) (8#usize) ⦃ r => r = 128#usize ⦄ := by
      change ((16#usize) * (8#usize) : Result Usize) ⦃ r => r = 128#usize ⦄
      step
      apply UScalar.eq_of_val_eq
      scalar_tac
    obtain ⟨r, hr, heq⟩ := returns h
    exact heq ▸ hr
  have hshift : IScalar.shiftLeft_UScalar n (128#usize) = .fail .integerOverflow := by
    unfold IScalar.shiftLeft_UScalar IScalar.shiftLeft
    have h : ¬ ((128#usize).val < IScalarTy.I128.numBits) := by decide
    exact if_neg h
  have ht : tail n (128#usize) = .fail .integerOverflow := by
    unfold tail
    exact (congrArg (fun x : Result I128 => x >>= fun i1 => do
      let i2 ← lift (IScalar.hcast .U128 i1)
      UScalar.shiftRight_UScalar i2 (128#usize)) hshift).trans (by simp only [bind_tc_fail])
  have hr : rest n (16#usize) = .fail .integerOverflow := by
    change (UScalar.mul (16#usize) (8#usize) >>= tail n) = _
    exact (congrArg (fun x : Result Usize => x >>= tail n) hm).trans (by simpa only [bind_tc_ok] using ht)
  change (UScalar.sub (16#usize) (0#usize) >>= rest n) = _
  exact (congrArg (fun x : Result Usize => x >>= rest n) hs).trans (by simpa only [bind_tc_ok] using hr)
end Signed128Writes
#print axioms Signed128Writes.unextend_zero
#print axioms Signed128Writes.unextend_large

namespace Signed128Writes
theorem unextend_invalid (n : I128) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 16)) :
    unextend_sign128 n count = .fail .integerOverflow := by
  by_cases h : count.val = 0
  · have hc : count = 0#usize := UScalar.eq_of_val_eq (by simpa using h)
    exact hc ▸ unextend_zero n
  · exact unextend_large n count (by omega)
theorem be_write_int128_bad_count (buf : Slice U8) (n : I128) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 16)) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .integerOverflow := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int128
  rw [unextend_invalid n count bad]
  simp only [Std.bind_fail]

theorem be_write_int128_short (buf : Slice U8) (n : I128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (short : buf.length < count.val) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .assertionFailure := by
  obtain ⟨word, hw, hv⟩ := returns (unextend_sign128_spec n count minimum maximum)
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int128
  rw [hw]
  simp only [Std.bind_ok]
  apply be_write_uint128_invalid
  intro h
  omega
theorem le_write_int128_bad_count (buf : Slice U8) (n : I128) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 16)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .integerOverflow := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int128
  rw [unextend_invalid n count bad]
  simp only [Std.bind_fail]

theorem le_write_int128_short (buf : Slice U8) (n : I128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (short : buf.length < count.val) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .assertionFailure := by
  obtain ⟨word, hw, hv⟩ := returns (unextend_sign128_spec n count minimum maximum)
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int128
  rw [hw]
  simp only [Std.bind_ok]
  apply le_write_uint128_invalid
  intro h
  omega
end Signed128Writes
#print axioms Signed128Writes.be_write_int128_bad_count
#print axioms Signed128Writes.be_write_int128_short
#print axioms Signed128Writes.le_write_int128_bad_count
#print axioms Signed128Writes.le_write_int128_short
