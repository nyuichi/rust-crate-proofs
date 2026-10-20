import extensions.comparisons.Generated

open Aeneas Aeneas.Std Result bytes_comparison_harness

namespace BytesProofs.Extensions.Comparisons

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

private theorem u8_slice_eq_characterizes_contents
    (s1 s2 : Slice Std.U8) :
    ∃ same,
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 s1 s2 = ok same ∧
      (same = true ↔ s1 = s2) := by
  rcases u8_slice_eq_returns_boolean s1 s2 with ⟨same, heq⟩
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 s1 s2 u8_partial_ne_spec
  rw [heq] at hspec
  have htruth : same = true ↔ s1 = s2 := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨same, heq, htruth⟩

/-- A concrete, executable lexicographic byte-list comparator. Its scalar
    comparison uses the same natural-number comparison as Aeneas' `u8`
    `Ord` model. -/
def u8ByteListLexOrder : List Std.U8 → List Std.U8 → Ordering
  | [], [] => .eq
  | [], _ :: _ => .lt
  | _ :: _, [] => .gt
  | x :: xs, y :: ys =>
      match compare x.val y.val with
      | .lt => .lt
      | .eq => u8ByteListLexOrder xs ys
      | .gt => .gt

/-- The recursive comparator is exactly Lean's pure `List.compareLex`
    over byte values; this establishes its lexicographic behavior without
    using the opaque Rust slice comparison model. -/
theorem u8ByteListLexOrder_eq_list_compareLex
    (xs ys : List Std.U8) :
    u8ByteListLexOrder xs ys =
      List.compareLex (fun x y => compare x.val y.val) xs ys := by
  induction xs generalizing ys with
  | nil => cases ys <;> rfl
  | cons x xs ih =>
    cases ys with
    | nil => rfl
    | cons y ys =>
      simp only [u8ByteListLexOrder, List.compareLex_cons_cons]
      cases compare x.val y.val <;> simp [Ordering.then, ih]

/-- The standard Aeneas `u8` partial-order model itself compares the scalar
    values using Lean's natural-number ordering. -/
theorem u8_partial_cmp_is_value_compare (x y : Std.U8) :
    core.cmp.PartialOrdU8.partial_cmp x y = ok (some (compare x.val y.val)) := by
  rfl

/-- This is the explicit semantic law needed to interpret the generated
    standard-library slice `partial_cmp` axiom lexicographically. The axiom
    is not proved here; the law remains a premise on the following contracts. -/
def byteSlicePartialOrdLexLaw : Prop :=
  ∀ (left right : Slice Std.U8),
    Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right =
      ok (some (u8ByteListLexOrder left.val right.val))

theorem bytes_eq_slice_after_as_slice
    (self : bytes.bytes.Bytes) (other slice : Slice Std.U8)
    (hslice : bytes.bytes.Bytes.as_slice self = ok slice) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 slice other := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq, hslice]

theorem bytes_eq_slice_returns_boolean_characterizing_contents
    (self : bytes.bytes.Bytes) (other slice : Slice Std.U8)
    (hslice : bytes.bytes.Bytes.as_slice self = ok slice) :
    ∃ same,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq self other = ok same ∧
      (same = true ↔ slice = other) := by
  rcases u8_slice_eq_returns_boolean slice other with ⟨same, heq⟩
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 slice other u8_partial_ne_spec
  rw [heq] at hspec
  have htruth : same = true ↔ slice = other := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨same, by
    simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq, hslice, heq], htruth⟩

theorem bytes_partial_cmp_slice_after_as_slice
    (self : bytes.bytes.Bytes) (other slice : Slice Std.U8)
    (hslice : bytes.bytes.Bytes.as_slice self = ok slice) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdSliceU8.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 slice other := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialOrdSliceU8.partial_cmp, hslice]

theorem bytes_partial_cmp_slice_matches_lexicographic_spec
    (self : bytes.bytes.Bytes) (other slice : Slice Std.U8)
    (hslice : bytes.bytes.Bytes.as_slice self = ok slice)
    (hstd : byteSlicePartialOrdLexLaw) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdSliceU8.partial_cmp self other =
      ok (some (u8ByteListLexOrder slice.val other.val)) := by
  rw [bytes_partial_cmp_slice_after_as_slice self other slice hslice]
  exact hstd slice other

