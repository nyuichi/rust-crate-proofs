import extensions.core_more.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.CoreMore

/-- The `Bytes` equality implementation compares the two successful slice
    views with the generated slice equality routine. -/
theorem bytes_partial_eq_uses_slice_eq
    (self other : bytes.Bytes) (s1 s2 : Slice Std.U8) (same : Bool)
    (hself : bytes.Bytes.as_slice self = ok s1)
    (hother : bytes.Bytes.as_slice other = ok s2)
    (heq : core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 s1 s2 =
      ok same) :
    bytes.Bytes.Insts.CoreCmpPartialEqBytes.eq self other = ok same := by
  simp [bytes.Bytes.Insts.CoreCmpPartialEqBytes.eq, hself, hother, heq]

/-- `BytesMut` equality compares the two successful slice views in the same
    way. -/
theorem bytes_mut_partial_eq_uses_slice_eq
    (self other : bytes_mut.BytesMut) (s1 s2 : Slice Std.U8) (same : Bool)
    (hself : bytes_mut.BytesMut.as_slice self = ok s1)
    (hother : bytes_mut.BytesMut.as_slice other = ok s2)
    (heq : core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 s1 s2 =
      ok same) :
    bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytesMut.eq self other =
      ok same := by
  simp [bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytesMut.eq,
    hself, hother, heq]

theorem bytes_partial_eq_characterizes_slice_equality
    (self other : bytes.Bytes) (s1 s2 : Slice Std.U8) (same : Bool)
    (hself : bytes.Bytes.as_slice self = ok s1)
    (hother : bytes.Bytes.as_slice other = ok s2)
    (heq : core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 s1 s2 =
      ok same) :
    bytes.Bytes.Insts.CoreCmpPartialEqBytes.eq self other = ok same ∧
      (same = true ↔ s1 = s2) := by
  have hne : ∀ x y : Std.U8,
      core.cmp.PartialEqU8.ne x y ⦃ b => b ↔ x ≠ y ⦄ := by
    intro x y
    simp [core.cmp.impls.PartialEqU8.ne]
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 s1 s2 hne
  rw [heq] at hspec
  have htruth : (same = true ↔ s1 = s2) := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨bytes_partial_eq_uses_slice_eq self other s1 s2 same
      hself hother heq, htruth⟩

theorem bytes_mut_partial_eq_characterizes_slice_equality
    (self other : bytes_mut.BytesMut) (s1 s2 : Slice Std.U8) (same : Bool)
    (hself : bytes_mut.BytesMut.as_slice self = ok s1)
    (hother : bytes_mut.BytesMut.as_slice other = ok s2)
    (heq : core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 s1 s2 =
      ok same) :
    bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytesMut.eq self other =
        ok same ∧ (same = true ↔ s1 = s2) := by
  have hne : ∀ x y : Std.U8,
      core.cmp.PartialEqU8.ne x y ⦃ b => b ↔ x ≠ y ⦄ := by
    intro x y
    simp [core.cmp.impls.PartialEqU8.ne]
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 s1 s2 hne
  rw [heq] at hspec
  have htruth : (same = true ↔ s1 = s2) := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨bytes_mut_partial_eq_uses_slice_eq self other s1 s2 same
      hself hother heq, htruth⟩

private theorem u8_partial_ne_spec :
    ∀ x y : Std.U8,
      core.cmp.PartialEqU8.ne x y ⦃ b => b ↔ x ≠ y ⦄ := by
  intro x y
  simp [core.cmp.impls.PartialEqU8.ne]

private theorem result_allM_always_succeeds
    {α : Type} (p : α → Result Bool)
    (hp : ∀ x, ∃ b, p x = ok b) :
    ∀ xs : List α, ∃ b, List.allM p xs = ok b := by
  intro xs
  induction xs with
  | nil => exact ⟨true, rfl⟩
  | cons x xs ih =>
    rcases hp x with ⟨b, hb⟩
    rcases ih with ⟨result, hresult⟩
    cases b with
    | false =>
      refine ⟨false, ?_⟩
      simp [List.allM, hb]
      change ok false = ok false
      rfl
    | true => exact ⟨result, by simp [List.allM, hb, hresult]⟩

private theorem u8_slice_eq_returns_boolean
    (s1 s2 : Slice Std.U8) :
    ∃ same, core.slice.cmp.PartialEqSlice.eq
      core.cmp.PartialEqU8 s1 s2 = ok same := by
  let p : Std.U8 × Std.U8 → Result Bool := fun xy => do
    let neq ← core.cmp.PartialEqU8.ne xy.1 xy.2
    ok (!neq)
  have hp : ∀ xy, ∃ result, p xy = ok result := by
    intro xy
    refine ⟨decide (xy.1 = xy.2), ?_⟩
    have hscalar : xy.1.val = xy.2.val ↔ xy.1 = xy.2 :=
      ⟨UScalar.eq_of_val_eq, UScalar.val_eq_of_eq⟩
    simp [p, core.cmp.impls.PartialEqU8.ne, hscalar]
  by_cases hlen : s1.length = s2.length
  · have hsuccess := result_allM_always_succeeds p hp
      (List.zip s1.val s2.val)
    rcases hsuccess with ⟨same, hresult⟩
    refine ⟨same, ?_⟩
    unfold core.slice.cmp.PartialEqSlice.eq
    simp only [if_pos hlen]
    change List.allM p (List.zip s1.val s2.val) = ok same
    exact hresult
  · refine ⟨false, ?_⟩
    simp [core.slice.cmp.PartialEqSlice.eq, hlen]

