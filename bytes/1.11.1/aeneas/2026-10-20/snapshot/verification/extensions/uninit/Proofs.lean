import extensions.uninit.Generated

open Aeneas Aeneas.Std Result

namespace BytesProofs.Uninit

abbrev UninitSlice := bytes.buf.uninit_slice.UninitSlice
abbrev MaybeByte := bytes.core.mem.maybe_uninit.MaybeUninit Std.U8
abbrev UninitBytes := Slice MaybeByte

/-! The four range implementations available as concrete Aeneas slice models.
    The hypotheses below describe only the existing opaque `uninit_ref` and
    `uninit` conversions; the selected slices and their index/update behavior
    are reduced from the actual Aeneas slice-index implementations. -/

theorem index_range
    (self : UninitSlice) (r : core.ops.range.Range Usize)
    (hstart : r.start ≤ r.end) (hend : r.end ≤ self.length)
    (converted : UninitSlice)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref
      (Slice.from (self.val.slice r.start r.end) (by scalar_tac)) = ok converted)
    (hcontents : converted.val = self.val.slice r.start r.end) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeUsizeUninitSlice.index
      self r = ok converted ∧
    converted.val = self.val.slice r.start r.end ∧
    converted.length = r.end.val - r.start.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeUsizeUninitSlice.index,
      core.slice.index.Slice.index, core.slice.index.SliceIndexRangeUsizeSlice.index,
      Slice.length, hstart, hend, hconvert]
  constructor
  · exact hcontents
  · simp [Slice.length, hcontents]; simp_lists; scalar_tac

theorem index_from
    (self : UninitSlice) (r : core.ops.range.RangeFrom Usize)
    (hstart : r.start ≤ self.length)
    (converted : UninitSlice)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref (self.drop r.start) = ok converted)
    (hcontents : converted.val = self.val.drop r.start.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeFromUsizeUninitSlice.index
      self r = ok converted ∧
    converted.val = self.val.drop r.start.val ∧
    converted.length = self.length - r.start.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeFromUsizeUninitSlice.index,
      core.slice.index.Slice.index, core.slice.index.SliceIndexRangeFromUsizeSlice.index,
      Slice.length, hstart, hconvert]
  constructor
  · exact hcontents
  · simp [Slice.length, hcontents]

theorem index_full
    (self : UninitSlice) (r : core.ops.range.RangeFull)
    (converted : UninitSlice)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref self = ok converted)
    (hcontents : converted.val = self.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeFullUninitSlice.index
      self r = ok converted ∧
    converted.val = self.val ∧ converted.length = self.length := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeFullUninitSlice.index,
      core.slice.index.Slice.index, core.slice.index.SliceIndexRangeFullSlice.index, hconvert]
  constructor
  · exact hcontents
  · simp [Slice.length, hcontents]

theorem index_to
    (self : UninitSlice) (r : core.ops.range.RangeTo Usize)
    (hend : r.end ≤ self.length)
    (converted : UninitSlice)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref
      (Slice.from (self.val.take r.end.val) (by scalar_tac)) = ok converted)
    (hcontents : converted.val = self.val.take r.end.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeToUsizeUninitSlice.index
      self r = ok converted ∧
    converted.val = self.val.take r.end.val ∧ converted.length = r.end.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeToUsizeUninitSlice.index,
      core.slice.index.Slice.index, core.slice.index.SliceIndexRangeToUsizeSlice.index,
      Slice.length, hend, hconvert]
  constructor
  · exact hcontents
  · simp [Slice.length, hcontents]; simp_lists; scalar_tac

theorem index_mut_range
    (self : UninitSlice) (r : core.ops.range.Range Usize)
    (hstart : r.start ≤ r.end) (hend : r.end ≤ self.length)
    (converted : UninitSlice) (convertBack : UninitSlice → UninitBytes)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit
      (Slice.from (self.val.slice r.start r.end) (by scalar_tac)) =
        ok (converted, convertBack))
    (hconvertBack : ∀ replacement : UninitSlice,
      (convertBack replacement).val = replacement.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeUsizeUninitSlice.index_mut
      self r = ok (converted, fun replacement =>
        Slice.from (self.val.setSlice! r.start.val replacement.val) (by simp)) := by
  simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeUsizeUninitSlice.index_mut,
    core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeUsizeSlice.index_mut,
    Slice.length, hstart, hend, hconvert, hconvertBack]

theorem index_mut_from
    (self : UninitSlice) (r : core.ops.range.RangeFrom Usize)
    (hstart : r.start ≤ self.length)
    (converted : UninitSlice) (convertBack : UninitSlice → UninitBytes)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit (self.drop r.start) =
      ok (converted, convertBack))
    (hconvertBack : ∀ replacement : UninitSlice,
      (convertBack replacement).val = replacement.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeFromUsizeUninitSlice.index_mut
      self r = ok (converted, fun replacement =>
        Slice.from (self.val.setSlice! r.start.val replacement.val) (by simp)) := by
  simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeFromUsizeUninitSlice.index_mut,
    core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeFromUsizeSlice.index_mut,
    Slice.length, hstart, hconvert, hconvertBack]

theorem index_mut_full
    (self : UninitSlice) (_r : core.ops.range.RangeFull)
    (converted : UninitSlice) (convertBack : UninitSlice → UninitBytes)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit self = ok (converted, convertBack)) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeFullUninitSlice.index_mut
      self _r = ok (converted, convertBack) := by
  simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeFullUninitSlice.index_mut,
    core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeFullSlice.index_mut,
    hconvert]