theorem bytesmut_eq_slice_after_deref
    (self : bytes.bytes_mut.BytesMut) (other slice : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok slice) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 slice other := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq, hderef]

theorem bytesmut_eq_slice_returns_boolean_characterizing_contents
    (self : bytes.bytes_mut.BytesMut) (other slice : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok slice) :
    ∃ same,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq self other = ok same ∧
      (same = true ↔ slice = other) := by
  rcases u8_slice_eq_returns_boolean slice other with ⟨same, heq⟩
  have hspec := core.slice.cmp.PartialEqSlice.eq_homo_spec
    core.cmp.PartialEqU8 slice other u8_partial_ne_spec
  rw [heq] at hspec
  have htruth : same = true ↔ slice = other := by
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  exact ⟨same, by
    simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq, hderef, heq], htruth⟩

theorem bytesmut_partial_cmp_slice_after_deref
    (self : bytes.bytes_mut.BytesMut) (other slice : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok slice) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdSliceU8.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 slice other := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdSliceU8.partial_cmp, hderef]

theorem bytesmut_partial_cmp_slice_matches_lexicographic_spec
    (self : bytes.bytes_mut.BytesMut) (other slice : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok slice)
    (hstd : byteSlicePartialOrdLexLaw) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdSliceU8.partial_cmp self other =
      ok (some (u8ByteListLexOrder slice.val other.val)) := by
  rw [bytesmut_partial_cmp_slice_after_deref self other slice hderef]
  exact hstd slice other

theorem bytes_ref_eq_slice_uses_underlying_inst
    {T : Type _} (inst : core.cmp.PartialEq bytes.bytes.Bytes T)
    (self : bytes.bytes.Bytes) (other : T) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqSharedAT.eq inst self other = inst.eq self other := by
  rfl

theorem bytes_ref_partial_cmp_uses_underlying_inst
    {T : Type _} (inst : core.cmp.PartialOrd bytes.bytes.Bytes T)
    (self : bytes.bytes.Bytes) (other : T) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdSharedAT.partial_cmp inst self other =
      inst.partial_cmp self other := by
  rfl

theorem bytesmut_ref_eq_slice_uses_underlying_inst
    {T : Type _} (inst : core.cmp.PartialEq bytes.bytes_mut.BytesMut T)
    (self : bytes.bytes_mut.BytesMut) (other : T) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSharedAT.eq inst self other =
      inst.eq self other := by
  rfl

theorem bytesmut_ref_partial_cmp_uses_underlying_inst
    {T : Type _} (inst : core.cmp.PartialOrd bytes.bytes_mut.BytesMut T)
    (self : bytes.bytes_mut.BytesMut) (other : T) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdSharedAT.partial_cmp inst self other =
      inst.partial_cmp self other := by
  rfl

theorem bytes_slice_ref_eq_uses_bytes_generic_impl
    (self : Slice Std.U8) (other : bytes.bytes.Bytes) :
    Shared0SliceU8.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqSharedAT.eq
        bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8 other self := by
  rfl

theorem bytesmut_slice_ref_eq_uses_bytesmut_generic_impl
    (self : Slice Std.U8) (other : bytes.bytes_mut.BytesMut) :
    Shared0SliceU8.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSharedAT.eq
        bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8 other self := by
  rfl

theorem bytes_eq_str_after_slice_conversions
    (self : bytes.bytes.Bytes) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq, hleft, hright]

theorem bytes_eq_str_characterizes_contents
    (self : bytes.bytes.Bytes) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    ∃ same,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq, hleft, hright, heq], hcontents⟩

theorem bytes_partial_cmp_str_after_slice_conversions
    (self : bytes.bytes.Bytes) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdStr.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialOrdStr.partial_cmp, hleft, hright]

theorem bytesmut_eq_str_after_slice_conversions
    (self : bytes.bytes_mut.BytesMut) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq, hleft, hright]

theorem bytesmut_eq_str_characterizes_contents
    (self : bytes.bytes_mut.BytesMut) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    ∃ same,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq, hleft, hright, heq],
    hcontents⟩

theorem bytesmut_partial_cmp_str_after_slice_conversions
    (self : bytes.bytes_mut.BytesMut) (other : Str)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : core.str.Str.as_bytes other = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdStr.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdStr.partial_cmp, hleft, hright]

