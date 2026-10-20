import extensions.core.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Core

/-!
These are conditional contracts for transparent functions in the actual
Aeneas extraction. Conditions expose only results of frozen opaque callees or
of fallible scalar operations; no implementation model is substituted.
-/

theorem bytes_copy_from_slice_returns_converted_vector
    (data : Slice Std.U8) (vec : alloc.vec.Vec Std.U8)
    (out : bytes.Bytes)
    (hvec : alloc.slice.Slice.to_vec core.clone.CloneU8 data = ok vec)
    (hconvert : bytes.Bytes.Insts.CoreConvertFromVecU8.from vec = ok out) :
    bytes.Bytes.copy_from_slice data = ok out := by
  simp [bytes.Bytes.copy_from_slice, hvec, hconvert]

/-- The generated `from_vec` body copies the pointer, length, and capacity
    returned by its explicit vector operations into the `BytesMut` fields.
    These hypotheses provide results only for the frozen pointer and
    `ManuallyDrop` operations on the executed path. -/
theorem bytes_mut_from_vec_records_vector_metadata
    (vec : alloc.vec.Vec Std.U8)
    (md : core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (v : alloc.vec.Vec Std.U8)
    (back : alloc.vec.Vec Std.U8 →
      core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (rawPtr : MutRawPtr Std.U8) (v1 : alloc.vec.Vec Std.U8)
    (ptr : core.ptr.non_null.NonNull Std.U8)
    (v2 : alloc.vec.Vec Std.U8)
    (len cap repr shifted encoded : Std.Usize)
    (rawData : MutRawPtr bytes_mut.Shared)
    (hnew : core.mem.manually_drop.ManuallyDrop.new vec = ok md)
    (hderefMut :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDerefMut.deref_mut
        md = ok (v, back))
    (hasMutPtr : alloc.vec.Vec.as_mut_ptr alloc.alloc.Global v =
      ok (rawPtr, v1))
    (hvptr : bytes_mut.vptr rawPtr = ok ptr)
    (hderef :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDeref.deref
        (back v1) = ok v2)
    (hlen : alloc.vec.Vec.len v2 = len)
    (hcap : alloc.vec.Vec.capacity alloc.alloc.Global v2 = ok cap)
    (hrepr : bytes_mut.original_capacity_to_repr cap = ok repr)
    (hshift : repr <<< bytes_mut.ORIGINAL_CAPACITY_OFFSET = ok shifted)
    (hencode : lift (shifted ||| bytes_mut.KIND_VEC) = ok encoded)
    (hinvalid : bytes_mut.invalid_ptr bytes_mut.Shared encoded = ok rawData) :
    bytes_mut.BytesMut.from_vec vec =
      ok { ptr := ptr, len := len, cap := cap, data := rawData } := by
  simp [bytes_mut.BytesMut.from_vec, hnew, hderefMut, hasMutPtr, hvptr,
    hderef, hlen, hcap, hrepr, hshift, hencode, hinvalid]

theorem bytes_mut_new_uses_zero_capacity_constructor :
    bytes_mut.BytesMut.new =
      bytes_mut.BytesMut.with_capacity 0#usize := by
  rfl

theorem bytes_mut_split_uses_current_length
    (self leftPart suffix : bytes_mut.BytesMut)
    (hsplit : bytes_mut.BytesMut.split_to self self.len =
      ok (leftPart, suffix)) :
    bytes_mut.BytesMut.split self = ok (leftPart, suffix) := by
  simp [bytes_mut.BytesMut.split, bytes_mut.BytesMut.impl.len, hsplit]

theorem bytes_mut_zeroed_builds_zero_vector
    (len : Std.Usize) (vec : alloc.vec.Vec Std.U8)
    (out : bytes_mut.BytesMut)
    (hvec : alloc.vec.from_elem core.clone.CloneU8 0#u8 len = ok vec)
    (hfrom : bytes_mut.BytesMut.from_vec vec = ok out) :
    bytes_mut.BytesMut.zeroed len = ok out ∧
      vec.val = List.replicate len.val 0#u8 ∧ vec.length = len.val := by
  have hclone : core.clone.CloneU8.clone 0#u8 = ok 0#u8 := rfl
  have hspec := alloc.vec.from_elem_spec core.clone.CloneU8 0#u8 len hclone
  have hprops : vec.val = List.replicate len.val 0#u8 ∧
      vec.length = len.val := by
    rw [hvec] at hspec
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  constructor
  · simp [bytes_mut.BytesMut.zeroed, hvec, hfrom]
  · exact hprops

/-- `zeroed` preserves the requested logical length through the actual
    `from_vec` body. The premises expose the abstract pointer/allocator
    operation results needed to execute that body. -/
theorem bytes_mut_zeroed_returns_requested_length
    (len : Std.Usize) (vec : alloc.vec.Vec Std.U8)
    (md : core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (v : alloc.vec.Vec Std.U8)
    (back : alloc.vec.Vec Std.U8 →
      core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (rawPtr : MutRawPtr Std.U8) (v1 : alloc.vec.Vec Std.U8)
    (ptr : core.ptr.non_null.NonNull Std.U8)
    (cap repr shifted encoded : Std.Usize)
    (rawData : MutRawPtr bytes_mut.Shared)
    (hvec : alloc.vec.from_elem core.clone.CloneU8 0#u8 len = ok vec)
    (hnew : core.mem.manually_drop.ManuallyDrop.new vec = ok md)
    (hderefMut :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDerefMut.deref_mut
        md = ok (v, back))
    (hasMutPtr : alloc.vec.Vec.as_mut_ptr alloc.alloc.Global v =
      ok (rawPtr, v1))
    (hvptr : bytes_mut.vptr rawPtr = ok ptr)
    (hderef :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDeref.deref
        (back v1) = ok vec)
    (hcap : alloc.vec.Vec.capacity alloc.alloc.Global vec = ok cap)
    (hrepr : bytes_mut.original_capacity_to_repr cap = ok repr)
    (hshift : repr <<< bytes_mut.ORIGINAL_CAPACITY_OFFSET = ok shifted)
    (hencode : lift (shifted ||| bytes_mut.KIND_VEC) = ok encoded)
    (hinvalid : bytes_mut.invalid_ptr bytes_mut.Shared encoded = ok rawData) :
    bytes_mut.BytesMut.zeroed len =
      ok { ptr := ptr, len := len, cap := cap, data := rawData } := by
  have hclone : core.clone.CloneU8.clone 0#u8 = ok 0#u8 := rfl
  have hspec := alloc.vec.from_elem_spec core.clone.CloneU8 0#u8 len hclone
  have hprops : vec.val = List.replicate len.val 0#u8 ∧
      vec.length = len.val := by
    rw [hvec] at hspec
    simpa [Aeneas.Std.WP.spec_ok] using hspec
  have hlen : alloc.vec.Vec.len vec = len := by
    scalar_tac
  have hfrom := bytes_mut_from_vec_records_vector_metadata
    vec md v back rawPtr v1 ptr vec len cap repr shifted encoded rawData
    hnew hderefMut hasMutPtr hvptr hderef hlen hcap hrepr hshift hencode hinvalid
  have hzeroed := bytes_mut_zeroed_builds_zero_vector len vec
    { ptr := ptr, len := len, cap := cap, data := rawData } hvec hfrom
  exact hzeroed.1

/-- Every successful `with_capacity` path sets the logical length to zero.
    Its pointer and capacity outcomes remain explicit parameters of the
    generated `from_vec` path. -/
theorem bytes_mut_with_capacity_returns_empty_buffer
    (requested : Std.Usize) (vec : alloc.vec.Vec Std.U8)
    (md : core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (v : alloc.vec.Vec Std.U8)
    (back : alloc.vec.Vec Std.U8 →
      core.mem.manually_drop.ManuallyDrop (alloc.vec.Vec Std.U8))
    (rawPtr : MutRawPtr Std.U8) (v1 : alloc.vec.Vec Std.U8)
    (ptr : core.ptr.non_null.NonNull Std.U8)
    (cap repr shifted encoded : Std.Usize)
    (rawData : MutRawPtr bytes_mut.Shared)
    (hvec : alloc.vec.Vec.with_capacity Std.U8 requested = vec)
    (hnew : core.mem.manually_drop.ManuallyDrop.new vec = ok md)
    (hderefMut :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDerefMut.deref_mut
        md = ok (v, back))
    (hasMutPtr : alloc.vec.Vec.as_mut_ptr alloc.alloc.Global v =
      ok (rawPtr, v1))
    (hvptr : bytes_mut.vptr rawPtr = ok ptr)
    (hderef :
      core.mem.manually_drop.ManuallyDrop.Insts.CoreOpsDerefDeref.deref
        (back v1) = ok vec)
    (hcap : alloc.vec.Vec.capacity alloc.alloc.Global vec = ok cap)
    (hrepr : bytes_mut.original_capacity_to_repr cap = ok repr)
    (hshift : repr <<< bytes_mut.ORIGINAL_CAPACITY_OFFSET = ok shifted)
    (hencode : lift (shifted ||| bytes_mut.KIND_VEC) = ok encoded)
    (hinvalid : bytes_mut.invalid_ptr bytes_mut.Shared encoded = ok rawData) :
    bytes_mut.BytesMut.with_capacity requested =
      ok { ptr := ptr, len := 0#usize, cap := cap, data := rawData } := by
  have hlen : alloc.vec.Vec.len vec = 0#usize := by
    rw [← hvec]
    simp [alloc.vec.Vec.with_capacity, alloc.vec.Vec.new, alloc.vec.Vec.len]
    scalar_tac
  have hfrom := bytes_mut_from_vec_records_vector_metadata
    vec md v back rawPtr v1 ptr vec 0#usize cap repr shifted encoded rawData
    hnew hderefMut hasMutPtr hvptr hderef hlen hcap hrepr hshift hencode hinvalid
  simp [bytes_mut.BytesMut.with_capacity, hvec, hfrom]

theorem bytes_clear_truncates_to_zero
    (self updated : bytes.Bytes)
    (htruncate : bytes.Bytes.truncate self 0#usize = ok updated) :
    bytes.Bytes.clear self = ok updated := by
  simp [bytes.Bytes.clear, htruncate]

theorem bytes_buf_remaining_returns_length (self : bytes.Bytes) :
    bytes.Bytes.Insts.BytesBufBuf_implBuf.remaining self = ok self.len := by
  rfl

theorem bytes_buf_chunk_returns_as_slice
    (self : bytes.Bytes) (chunk : Slice Std.U8)
    (has_slice : bytes.Bytes.as_slice self = ok chunk) :
    bytes.Bytes.Insts.BytesBufBuf_implBuf.chunk self = ok chunk := by
  simp [bytes.Bytes.Insts.BytesBufBuf_implBuf.chunk, has_slice]

theorem bytes_buf_copy_to_bytes_returns_split
    (self consumed remainder : bytes.Bytes) (n : Std.Usize)
    (hsplit : bytes.Bytes.split_to self n = ok (consumed, remainder)) :
    bytes.Bytes.Insts.BytesBufBuf_implBuf.copy_to_bytes self n =
      ok (consumed, remainder) := by
  simp [bytes.Bytes.Insts.BytesBufBuf_implBuf.copy_to_bytes, hsplit]

theorem bytes_mut_default_is_zero_capacity_construction :
    bytes_mut.BytesMut.Insts.CoreDefaultDefault.default =
      bytes_mut.BytesMut.with_capacity 0#usize := by
  simp [bytes_mut.BytesMut.Insts.CoreDefaultDefault.default,
    bytes_mut.BytesMut.new]

theorem bytes_mut_deref_returns_as_slice
    (self : bytes_mut.BytesMut) (contents : Slice Std.U8)
    (has_slice : bytes_mut.BytesMut.as_slice self = ok contents) :
    bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref self =
      ok contents := by
  simp [bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref,
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref, has_slice]

theorem bytes_mut_clone_converts_full_slice
    (self : bytes_mut.BytesMut) (contents : Slice Std.U8)
    (copy : bytes_mut.BytesMut)
    (has_slice : bytes_mut.BytesMut.as_slice self = ok contents)
    (hfrom :
      bytes_mut.BytesMut.Insts.CoreConvertFromSharedASliceU8.from contents =
        ok copy) :
    bytes_mut.BytesMut.Insts.CoreCloneClone.clone self = ok copy := by
  simp [bytes_mut.BytesMut.Insts.CoreCloneClone.clone,
    bytes_mut.BytesMut.Insts.CoreOpsDerefDerefSliceU8.deref,
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref,
    core.slice.index.Slice.index, has_slice, hfrom]

theorem bytes_try_into_mut_when_unique
    (self : bytes.Bytes) (mutable : bytes_mut.BytesMut)
    (hunique : bytes.Bytes.is_unique self = ok true)
    (hconvert :
      bytes_mut.BytesMut.Insts.CoreConvertFromBytes.from self = ok mutable) :
    bytes.Bytes.try_into_mut self =
      ok (core.result.Result.Ok mutable) := by
  simp [bytes.Bytes.try_into_mut, hunique, hconvert]

theorem bytes_try_into_mut_when_shared
    (self : bytes.Bytes)
    (hshared : bytes.Bytes.is_unique self = ok false) :
    bytes.Bytes.try_into_mut self =
      ok (core.result.Result.Err self) := by
  simp [bytes.Bytes.try_into_mut, hshared]

theorem bytes_mut_reserve_noop_when_additional_fits
    (self : bytes_mut.BytesMut) (additional rem : Std.Usize)
    (hsub : self.cap - self.len = ok rem)
    (hfits : additional ≤ rem) :
    bytes_mut.BytesMut.reserve self additional = ok self := by
  have hnat : ¬ rem.val < additional.val := by
    intro h
    scalar_tac
  simp [bytes_mut.BytesMut.reserve, bytes_mut.BytesMut.impl.len,
    bytes_mut.BytesMut.capacity, hsub, hnat]

/-- If the requested additional capacity already fits, `reserve` returns
    the exact original handle. Consequently every successful `as_slice` view
    of that handle is also the view of the returned buffer. -/
theorem bytes_mut_reserve_fit_preserves_slice_observation
    (self : bytes_mut.BytesMut) (additional rem : Std.Usize)
    (hsub : self.cap - self.len = ok rem)
    (hfits : additional ≤ rem) :
    ∃ reserved : bytes_mut.BytesMut,
      bytes_mut.BytesMut.reserve self additional = ok reserved ∧
      reserved.len = self.len ∧ reserved.cap = self.cap ∧
      bytes_mut.BytesMut.as_slice reserved = bytes_mut.BytesMut.as_slice self := by
  have hreserve : bytes_mut.BytesMut.reserve self additional = ok self :=
    bytes_mut_reserve_noop_when_additional_fits self additional rem hsub hfits
  exact ⟨self, hreserve, rfl, rfl, rfl⟩

theorem bytes_mut_reserve_returns_inner_state_when_additional_does_not_fit
    (self updated : bytes_mut.BytesMut) (additional rem : Std.Usize)
    (didGrow : Bool)
    (hsub : self.cap - self.len = ok rem)
    (hdoesNotFit : ¬ additional ≤ rem)
    (hinner : bytes_mut.BytesMut.reserve_inner self additional true =
      ok (didGrow, updated)) :
    bytes_mut.BytesMut.reserve self additional = ok updated := by
  have hnat : rem.val < additional.val := by
    scalar_tac
  simp [bytes_mut.BytesMut.reserve, bytes_mut.BytesMut.impl.len,
    bytes_mut.BytesMut.capacity, hsub, hnat, hinner]

theorem bytes_mut_try_reclaim_reports_success_without_changing_state
    (self : bytes_mut.BytesMut) (additional rem : Std.Usize)
    (hsub : self.cap - self.len = ok rem)
    (hfits : additional ≤ rem) :
    bytes_mut.BytesMut.try_reclaim self additional = ok (true, self) := by
  have hnat : ¬ rem.val < additional.val := by
    intro h
    scalar_tac
  simp [bytes_mut.BytesMut.try_reclaim, bytes_mut.BytesMut.impl.len,
    bytes_mut.BytesMut.capacity, hsub, hnat]

theorem bytes_mut_try_reclaim_delegates_when_additional_does_not_fit
    (self : bytes_mut.BytesMut) (additional rem : Std.Usize)
    (hsub : self.cap - self.len = ok rem)
    (hdoesNotFit : ¬ additional ≤ rem) :
    bytes_mut.BytesMut.try_reclaim self additional =
      bytes_mut.BytesMut.reserve_inner self additional false := by
  have hnat : rem.val < additional.val := by
    scalar_tac
  simp [bytes_mut.BytesMut.try_reclaim, bytes_mut.BytesMut.impl.len,
    bytes_mut.BytesMut.capacity, hsub, hnat]

theorem bytes_mut_unsplit_empty_left_returns_other
    (self other : bytes_mut.BytesMut) (hempty : self.len = 0#usize) :
    bytes_mut.BytesMut.unsplit self other = ok other := by
  simp [bytes_mut.BytesMut.unsplit, bytes_mut.BytesMut.is_empty, hempty]

theorem bytes_mut_unsplit_uses_successful_in_place_join
    (self other joined : bytes_mut.BytesMut)
    (hnonempty : self.len ≠ 0#usize)
    (hjoin : bytes_mut.BytesMut.try_unsplit self other =
      ok (core.result.Result.Ok (), joined)) :
    bytes_mut.BytesMut.unsplit self other = ok joined := by
  simp [bytes_mut.BytesMut.unsplit, bytes_mut.BytesMut.is_empty,
    hnonempty, hjoin]

theorem bytes_mut_unsplit_extends_after_failed_in_place_join
    (self other self1 other1 joined : bytes_mut.BytesMut)
    (s : Slice Std.U8) (hnonempty : self.len ≠ 0#usize)
    (hjoin : bytes_mut.BytesMut.try_unsplit self other =
      ok (core.result.Result.Err other1, self1))
    (has_ref : bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref other1 =
      ok s)
    (hextend : bytes_mut.BytesMut.extend_from_slice self1 s = ok joined) :
    bytes_mut.BytesMut.unsplit self other = ok joined := by
  simp [bytes_mut.BytesMut.unsplit, bytes_mut.BytesMut.is_empty,
    hnonempty, hjoin, has_ref, hextend]

theorem bytes_mut_buf_remaining_returns_length
    (self : bytes_mut.BytesMut) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_implBuf.remaining self =
      ok self.len := by
  rfl

theorem bytes_mut_buf_chunk_returns_as_slice
    (self : bytes_mut.BytesMut) (chunk : Slice Std.U8)
    (has_slice : bytes_mut.BytesMut.as_slice self = ok chunk) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_implBuf.chunk self = ok chunk := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_implBuf.chunk, has_slice]

theorem bytes_mut_as_ref_returns_as_slice
    (self : bytes_mut.BytesMut) (contents : Slice Std.U8)
    (has_slice : bytes_mut.BytesMut.as_slice self = ok contents) :
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref self =
      ok contents := by
  simp [bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref, has_slice]

theorem bytes_mut_copy_to_bytes_returns_frozen_split
    (self prefixBuf remainder : bytes_mut.BytesMut) (n : Std.Usize)
    (frozen : bytes.Bytes)
    (hsplit : bytes_mut.BytesMut.split_to self n = ok (prefixBuf, remainder))
    (hfreeze : bytes_mut.BytesMut.freeze prefixBuf = ok frozen) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_implBuf.copy_to_bytes self n =
      ok (frozen, remainder) := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_implBuf.copy_to_bytes,
    hsplit, hfreeze]

theorem bytes_ord_matches_slice_order
    (self other : bytes.Bytes) (selfSlice otherSlice : Slice Std.U8)
    (ordering : Ordering)
    (hself : bytes.Bytes.as_slice self = ok selfSlice)
    (hother : bytes.Bytes.as_slice other = ok otherSlice)
    (hcmp : Slice.Insts.CoreCmpOrd.cmp core.cmp.OrdU8 selfSlice otherSlice =
      ok ordering) :
    bytes.Bytes.Insts.CoreCmpOrd.cmp self other = ok ordering := by
  simp [bytes.Bytes.Insts.CoreCmpOrd.cmp, hself, hother, hcmp]

theorem bytes_mut_ord_matches_slice_order
    (self other : bytes_mut.BytesMut)
    (selfSlice otherSlice : Slice Std.U8) (ordering : Ordering)
    (hself : bytes_mut.BytesMut.as_slice self = ok selfSlice)
    (hother : bytes_mut.BytesMut.as_slice other = ok otherSlice)
    (hcmp : Slice.Insts.CoreCmpOrd.cmp core.cmp.OrdU8 selfSlice otherSlice =
      ok ordering) :
    bytes_mut.BytesMut.Insts.CoreCmpOrd.cmp self other = ok ordering := by
  simp [bytes_mut.BytesMut.Insts.CoreCmpOrd.cmp, hself, hother, hcmp]

#print axioms bytes_copy_from_slice_returns_converted_vector
#print axioms bytes_mut_from_vec_records_vector_metadata
#print axioms bytes_mut_new_uses_zero_capacity_constructor
#print axioms bytes_mut_split_uses_current_length
#print axioms bytes_mut_zeroed_builds_zero_vector
#print axioms bytes_mut_zeroed_returns_requested_length
#print axioms bytes_mut_with_capacity_returns_empty_buffer
#print axioms bytes_clear_truncates_to_zero
#print axioms bytes_buf_remaining_returns_length
#print axioms bytes_buf_chunk_returns_as_slice
#print axioms bytes_buf_copy_to_bytes_returns_split
#print axioms bytes_mut_default_is_zero_capacity_construction
#print axioms bytes_mut_deref_returns_as_slice
#print axioms bytes_mut_clone_converts_full_slice
#print axioms bytes_try_into_mut_when_unique
#print axioms bytes_try_into_mut_when_shared
#print axioms bytes_mut_reserve_noop_when_additional_fits
#print axioms bytes_mut_reserve_fit_preserves_slice_observation
#print axioms bytes_mut_reserve_returns_inner_state_when_additional_does_not_fit
#print axioms bytes_mut_try_reclaim_reports_success_without_changing_state
#print axioms bytes_mut_try_reclaim_delegates_when_additional_does_not_fit
#print axioms bytes_mut_unsplit_empty_left_returns_other
#print axioms bytes_mut_unsplit_uses_successful_in_place_join
#print axioms bytes_mut_unsplit_extends_after_failed_in_place_join
#print axioms bytes_mut_buf_remaining_returns_length
#print axioms bytes_mut_buf_chunk_returns_as_slice
#print axioms bytes_mut_as_ref_returns_as_slice
#print axioms bytes_mut_copy_to_bytes_returns_frozen_split
#print axioms bytes_ord_matches_slice_order
#print axioms bytes_mut_ord_matches_slice_order

end BytesProofs.Core