theorem bytes_partial_eq_returns_boolean_characterizing_slices
    (self other : bytes.Bytes) (s1 s2 : Slice Std.U8)
    (hself : bytes.Bytes.as_slice self = ok s1)
    (hother : bytes.Bytes.as_slice other = ok s2) :
    ∃ same,
      bytes.Bytes.Insts.CoreCmpPartialEqBytes.eq self other = ok same ∧
      (same = true ↔ s1 = s2) := by
  rcases u8_slice_eq_returns_boolean s1 s2 with ⟨same, heq⟩
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 s1 s2 u8_partial_ne_spec
  rw [heq] at hspec
  have htruth : same = true ↔ s1 = s2 := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨same,
    bytes_partial_eq_uses_slice_eq self other s1 s2 same hself hother heq,
    htruth⟩

theorem bytes_mut_partial_eq_returns_boolean_characterizing_slices
    (self other : bytes_mut.BytesMut) (s1 s2 : Slice Std.U8)
    (hself : bytes_mut.BytesMut.as_slice self = ok s1)
    (hother : bytes_mut.BytesMut.as_slice other = ok s2) :
    ∃ same,
      bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytesMut.eq self other =
        ok same ∧ (same = true ↔ s1 = s2) := by
  rcases u8_slice_eq_returns_boolean s1 s2 with ⟨same, heq⟩
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 s1 s2 u8_partial_ne_spec
  rw [heq] at hspec
  have htruth : same = true ↔ s1 = s2 := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨same,
    bytes_mut_partial_eq_uses_slice_eq self other s1 s2 same hself hother heq,
    htruth⟩

/-- `PartialOrd` for `Bytes` returns the total ordering result in `some`. -/
theorem bytes_partial_cmp_wraps_slice_order
    (self other : bytes.Bytes) (s1 s2 : Slice Std.U8) (ordering : Ordering)
    (hself : bytes.Bytes.as_slice self = ok s1)
    (hother : bytes.Bytes.as_slice other = ok s2)
    (hcmp : Slice.Insts.CoreCmpOrd.cmp core.cmp.OrdU8 s1 s2 =
      ok ordering) :
    bytes.Bytes.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      ok (some ordering) := by
  simp [bytes.Bytes.Insts.CoreCmpPartialOrdBytes.partial_cmp,
    bytes.Bytes.Insts.CoreCmpOrd.cmp, hself, hother, hcmp]

/-- `PartialOrd` for `BytesMut` returns the total ordering result in `some`. -/
theorem bytes_mut_partial_cmp_wraps_slice_order
    (self other : bytes_mut.BytesMut) (s1 s2 : Slice Std.U8)
    (ordering : Ordering)
    (hself : bytes_mut.BytesMut.as_slice self = ok s1)
    (hother : bytes_mut.BytesMut.as_slice other = ok s2)
    (hcmp : Slice.Insts.CoreCmpOrd.cmp core.cmp.OrdU8 s1 s2 =
      ok ordering) :
    bytes_mut.BytesMut.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      ok (some ordering) := by
  simp [bytes_mut.BytesMut.Insts.CoreCmpPartialOrdBytesMut.partial_cmp,
    bytes_mut.BytesMut.Insts.CoreCmpOrd.cmp, hself, hother, hcmp]

#print axioms bytes_partial_eq_uses_slice_eq
#print axioms bytes_mut_partial_eq_uses_slice_eq
#print axioms bytes_partial_eq_characterizes_slice_equality
#print axioms bytes_mut_partial_eq_characterizes_slice_equality
#print axioms bytes_partial_eq_returns_boolean_characterizing_slices
#print axioms bytes_mut_partial_eq_returns_boolean_characterizing_slices
#print axioms bytes_partial_cmp_wraps_slice_order
#print axioms bytes_mut_partial_cmp_wraps_slice_order

end BytesProofs.CoreMore

#print axioms BytesProofs.CoreMore.bytes_mut_partial_cmp_wraps_slice_order
#print axioms BytesProofs.CoreMore.bytes_mut_partial_eq_characterizes_slice_equality
#print axioms BytesProofs.CoreMore.bytes_mut_partial_eq_returns_boolean_characterizing_slices
#print axioms BytesProofs.CoreMore.bytes_mut_partial_eq_uses_slice_eq
#print axioms BytesProofs.CoreMore.bytes_partial_cmp_wraps_slice_order
#print axioms BytesProofs.CoreMore.bytes_partial_eq_characterizes_slice_equality
#print axioms BytesProofs.CoreMore.bytes_partial_eq_returns_boolean_characterizing_slices
#print axioms BytesProofs.CoreMore.bytes_partial_eq_uses_slice_eq