theorem bytes_eq_vec_after_conversions
    (self : bytes.bytes.Bytes) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqVecU8.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqVecU8.eq,
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq, hleft, hright]

theorem bytes_eq_vec_characterizes_contents
    (self : bytes.bytes.Bytes) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    ∃ same,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqVecU8.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqVecU8.eq,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq, hleft, hright, heq],
    hcontents⟩

theorem bytes_partial_cmp_vec_after_conversions
    (self : bytes.bytes.Bytes) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdVecU8.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialOrdVecU8.partial_cmp, hleft, hright]

theorem bytesmut_eq_vec_after_conversions
    (self : bytes.bytes_mut.BytesMut) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqVecU8.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqVecU8.eq,
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq, hleft, hright]

theorem bytesmut_eq_vec_characterizes_contents
    (self : bytes.bytes_mut.BytesMut) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    ∃ same,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqVecU8.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqVecU8.eq,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq, hleft, hright, heq],
    hcontents⟩

theorem bytesmut_partial_cmp_vec_after_conversions
    (self : bytes.bytes_mut.BytesMut) (other : alloc.vec.Vec Std.U8)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) other () = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdVecU8.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdVecU8.partial_cmp, hleft, hright]

theorem bytes_eq_string_after_conversions
    (self : bytes.bytes.Bytes) (other : String)
    (left : Slice Std.U8) (text : Str) (right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (htext : alloc.string.String.Insts.CoreOpsIndexIndex.index
      core.ops.range.RangeFull.Insts.CoreSliceIndexSliceIndexStrStr other () = ok text)
    (hright : core.str.Str.as_bytes text = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqString.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqString.eq,
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq, hleft, htext, hright]

theorem bytes_eq_string_characterizes_contents
    (self : bytes.bytes.Bytes) (other : String)
    (left : Slice Std.U8) (text : Str) (right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (htext : alloc.string.String.Insts.CoreOpsIndexIndex.index
      core.ops.range.RangeFull.Insts.CoreSliceIndexSliceIndexStrStr other () = ok text)
    (hright : core.str.Str.as_bytes text = ok right) :
    ∃ same,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqString.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqString.eq,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq, hleft, htext, hright, heq],
    hcontents⟩

theorem bytes_partial_cmp_string_after_conversions
    (self : bytes.bytes.Bytes) (other : String)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes.Bytes.as_slice self = ok left)
    (hright : alloc.string.String.as_bytes other = ok right) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialOrdString.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes.Bytes.Insts.CoreCmpPartialOrdString.partial_cmp, hleft, hright]

theorem bytesmut_eq_string_after_conversions
    (self : bytes.bytes_mut.BytesMut) (other : String)
    (left : Slice Std.U8) (text : Str) (right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (htext : alloc.string.String.Insts.CoreOpsIndexIndex.index
      core.ops.range.RangeFull.Insts.CoreSliceIndexSliceIndexStrStr other () = ok text)
    (hright : core.str.Str.as_bytes text = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqString.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqString.eq,
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq, hleft, htext, hright]

theorem bytesmut_eq_string_characterizes_contents
    (self : bytes.bytes_mut.BytesMut) (other : String)
    (left : Slice Std.U8) (text : Str) (right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (htext : alloc.string.String.Insts.CoreOpsIndexIndex.index
      core.ops.range.RangeFull.Insts.CoreSliceIndexSliceIndexStrStr other () = ok text)
    (hright : core.str.Str.as_bytes text = ok right) :
    ∃ same,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqString.eq self other = ok same ∧
      (same = true ↔ left = right) := by
  rcases u8_slice_eq_characterizes_contents left right with ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqString.eq,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq, hleft, htext, hright, heq],
    hcontents⟩

theorem bytesmut_partial_cmp_string_after_conversions
    (self : bytes.bytes_mut.BytesMut) (other : String)
    (left right : Slice Std.U8)
    (hleft : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok left)
    (hright : alloc.string.String.as_bytes other = ok right) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdString.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdString.partial_cmp, hleft, hright]

theorem bytesmut_eq_bytes_after_conversions
    (self : bytes.bytes_mut.BytesMut) (other : bytes.bytes.Bytes)
    (bytes_slice bytes_index mut_slice mut_index : Slice Std.U8)
    (hbytes : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok bytes_slice)
    (hbytes_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) bytes_slice () = ok bytes_index)
    (hmut : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok mut_slice)
    (hmut_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) mut_slice () = ok mut_index) :
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytes.eq self other =
      core.slice.cmp.PartialEqSlice.eq core.cmp.PartialEqU8 bytes_index mut_index := by
  simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytes.eq,
    hbytes, hbytes_index, hmut, hmut_index]

