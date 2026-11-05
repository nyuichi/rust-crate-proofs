import SignedVariableNormal
open Aeneas Aeneas.Std Result Std.WP
open signed_variable_roots.byteorder_lite
namespace SignedWrites
private theorem returns {T : Type} {m : Result T} {post : T → Prop}
    (h : m ⦃ post ⦄) : ∃ value, m = .ok value ∧ post value := by
  cases m using Result.cases with
  | ret value => exact ⟨value, rfl, (spec_ok value).mp h⟩
  | vis effect k => simp only [spec_vis] at h
  | div => simp only [spec_div] at h

private def tail (n : I64) (shift : Usize) : Result U64 := do
  let i1 ← IScalar.shiftLeft_UScalar n shift
  let i2 ← lift (IScalar.hcast .U64 i1)
  UScalar.shiftRight_UScalar i2 shift
private def rest (n : I64) (i : Usize) : Result U64 := do
  let shift ← UScalar.mul i (8#usize)
  tail n shift

theorem unextend_large (n : I64) (count : Usize) (large : 8 < count.val) :
    unextend_sign n count = .fail .integerOverflow := by
  have hsub : UScalar.sub (8#usize) count = .fail .integerOverflow := by
    exact if_pos large
  change (UScalar.sub (8#usize) count >>= rest n) = _
  exact (congrArg (fun x : Result Usize => x >>= rest n) hsub).trans (by simp only [bind_tc_fail])

theorem unextend_zero (n : I64) : unextend_sign n (0#usize) = .fail .integerOverflow := by
  have hs : UScalar.sub (8#usize) (0#usize) = .ok (8#usize) := by
    have h : UScalar.sub (8#usize) (0#usize) ⦃ r => r = 8#usize ⦄ := by
      change ((8#usize) - (0#usize) : Result Usize) ⦃ r => r = 8#usize ⦄
      step
      apply UScalar.eq_of_val_eq
      scalar_tac
    obtain ⟨r, hr, heq⟩ := returns h
    exact heq ▸ hr
  have hm : UScalar.mul (8#usize) (8#usize) = .ok (64#usize) := by
    have h : UScalar.mul (8#usize) (8#usize) ⦃ r => r = 64#usize ⦄ := by
      change ((8#usize) * (8#usize) : Result Usize) ⦃ r => r = 64#usize ⦄
      step
      apply UScalar.eq_of_val_eq
      scalar_tac
    obtain ⟨r, hr, heq⟩ := returns h
    exact heq ▸ hr
  have hshift : IScalar.shiftLeft_UScalar n (64#usize) = .fail .integerOverflow := by
    unfold IScalar.shiftLeft_UScalar IScalar.shiftLeft
    have h : ¬ ((64#usize).val < IScalarTy.I64.numBits) := by decide
    exact if_neg h
  have ht : tail n (64#usize) = .fail .integerOverflow := by
    unfold tail
    exact (congrArg (fun x : Result I64 => x >>= fun i1 => do
      let i2 ← lift (IScalar.hcast .U64 i1)
      UScalar.shiftRight_UScalar i2 (64#usize)) hshift).trans (by simp only [bind_tc_fail])
  have hr : rest n (8#usize) = .fail .integerOverflow := by
    change (UScalar.mul (8#usize) (8#usize) >>= tail n) = _
    exact (congrArg (fun x : Result Usize => x >>= tail n) hm).trans (by simpa only [bind_tc_ok] using ht)
  change (UScalar.sub (8#usize) (0#usize) >>= rest n) = _
  exact (congrArg (fun x : Result Usize => x >>= rest n) hs).trans (by simpa only [bind_tc_ok] using hr)
end SignedWrites
#print axioms SignedWrites.unextend_zero
#print axioms SignedWrites.unextend_large

namespace SignedWrites
theorem unextend_invalid (n : I64) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 8)) :
    unextend_sign n count = .fail .integerOverflow := by
  by_cases h : count.val = 0
  · have hc : count = 0#usize := UScalar.eq_of_val_eq (by simpa using h)
    exact hc ▸ unextend_zero n
  · exact unextend_large n count (by omega)
theorem be_write_int_bad_count (buf : Slice U8) (n : I64) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 8)) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .integerOverflow := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int
  rw [unextend_invalid n count bad]
  simp only [Std.bind_fail]

theorem be_write_int_short (buf : Slice U8) (n : I64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (short : buf.length < count.val) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .assertionFailure := by
  obtain ⟨word, hw, hv⟩ := returns (unextend_sign_spec n count minimum maximum)
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_int
  rw [hw]
  simp only [Std.bind_ok]
  apply be_write_uint_invalid
  intro h
  omega
theorem le_write_int_bad_count (buf : Slice U8) (n : I64) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 8)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .integerOverflow := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int
  rw [unextend_invalid n count bad]
  simp only [Std.bind_fail]

theorem le_write_int_short (buf : Slice U8) (n : I64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (short : buf.length < count.val) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .assertionFailure := by
  obtain ⟨word, hw, hv⟩ := returns (unextend_sign_spec n count minimum maximum)
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_int
  rw [hw]
  simp only [Std.bind_ok]
  apply le_write_uint_invalid
  intro h
  omega
end SignedWrites
#print axioms SignedWrites.be_write_int_bad_count
#print axioms SignedWrites.be_write_int_short
#print axioms SignedWrites.le_write_int_bad_count
#print axioms SignedWrites.le_write_int_short