theorem index_mut_to
    (self : UninitSlice) (r : core.ops.range.RangeTo Usize)
    (hend : r.end ≤ self.length)
    (converted : UninitSlice) (convertBack : UninitSlice → UninitBytes)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit
      (Slice.from (self.val.take r.end.val) (by scalar_tac)) = ok (converted, convertBack))
    (hconvertBack : ∀ replacement : UninitSlice,
      (convertBack replacement).val = replacement.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeToUsizeUninitSlice.index_mut
      self r = ok (converted, fun replacement =>
        Slice.from (self.val.setSlice! 0 replacement.val) (by simp)) := by
  simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeToUsizeUninitSlice.index_mut,
    core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice.index_mut,
    Slice.length, hend, hconvert, hconvertBack]

/-! The Rust standard-library `SliceIndex` methods for inclusive ranges are
    frozen opaque inputs. These two contracts verify the transparent bytes
    wrappers conditionally on successful underlying indexing and its mutation
    law; the preconditions deliberately expose those callback facts. -/

theorem index_inclusive
    (self : UninitSlice) (r : core.ops.range.RangeInclusive Usize)
    (selected converted : UninitSlice)
    (hindex : core.slice.index.Slice.index
      (bytes.core.ops.range.RangeInclusiveUsize.Insts.CoreSliceIndexSliceIndexSliceSlice MaybeByte)
      self r = ok selected)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref selected = ok converted)
    (hcontents : converted.val = selected.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeInclusiveUsizeUninitSlice.index
      self r = ok converted ∧ converted.val = selected.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeInclusiveUsizeUninitSlice.index,
      hindex, hconvert]
  · exact hcontents

theorem index_mut_inclusive
    (self : UninitSlice) (r : core.ops.range.RangeInclusive Usize)
    (selected converted : UninitSlice)
    (sliceBack : UninitSlice → UninitBytes)
    (convertBack : UninitSlice → UninitBytes)
    (hindex : core.slice.index.Slice.index_mut
      (bytes.core.ops.range.RangeInclusiveUsize.Insts.CoreSliceIndexSliceIndexSliceSlice MaybeByte)
      self r = ok (selected, sliceBack))
    (hindexBack : ∀ replacement,
      (sliceBack replacement).val = self.val.setSlice! r.start.val replacement.val)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit selected = ok (converted, convertBack))
    (hconvertBack : ∀ replacement, (convertBack replacement).val = replacement.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeInclusiveUsizeUninitSlice.index_mut
      self r = ok (converted, fun replacement => sliceBack (convertBack replacement)) ∧
    ∀ replacement,
      (sliceBack (convertBack replacement)).val = self.val.setSlice! r.start.val replacement.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeInclusiveUsizeUninitSlice.index_mut,
      hindex, hconvert]
  · intro replacement
    rw [hindexBack, hconvertBack]

theorem index_to_inclusive
    (self : UninitSlice) (r : bytes.core.ops.range.RangeToInclusive Usize)
    (selected converted : UninitSlice)
    (hindex : core.slice.index.Slice.index
      (bytes.core.ops.range.RangeToInclusiveUsize.Insts.CoreSliceIndexSliceIndexSliceSlice MaybeByte)
      self r = ok selected)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit_ref selected = ok converted)
    (hcontents : converted.val = selected.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeToInclusiveUsizeUninitSlice.index
      self r = ok converted ∧ converted.val = selected.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexRangeToInclusiveUsizeUninitSlice.index,
      hindex, hconvert]
  · exact hcontents

theorem index_mut_to_inclusive
    (self : UninitSlice) (r : bytes.core.ops.range.RangeToInclusive Usize)
    (selected converted : UninitSlice)
    (sliceBack : UninitSlice → UninitBytes)
    (convertBack : UninitSlice → UninitBytes)
    (hindex : core.slice.index.Slice.index_mut
      (bytes.core.ops.range.RangeToInclusiveUsize.Insts.CoreSliceIndexSliceIndexSliceSlice MaybeByte)
      self r = ok (selected, sliceBack))
    (hindexBack : ∀ replacement,
      (sliceBack replacement).val = self.val.setSlice! 0 replacement.val)
    (hconvert : bytes.buf.uninit_slice.UninitSlice.uninit selected = ok (converted, convertBack))
    (hconvertBack : ∀ replacement, (convertBack replacement).val = replacement.val) :
    bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeToInclusiveUsizeUninitSlice.index_mut
      self r = ok (converted, fun replacement => sliceBack (convertBack replacement)) ∧
    ∀ replacement,
      (sliceBack (convertBack replacement)).val = self.val.setSlice! 0 replacement.val := by
  constructor
  · simp [bytes.buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeToInclusiveUsizeUninitSlice.index_mut,
      hindex, hconvert]
  · intro replacement
    rw [hindexBack, hconvertBack]

end BytesProofs.Uninit

#print axioms BytesProofs.Uninit.index_range
#print axioms BytesProofs.Uninit.index_from
#print axioms BytesProofs.Uninit.index_full
#print axioms BytesProofs.Uninit.index_to
#print axioms BytesProofs.Uninit.index_mut_range
#print axioms BytesProofs.Uninit.index_mut_from
#print axioms BytesProofs.Uninit.index_mut_full
#print axioms BytesProofs.Uninit.index_mut_to
#print axioms BytesProofs.Uninit.index_inclusive
#print axioms BytesProofs.Uninit.index_mut_inclusive
#print axioms BytesProofs.Uninit.index_to_inclusive
#print axioms BytesProofs.Uninit.index_mut_to_inclusive