theorem bytesmut_eq_bytes_characterizes_contents
    (self : bytes.bytes_mut.BytesMut) (other : bytes.bytes.Bytes)
    (bytes_slice bytes_index mut_slice mut_index : Slice Std.U8)
    (hbytes : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok bytes_slice)
    (hbytes_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) bytes_slice () = ok bytes_index)
    (hmut : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self = ok mut_slice)
    (hmut_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) mut_slice () = ok mut_index) :
    ∃ same,
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytes.eq self other = ok same ∧
      (same = true ↔ bytes_index = mut_index) := by
  rcases u8_slice_eq_characterizes_contents bytes_index mut_index with
    ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytes.eq,
      hbytes, hbytes_index, hmut, hmut_index, heq], hcontents⟩

theorem bytes_eq_bytesmut_characterizes_contents
    (self : bytes.bytes.Bytes) (other : bytes.bytes_mut.BytesMut)
    (mut_slice mut_index bytes_slice bytes_index : Slice Std.U8)
    (hmut : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok mut_slice)
    (hmut_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) mut_slice () = ok mut_index)
    (hbytes : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref self = ok bytes_slice)
    (hbytes_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) bytes_slice () = ok bytes_index) :
    ∃ same,
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqBytesMut.eq self other = ok same ∧
      (same = true ↔ mut_index = bytes_index) := by
  rcases u8_slice_eq_characterizes_contents mut_index bytes_index with
    ⟨same, heq, hcontents⟩
  exact ⟨same, by
    simp [bytes.bytes.Bytes.Insts.CoreCmpPartialEqBytesMut.eq,
      hmut, hmut_index, hbytes, hbytes_index, heq], hcontents⟩

theorem bytes_bytesmut_eq_is_symmetric
    (self : bytes.bytes.Bytes) (other : bytes.bytes_mut.BytesMut)
    (mut_slice mut_index bytes_slice bytes_index : Slice Std.U8)
    (hmut : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok mut_slice)
    (hmut_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) mut_slice () = ok mut_index)
    (hbytes : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref self = ok bytes_slice)
    (hbytes_index : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) bytes_slice () = ok bytes_index) :
    bytes.bytes.Bytes.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqBytes.eq other self := by
  rcases bytes_eq_bytesmut_characterizes_contents self other
      mut_slice mut_index bytes_slice bytes_index hmut hmut_index hbytes hbytes_index with
    ⟨leftResult, hleft, hleftSpec⟩
  rcases bytesmut_eq_bytes_characterizes_contents other self
      bytes_slice bytes_index mut_slice mut_index hbytes hbytes_index hmut hmut_index with
    ⟨rightResult, hright, hrightSpec⟩
  rw [hleft, hright]
  cases leftResult <;> cases rightResult <;> simp_all [eq_comm]

theorem reverse_slice_eq_bytes_uses_forward_body
    (self : Slice Std.U8) (other : bytes.bytes.Bytes) :
    SliceU8.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqSliceU8.eq other self := by
  rfl

theorem reverse_slice_eq_bytesmut_uses_forward_body
    (self : Slice Std.U8) (other : bytes.bytes_mut.BytesMut) :
    SliceU8.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSliceU8.eq other self := by
  rfl

