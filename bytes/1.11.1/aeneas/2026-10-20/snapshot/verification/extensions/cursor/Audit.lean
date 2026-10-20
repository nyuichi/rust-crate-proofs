import extensions.cursor.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Cursor

theorem cursor_remaining_is_saturating_sub {T : Type _}
    (asRefInst : core.convert.AsRef T (Slice Std.U8))
    (self : core.io.cursor.Cursor T) (inner : T) (s : Slice Std.U8)
    (position : Std.U64) (positionUsize : Std.Usize)
    (hgetRef : core.io.cursor.Cursor.get_ref self = ok inner)
    (hasRef : asRefInst.as_ref inner = ok s)
    (hposition : core.io.cursor.Cursor.position self = ok position)
    (hconvert : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from
      position = ok (.Ok positionUsize)) :
    core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.remaining asRefInst self =
      ok (core.num.Usize.saturating_sub (Slice.len s) positionUsize) := by
  simp [core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.remaining,
    saturating_sub_usize_u64, hgetRef, hasRef, hposition, hconvert]

theorem cursor_remaining_is_zero_after_end {T : Type _}
    (asRefInst : core.convert.AsRef T (Slice Std.U8))
    (self : core.io.cursor.Cursor T) (inner : T) (s : Slice Std.U8)
    (position : Std.U64) (positionUsize : Std.Usize)
    (hgetRef : core.io.cursor.Cursor.get_ref self = ok inner)
    (hasRef : asRefInst.as_ref inner = ok s)
    (hposition : core.io.cursor.Cursor.position self = ok position)
    (hconvert : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from
      position = ok (.Ok positionUsize))
    (hafter : (Slice.len s).val ≤ positionUsize.val) :
    core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.remaining asRefInst self =
      ok 0#usize := by
  rw [cursor_remaining_is_saturating_sub asRefInst self inner s position
    positionUsize hgetRef hasRef hposition hconvert]
  have hsubval :
      (core.num.Usize.saturating_sub (Slice.len s) positionUsize).val =
        (Slice.len s).val - positionUsize.val := by
    change (BitVec.ofNat _ (max 0 ((Slice.len s).val - positionUsize.val))).toNat = _
    rw [Nat.zero_max, BitVec.toNat_ofNat, Nat.mod_eq_of_lt]
    exact Nat.lt_of_le_of_lt (Nat.sub_le _ _) (Slice.len s).hBounds
  have hsubzero : core.num.Usize.saturating_sub (Slice.len s) positionUsize =
      0#usize := by
    apply UScalar.eq_of_val_eq
    rw [hsubval]
    simpa using (Nat.sub_eq_zero_iff_le.mpr hafter)
  simp [hsubzero]

theorem cursor_remaining_is_zero_when_position_unrepresentable {T : Type _}
    (asRefInst : core.convert.AsRef T (Slice Std.U8))
    (self : core.io.cursor.Cursor T) (inner : T) (s : Slice Std.U8)
    (position : Std.U64) (err : core.num.error.TryFromIntError)
    (hgetRef : core.io.cursor.Cursor.get_ref self = ok inner)
    (hasRef : asRefInst.as_ref inner = ok s)
    (hposition : core.io.cursor.Cursor.position self = ok position)
    (hconvert : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from
      position = ok (.Err err)) :
    core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.remaining asRefInst self =
      ok 0#usize := by
  simp [core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.remaining,
    saturating_sub_usize_u64, hgetRef, hasRef, hposition, hconvert]

theorem cursor_chunk_returns_bounded_suffix {T : Type _}
    (asRefInst : core.convert.AsRef T (Slice Std.U8))
    (self : core.io.cursor.Cursor T) (inner : T) (s : Slice Std.U8)
    (position : Std.U64) (positionUsize : Std.Usize)
    (hgetRef : core.io.cursor.Cursor.get_ref self = ok inner)
    (hasRef : asRefInst.as_ref inner = ok s)
    (hposition : core.io.cursor.Cursor.position self = ok position)
    (hconvert : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from
      position = ok (.Ok positionUsize)) :
    core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.chunk asRefInst self =
      ok (s.drop (core.cmp.impls.OrdUsize.min positionUsize (Slice.len s))) ∧
    (core.cmp.impls.OrdUsize.min positionUsize (Slice.len s)).val =
      min positionUsize.val s.length ∧
    (core.cmp.impls.OrdUsize.min positionUsize (Slice.len s)).val ≤ s.length := by
  let start := core.cmp.impls.OrdUsize.min positionUsize (Slice.len s)
  have hstartVal : start.val = min positionUsize.val s.length := by
    simp [start, Slice.len, Slice.length]
  have hstartBound : start.val ≤ s.length := by
    rw [hstartVal]
    exact Nat.min_le_right _ _
  have hindex :
      core.slice.index.Slice.index
        (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8) s
        { start := start } = ok (s.drop start) := by
    simp [core.slice.index.Slice.index,
      core.slice.index.SliceIndexRangeFromUsizeSlice.index, hstartBound]
  have hindex' :
      core.slice.index.SliceIndexRangeFromUsizeSlice.index
        { start := start } s = ok (s.drop start) := by
    simpa using hindex
  have hindexMin :
      core.slice.index.SliceIndexRangeFromUsizeSlice.index
        { start := core.cmp.impls.OrdUsize.min positionUsize (Slice.len s) } s =
        ok (s.drop (core.cmp.impls.OrdUsize.min positionUsize (Slice.len s))) := by
    simpa [start] using hindex'
  have hchunk : core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.chunk
      asRefInst self = ok (s.drop start) := by
    simp [core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.chunk,
      hgetRef, hasRef, hposition, min_u64_usize, hconvert, start, hindexMin]
  refine ⟨hchunk, ?_, ?_⟩
  · simp
  · simp

theorem cursor_chunk_is_empty_when_position_unrepresentable {T : Type _}
    (asRefInst : core.convert.AsRef T (Slice Std.U8))
    (self : core.io.cursor.Cursor T) (inner : T) (s : Slice Std.U8)
    (position : Std.U64) (err : core.num.error.TryFromIntError)
    (hgetRef : core.io.cursor.Cursor.get_ref self = ok inner)
    (hasRef : asRefInst.as_ref inner = ok s)
    (hposition : core.io.cursor.Cursor.position self = ok position)
    (hconvert : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from
      position = ok (.Err err)) :
    core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.chunk asRefInst self =
      ok (s.drop (Slice.len s)) := by
  simp [core.io.cursor.Cursor.Insts.BytesBufBuf_implBuf.chunk,
    min_u64_usize, hgetRef, hasRef, hposition, hconvert,
    core.slice.index.Slice.index,
    core.slice.index.SliceIndexRangeFromUsizeSlice.index,
    Slice.len, Slice.length]

end BytesProofs.Cursor

#print axioms BytesProofs.Cursor.cursor_chunk_is_empty_when_position_unrepresentable
#print axioms BytesProofs.Cursor.cursor_chunk_returns_bounded_suffix
#print axioms BytesProofs.Cursor.cursor_remaining_is_saturating_sub
#print axioms BytesProofs.Cursor.cursor_remaining_is_zero_after_end
#print axioms BytesProofs.Cursor.cursor_remaining_is_zero_when_position_unrepresentable