theorem reverse_slice_partial_cmp_bytes_after_deref
    (self : Slice Std.U8) (other : bytes.bytes.Bytes) (right : Slice Std.U8)
    (hderef : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    SliceU8.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 self right := by
  simp [SliceU8.Insts.CoreCmpPartialOrdBytes.partial_cmp, hderef]

theorem reverse_slice_partial_cmp_bytesmut_after_deref
    (self : Slice Std.U8) (other : bytes.bytes_mut.BytesMut) (right : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    SliceU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 self right := by
  simp [SliceU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp, hderef]

theorem bytes_reverse_str_eq_delegates
    (self : Str) (other : bytes.bytes.Bytes) :
    Str.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr.eq other self := by
  rfl

theorem bytes_reverse_str_partial_cmp_after_conversions
    (self : Str) (other : bytes.bytes.Bytes)
    (left right : Slice Std.U8)
    (hleft : core.str.Str.as_bytes self = ok left)
    (hright : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Str.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [Str.Insts.CoreCmpPartialOrdBytes.partial_cmp, hleft, hright]

theorem bytes_reverse_vec_eq_delegates
    (self : alloc.vec.Vec Std.U8) (other : bytes.bytes.Bytes) :
    alloc.vec.VecU8.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqVecU8.eq other self := by
  rfl

theorem bytes_reverse_vec_partial_cmp_after_conversions
    (self : alloc.vec.Vec Std.U8) (other : bytes.bytes.Bytes)
    (left right : Slice Std.U8)
    (hleft : alloc.vec.Vec.deref self = left)
    (hright : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    alloc.vec.VecU8.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [alloc.vec.VecU8.Insts.CoreCmpPartialOrdBytes.partial_cmp, hleft, hright]

theorem bytes_reverse_string_eq_delegates
    (self : String) (other : bytes.bytes.Bytes) :
    alloc.string.String.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqString.eq other self := by
  rfl

theorem bytes_reverse_string_partial_cmp_after_conversions
    (self : String) (other : bytes.bytes.Bytes)
    (left right : Slice Std.U8)
    (hleft : alloc.string.String.as_bytes self = ok left)
    (hright : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    alloc.string.String.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [alloc.string.String.Insts.CoreCmpPartialOrdBytes.partial_cmp, hleft, hright]

theorem bytesmut_reverse_str_eq_delegates
    (self : Str) (other : bytes.bytes_mut.BytesMut) :
    Str.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr.eq other self := by
  rfl

theorem bytesmut_reverse_str_partial_cmp_after_conversions
    (self : Str) (other : bytes.bytes_mut.BytesMut)
    (left right : Slice Std.U8)
    (hleft : core.str.Str.as_bytes self = ok left)
    (hright : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Str.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [Str.Insts.CoreCmpPartialOrdBytesMut.partial_cmp, hleft, hright]

theorem bytesmut_reverse_vec_eq_delegates
    (self : alloc.vec.Vec Std.U8) (other : bytes.bytes_mut.BytesMut) :
    alloc.vec.VecU8.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqVecU8.eq other self := by
  rfl

theorem bytesmut_reverse_vec_partial_cmp_after_conversions
    (self : alloc.vec.Vec Std.U8) (other : bytes.bytes_mut.BytesMut)
    (left right : Slice Std.U8)
    (hleft : alloc.vec.Vec.index
      (core.slice.index.SliceIndexRangeFullSlice Std.U8) self () = ok left)
    (hright : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    alloc.vec.VecU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 right left := by
  simp [alloc.vec.VecU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp,
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdVecU8.partial_cmp, hleft, hright]

theorem bytesmut_reverse_string_eq_delegates
    (self : String) (other : bytes.bytes_mut.BytesMut) :
    alloc.string.String.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqString.eq other self := by
  rfl

theorem bytesmut_reverse_string_partial_cmp_after_conversions
    (self : String) (other : bytes.bytes_mut.BytesMut)
    (left right : Slice Std.U8)
    (hleft : alloc.string.String.as_bytes self = ok left)
    (hright : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    alloc.string.String.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [alloc.string.String.Insts.CoreCmpPartialOrdBytesMut.partial_cmp, hleft, hright]

theorem bytes_ref_slice_partial_cmp_after_deref
    (self : Slice Std.U8) (other : bytes.bytes.Bytes) (right : Slice Std.U8)
    (hderef : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Shared0SliceU8.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 self right := by
  simp [Shared0SliceU8.Insts.CoreCmpPartialOrdBytes.partial_cmp, hderef]

theorem bytes_ref_str_eq_delegates
    (self : Str) (other : bytes.bytes.Bytes) :
    Shared0Str.Insts.CoreCmpPartialEqBytes.eq self other =
      bytes.bytes.Bytes.Insts.CoreCmpPartialEqSharedAT.eq
        bytes.bytes.Bytes.Insts.CoreCmpPartialEqStr other self := by
  rfl

theorem bytes_ref_str_partial_cmp_after_conversions
    (self : Str) (other : bytes.bytes.Bytes)
    (left right : Slice Std.U8)
    (hleft : core.str.Str.as_bytes self = ok left)
    (hright : bytes.bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Shared0Str.Insts.CoreCmpPartialOrdBytes.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 left right := by
  simp [Shared0Str.Insts.CoreCmpPartialOrdBytes.partial_cmp, hleft, hright]

theorem bytesmut_ref_slice_partial_cmp_after_deref
    (self : Slice Std.U8) (other : bytes.bytes_mut.BytesMut) (right : Slice Std.U8)
    (hderef : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Shared0SliceU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 self right := by
  simp [Shared0SliceU8.Insts.CoreCmpPartialOrdBytesMut.partial_cmp, hderef]

theorem bytesmut_ref_str_eq_delegates
    (self : Str) (other : bytes.bytes_mut.BytesMut) :
    Shared0Str.Insts.CoreCmpPartialEqBytesMut.eq self other =
      bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqSharedAT.eq
        bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialEqStr other self := by
  rfl

theorem bytesmut_ref_str_partial_cmp_after_conversions
    (self : Str) (other : bytes.bytes_mut.BytesMut)
    (left right : Slice Std.U8)
    (hleft : core.str.Str.as_bytes self = ok left)
    (hright : bytes.bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref other = ok right) :
    Shared0Str.Insts.CoreCmpPartialOrdBytesMut.partial_cmp self other =
      Slice.Insts.CoreCmpPartialOrdSlice.partial_cmp core.cmp.PartialOrdU8 right left := by
  simp [Shared0Str.Insts.CoreCmpPartialOrdBytesMut.partial_cmp,
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdSharedAT.partial_cmp,
    bytes.bytes_mut.BytesMut.Insts.CoreCmpPartialOrdStr.partial_cmp, hleft, hright]

#print axioms u8ByteListLexOrder_eq_list_compareLex
#print axioms u8_partial_cmp_is_value_compare

end BytesProofs.Extensions.Comparisons

#print axioms BytesProofs.Extensions.Comparisons.bytes_bytesmut_eq_is_symmetric
#print axioms BytesProofs.Extensions.Comparisons.bytes_eq_bytesmut_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytes_eq_slice_returns_boolean_characterizing_contents
#print axioms BytesProofs.Extensions.Comparisons.bytes_eq_str_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytes_eq_string_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytes_eq_vec_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytes_partial_cmp_slice_after_as_slice
#print axioms BytesProofs.Extensions.Comparisons.bytes_partial_cmp_slice_matches_lexicographic_spec
#print axioms BytesProofs.Extensions.Comparisons.bytes_partial_cmp_str_after_slice_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_partial_cmp_string_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_partial_cmp_vec_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_ref_eq_slice_uses_underlying_inst
#print axioms BytesProofs.Extensions.Comparisons.bytes_ref_partial_cmp_uses_underlying_inst
#print axioms BytesProofs.Extensions.Comparisons.bytes_ref_slice_partial_cmp_after_deref
#print axioms BytesProofs.Extensions.Comparisons.bytes_ref_str_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytes_ref_str_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_str_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_str_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_string_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_string_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_vec_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytes_reverse_vec_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytes_slice_ref_eq_uses_bytes_generic_impl
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_eq_bytes_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_eq_slice_returns_boolean_characterizing_contents
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_eq_str_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_eq_string_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_eq_vec_characterizes_contents
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_partial_cmp_slice_after_deref
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_partial_cmp_slice_matches_lexicographic_spec
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_partial_cmp_str_after_slice_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_partial_cmp_string_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_partial_cmp_vec_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_ref_eq_slice_uses_underlying_inst
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_ref_partial_cmp_uses_underlying_inst
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_ref_slice_partial_cmp_after_deref
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_ref_str_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_ref_str_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_str_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_str_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_string_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_string_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_vec_eq_delegates
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_reverse_vec_partial_cmp_after_conversions
#print axioms BytesProofs.Extensions.Comparisons.bytesmut_slice_ref_eq_uses_bytesmut_generic_impl
#print axioms BytesProofs.Extensions.Comparisons.reverse_slice_eq_bytes_uses_forward_body
#print axioms BytesProofs.Extensions.Comparisons.reverse_slice_eq_bytesmut_uses_forward_body
#print axioms BytesProofs.Extensions.Comparisons.reverse_slice_partial_cmp_bytes_after_deref
#print axioms BytesProofs.Extensions.Comparisons.reverse_slice_partial_cmp_bytesmut_after_deref
