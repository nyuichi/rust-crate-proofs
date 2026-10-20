import extensions.concrete_buffers.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Extensions.ConcreteBuffers

private def isizeMaxAsUsize : Std.Usize :=
  IScalar.hcast .Usize core.num.Isize.MAX

private theorem isize_max_fits_usize : isizeMaxAsUsize.val ≤ Std.Usize.max := by
  scalar_tac

private theorem list_mapM_u8_is_identity (xs : List Std.U8) :
    List.mapM_with_length (Aeneas.Std.liftFun1 core.clone.impls.CloneU8.clone) xs =
      ok ⟨xs, rfl⟩ := by
  induction xs with
  | nil => rfl
  | cons x xs ih =>
    simp [List.mapM_with_length, Aeneas.Std.liftFun1,
      core.clone.impls.CloneU8.clone, ih, bind_tc_ok, Functor.map]

private theorem vec_extend_from_slice_appends
    (self : alloc.vec.Vec Std.U8) (src : Slice Std.U8)
    (hfit : self.val.length + src.val.length ≤ Std.Usize.max) :
    ∃ after : alloc.vec.Vec Std.U8,
      alloc.vec.Vec.extend_from_slice core.clone.CloneU8 self src = ok after ∧
        after.val = self.val ++ src.val := by
  have hresultFit : (self.val ++ src.val).length ≤ Std.Usize.max := by
    simp [List.length_append]
    omega
  let after := alloc.vec.Vec.from (self.val ++ src.val) hresultFit
  refine ⟨after, ?_, ?_⟩
  · simp [alloc.vec.Vec.extend_from_slice, hfit, List.clone,
      list_mapM_u8_is_identity, after]
  · simp [after, alloc.vec.Vec.from, alloc.vec.Vec.val]

theorem vec_remaining_mut_is_isize_max_minus_len (self : alloc.vec.Vec Std.U8) :
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.remaining_mut self =
      isizeMaxAsUsize - alloc.vec.Vec.len self := by
  simp [alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.remaining_mut,
    isizeMaxAsUsize, Aeneas.Std.lift]

theorem bytes_mut_remaining_mut_is_isize_max_minus_len
    (self : bytes_mut.BytesMut) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.remaining_mut self =
      isizeMaxAsUsize - self.len := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.remaining_mut,
    bytes_mut.BytesMut.impl.len, isizeMaxAsUsize, Aeneas.Std.lift]

theorem vec_remaining_mut_exact_amount
    (self : alloc.vec.Vec Std.U8)
    (hLen : (alloc.vec.Vec.len self).val ≤ isizeMaxAsUsize.val) :
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.remaining_mut self
      ⦃ amount => amount.val = isizeMaxAsUsize.val - (alloc.vec.Vec.len self).val ∧
          (alloc.vec.Vec.len self).val ≤ isizeMaxAsUsize.val ⦄ := by
  rw [vec_remaining_mut_is_isize_max_minus_len]
  exact UScalar.sub_spec hLen

theorem bytes_mut_remaining_mut_exact_amount
    (self : bytes_mut.BytesMut)
    (hLen : self.len.val ≤ isizeMaxAsUsize.val) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.remaining_mut self
      ⦃ amount => amount.val = isizeMaxAsUsize.val - self.len.val ∧
          self.len.val ≤ isizeMaxAsUsize.val ⦄ := by
  rw [bytes_mut_remaining_mut_is_isize_max_minus_len]
  exact UScalar.sub_spec hLen

theorem bytes_mut_put_slice_appends
    (contents : bytes_mut.BytesMut → List Std.U8)
    (extend_appends : ∀ (self : bytes_mut.BytesMut) (src : Slice Std.U8)
      (after : bytes_mut.BytesMut),
      bytes_mut.BytesMut.extend_from_slice self src = ok after →
        contents after = contents self ++ src.val)
    (self : bytes_mut.BytesMut) (src : Slice Std.U8)
    (after : bytes_mut.BytesMut)
    (hextend : bytes_mut.BytesMut.extend_from_slice self src = ok after) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put_slice self src =
        ok after ∧ contents after = contents self ++ src.val := by
  constructor
  · simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put_slice, hextend]
  · exact extend_appends self src after hextend

theorem bytes_mut_put_returns_self_for_empty_source
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (self : bytes_mut.BytesMut) (src : T)
    (hempty : bufInst.has_remaining src = ok false) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put bufInst self src =
      ok self := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put, hempty]

theorem bytes_mut_put_with_capacity_delegates_to_loop
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (self reserved after : bytes_mut.BytesMut) (src : T)
    (cap remaining : Std.Usize)
    (hhas : bufInst.has_remaining src = ok true)
    (hcapacity : bytes_mut.BytesMut.capacity self = ok cap)
    (hcap_nonzero : cap ≠ 0#usize)
    (hremaining : bufInst.remaining src = ok remaining)
    (hreserve : bytes_mut.BytesMut.reserve self remaining = ok reserved)
    (hloop : bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put_loop
      bufInst reserved src = ok after) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put bufInst self src =
      ok after := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put, hhas,
    hcapacity, hcap_nonzero, hremaining, hreserve, hloop]

theorem bytes_mut_put_zero_capacity_reuses_unique_bytes
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (self after : bytes_mut.BytesMut) (src : T) (remaining : Std.Usize)
    (copied : bytes.Bytes) (srcAfter : T)
    (hhas : bufInst.has_remaining src = ok true)
    (hcapacity : bytes_mut.BytesMut.capacity self = ok 0#usize)
    (hremaining : bufInst.remaining src = ok remaining)
    (hcopy : bufInst.copy_to_bytes src remaining = ok (copied, srcAfter))
    (hdrop : core.mem.drop srcAfter = ok ())
    (hunique : bytes.Bytes.try_into_mut copied =
      ok (core.result.Result.Ok after)) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put bufInst self src =
      ok after := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put, hhas,
    hcapacity, hremaining, hcopy, hdrop, hunique]

theorem bytes_mut_put_zero_capacity_extends_when_bytes_are_shared
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (self after : bytes_mut.BytesMut) (src : T) (remaining : Std.Usize)
    (copied : bytes.Bytes) (srcAfter : T) (fallback : bytes.Bytes)
    (slice : Slice Std.U8)
    (hhas : bufInst.has_remaining src = ok true)
    (hcapacity : bytes_mut.BytesMut.capacity self = ok 0#usize)
    (hremaining : bufInst.remaining src = ok remaining)
    (hcopy : bufInst.copy_to_bytes src remaining = ok (copied, srcAfter))
    (hdrop : core.mem.drop srcAfter = ok ())
    (hshared : bytes.Bytes.try_into_mut copied =
      ok (core.result.Result.Err fallback))
    (hderef : bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref fallback =
      ok slice)
    (hextend : bytes_mut.BytesMut.extend_from_slice self slice = ok after) :
    bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put bufInst self src =
      ok after := by
  simp [bytes_mut.BytesMut.Insts.BytesBufBuf_mutBufMut.put, hhas,
    hcapacity, hremaining, hcopy, hdrop, hshared, hderef, hextend]

theorem vec_put_loop_appends_the_unread_source
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (sourceBytes : T → List Std.U8)
    (hhas : ∀ src, bufInst.has_remaining src =
      ok (decide (sourceBytes src ≠ [])))
    (hchunk : ∀ src,
      bufInst.has_remaining src = ok true →
        ∃ chunk, chunk.val ≠ [] ∧ ∃ src',
          bufInst.chunk src = ok chunk ∧
          bufInst.advance src (Slice.len chunk) = ok src' ∧
          sourceBytes src = chunk.val ++ sourceBytes src')
    (self : alloc.vec.Vec Std.U8) (src : T)
    (hfits : self.val.length + (sourceBytes src).length ≤
      isizeMaxAsUsize.val)
    :
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put_loop bufInst self src
      ⦃ after => after.val = self.val ++ sourceBytes src ⦄ := by
  let body := fun state : alloc.vec.Vec Std.U8 × T =>
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put_loop.body
      bufInst state.1 state.2
  let inv := fun state : alloc.vec.Vec Std.U8 × T =>
    state.1.val ++ sourceBytes state.2 = self.val ++ sourceBytes src
  let measure := fun state : alloc.vec.Vec Std.U8 × T =>
    (sourceBytes state.2).length
  let post := fun (after : alloc.vec.Vec Std.U8) =>
    after.val = self.val ++ sourceBytes src
  have hbody : ∀ state, inv state → body state ⦃ result =>
      Aeneas.Std.loop.spec.match_1
        (fun _ : ControlFlow (alloc.vec.Vec Std.U8 × T) (alloc.vec.Vec Std.U8) => Prop)
        result (fun after => post after)
        (fun state' => inv state' ∧ measure state' < measure state) ⦄ := by
    intro state hinv
    rcases state with ⟨current, cursor⟩
    by_cases hEmpty : sourceBytes cursor = []
    · have hHas := hhas cursor
      simp [hEmpty] at hHas
      change current.val ++ sourceBytes cursor =
        self.val ++ sourceBytes src at hinv
      simp [body, alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put_loop.body,
        hHas]
      simpa [hEmpty] using hinv
    · have hHas := hhas cursor
      simp [hEmpty] at hHas
      rcases hchunk cursor hHas with
        ⟨chunk, hchunkNonempty, next, hgetChunk, hadvance, hsplit⟩
      have hcombined : current.val.length + (sourceBytes cursor).length =
          self.val.length + (sourceBytes src).length := by
        change current.val ++ sourceBytes cursor =
          self.val ++ sourceBytes src at hinv
        have h := congrArg List.length hinv
        simpa [List.length_append] using h
      have hchunkFit : current.val.length + chunk.val.length ≤
          Std.Usize.max := by
        rw [hsplit, List.length_append] at hcombined
        have hIsizeMax := isize_max_fits_usize
        omega
      rcases vec_extend_from_slice_appends current chunk hchunkFit with
        ⟨after, hextendOk, hcontents⟩
      have hnextInv : inv (after, next) := by
        dsimp [inv]
        calc
          after.val ++ sourceBytes next =
              (current.val ++ chunk.val) ++ sourceBytes next := by
                rw [hcontents]
          _ = current.val ++ (chunk.val ++ sourceBytes next) := by
                rw [List.append_assoc]
          _ = current.val ++ sourceBytes cursor := by
                rw [← hsplit]
          _ = self.val ++ sourceBytes src := hinv
      have hmeasure : (sourceBytes next).length < (sourceBytes cursor).length := by
        rw [hsplit, List.length_append]
        have hpositive : 0 < chunk.val.length := by
          cases h : chunk.val with
          | nil => contradiction
          | cons _ _ => simp
        omega
      simp [body, alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put_loop.body,
        hHas, hgetChunk, hextendOk, hadvance, Aeneas.Std.loop.spec.match_1]
      exact ⟨hnextInv, by simpa [measure] using hmeasure⟩
  have hloop : Aeneas.Std.loop body (self, src) ⦃ post ⦄ :=
    Aeneas.Std.loop.spec_decr_nat
    measure
    inv post body (self, src) hbody (by simp [inv])
  simpa [body, alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put_loop] using hloop

theorem vec_put_appends_the_source
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (sourceBytes : T → List Std.U8)
    (hhas : ∀ source, bufInst.has_remaining source =
      ok (decide (sourceBytes source ≠ [])))
    (hchunk : ∀ source,
      bufInst.has_remaining source = ok true →
        ∃ chunk, chunk.val ≠ [] ∧ ∃ source',
          bufInst.chunk source = ok chunk ∧
          bufInst.advance source (Slice.len chunk) = ok source' ∧
          sourceBytes source = chunk.val ++ sourceBytes source')
    (self : alloc.vec.Vec Std.U8) (src : T)
    (hfits : self.val.length + (sourceBytes src).length ≤
      isizeMaxAsUsize.val)
    (reserved : alloc.vec.Vec Std.U8) (remaining : Std.Usize)
    (hremaining : bufInst.remaining src = ok remaining)
    (hreserve : alloc.vec.Vec.reserve alloc.alloc.Global self remaining = ok reserved)
    (hreserveContents : reserved.val = self.val) :
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put bufInst self src
      ⦃ result => result.val = self.val ++ sourceBytes src ⦄ := by
  have hput := vec_put_loop_appends_the_unread_source bufInst sourceBytes
    hhas hchunk reserved src
    (by simpa [hreserveContents] using hfits)
  simp [alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put,
    hremaining, hreserve]
  exact WP.spec_mono hput (by
    intro result hresult
    simpa [hreserveContents] using hresult)

/-- A small concrete finite-slice source for exercising the actual generic
    `Vec::put` loop. Its chunks are bounded prefixes of the remaining slice;
    positive `width` guarantees progress. The other Buf operations are outside
    this source model and fail if called. -/
private structure FiniteChunkedSlice where
  rest : Slice Std.U8
  width : Nat
  width_positive : 0 < width

private def finiteChunk (state : FiniteChunkedSlice) : Slice Std.U8 :=
  Slice.from (state.rest.val.take (min state.width state.rest.val.length)) (by
    have h := state.rest.property
    simp)

private def finiteChunkedSliceBuf : buf.buf_impl.Buf FiniteChunkedSlice := {
  remaining := fun state => ok (Slice.len state.rest)
  chunk := fun state => ok (finiteChunk state)
  chunks_vectored := fun _ _ => Result.fail .panic
  advance := fun state cnt => ok ⟨Slice.drop state.rest cnt, state.width, state.width_positive⟩
  has_remaining := fun state => ok (decide (state.rest.val ≠ []))
  copy_to_slice := fun _ _ => Result.fail .panic
  get_u8 := fun _ => Result.fail .panic
  get_i8 := fun _ => Result.fail .panic
  get_u16 := fun _ => Result.fail .panic
  get_u16_le := fun _ => Result.fail .panic
  get_u16_ne := fun _ => Result.fail .panic
  get_i16 := fun _ => Result.fail .panic
  get_i16_le := fun _ => Result.fail .panic
  get_i16_ne := fun _ => Result.fail .panic
  get_u32 := fun _ => Result.fail .panic
  get_u32_le := fun _ => Result.fail .panic
  get_u32_ne := fun _ => Result.fail .panic
  get_i32 := fun _ => Result.fail .panic
  get_i32_le := fun _ => Result.fail .panic
  get_i32_ne := fun _ => Result.fail .panic
  get_u64 := fun _ => Result.fail .panic
  get_u64_le := fun _ => Result.fail .panic
  get_u64_ne := fun _ => Result.fail .panic
  get_i64 := fun _ => Result.fail .panic
  get_i64_le := fun _ => Result.fail .panic
  get_i64_ne := fun _ => Result.fail .panic
  get_u128 := fun _ => Result.fail .panic
  get_u128_le := fun _ => Result.fail .panic
  get_u128_ne := fun _ => Result.fail .panic
  get_i128 := fun _ => Result.fail .panic
  get_i128_le := fun _ => Result.fail .panic
  get_i128_ne := fun _ => Result.fail .panic
  get_uint := fun _ _ => Result.fail .panic
  get_uint_le := fun _ _ => Result.fail .panic
  get_uint_ne := fun _ _ => Result.fail .panic
  get_int := fun _ _ => Result.fail .panic
  get_int_le := fun _ _ => Result.fail .panic
  get_int_ne := fun _ _ => Result.fail .panic
  get_f32 := fun _ => Result.fail .panic
  get_f32_le := fun _ => Result.fail .panic
  get_f32_ne := fun _ => Result.fail .panic
  get_f64 := fun _ => Result.fail .panic
  get_f64_le := fun _ => Result.fail .panic
  get_f64_ne := fun _ => Result.fail .panic
  try_copy_to_slice := fun _ _ => Result.fail .panic
  try_get_u8 := fun _ => Result.fail .panic
  try_get_i8 := fun _ => Result.fail .panic
  try_get_u16 := fun _ => Result.fail .panic
  try_get_u16_le := fun _ => Result.fail .panic
  try_get_u16_ne := fun _ => Result.fail .panic
  try_get_i16 := fun _ => Result.fail .panic
  try_get_i16_le := fun _ => Result.fail .panic
  try_get_i16_ne := fun _ => Result.fail .panic
  try_get_u32 := fun _ => Result.fail .panic
  try_get_u32_le := fun _ => Result.fail .panic
  try_get_u32_ne := fun _ => Result.fail .panic
  try_get_i32 := fun _ => Result.fail .panic
  try_get_i32_le := fun _ => Result.fail .panic
  try_get_i32_ne := fun _ => Result.fail .panic
  try_get_u64 := fun _ => Result.fail .panic
  try_get_u64_le := fun _ => Result.fail .panic
  try_get_u64_ne := fun _ => Result.fail .panic
  try_get_i64 := fun _ => Result.fail .panic
  try_get_i64_le := fun _ => Result.fail .panic
  try_get_i64_ne := fun _ => Result.fail .panic
  try_get_u128 := fun _ => Result.fail .panic
  try_get_u128_le := fun _ => Result.fail .panic
  try_get_u128_ne := fun _ => Result.fail .panic
  try_get_i128 := fun _ => Result.fail .panic
  try_get_i128_le := fun _ => Result.fail .panic
  try_get_i128_ne := fun _ => Result.fail .panic
  try_get_uint := fun _ _ => Result.fail .panic
  try_get_uint_le := fun _ _ => Result.fail .panic
  try_get_uint_ne := fun _ _ => Result.fail .panic
  try_get_int := fun _ _ => Result.fail .panic
  try_get_int_le := fun _ _ => Result.fail .panic
  try_get_int_ne := fun _ _ => Result.fail .panic
  try_get_f32 := fun _ => Result.fail .panic
  try_get_f32_le := fun _ => Result.fail .panic
  try_get_f32_ne := fun _ => Result.fail .panic
  try_get_f64 := fun _ => Result.fail .panic
  try_get_f64_le := fun _ => Result.fail .panic
  try_get_f64_ne := fun _ => Result.fail .panic
  copy_to_bytes := fun _ _ => Result.fail .panic
  take := fun _ _ => Result.fail .panic
  chain := fun {U : Type _} _ _ => Result.fail .panic
  reader := fun _ => Result.fail .panic
}

private def finiteSourceBytes (state : FiniteChunkedSlice) : List Std.U8 :=
  state.rest.val

private theorem finiteSliceBuf_has_remaining (state : FiniteChunkedSlice) :
    finiteChunkedSliceBuf.has_remaining state =
      ok (decide (finiteSourceBytes state ≠ [])) := by
  rfl

private theorem finiteSliceBuf_remaining (state : FiniteChunkedSlice) :
    finiteChunkedSliceBuf.remaining state = ok (Slice.len state.rest) := by
  rfl

private theorem finiteSliceBuf_chunk_advance_split
    (state : FiniteChunkedSlice)
    (hhas : finiteChunkedSliceBuf.has_remaining state = ok true) :
    ∃ chunk, chunk.val ≠ [] ∧ ∃ next,
      finiteChunkedSliceBuf.chunk state = ok chunk ∧
      finiteChunkedSliceBuf.advance state (Slice.len chunk) = ok next ∧
      finiteSourceBytes state = chunk.val ++ finiteSourceBytes next := by
  have hdec : decide (finiteSourceBytes state ≠ []) = true := by
    simpa [finiteSliceBuf_has_remaining] using hhas
  have hnonempty : state.rest.val ≠ [] := by
    simpa [finiteSourceBytes] using of_decide_eq_true hdec
  let chunk := finiteChunk state
  let next : FiniteChunkedSlice := ⟨Slice.drop state.rest (Slice.len chunk),
    state.width, state.width_positive⟩
  have hchunkNonempty : chunk.val ≠ [] := by
    have hlen : 0 < chunk.val.length := by
      dsimp [chunk, finiteChunk]
      simp only [Slice.from_val, List.length_take]
      have hrest : 0 < state.rest.val.length := by
        cases h : state.rest.val with
        | nil => simp_all
        | cons _ _ => simp
      have hwidth := state.width_positive
      omega
    exact List.ne_nil_of_length_pos hlen
  refine ⟨chunk, hchunkNonempty, next, ?_, ?_, ?_⟩
  · simp [chunk, finiteChunkedSliceBuf]
  · simp [next, chunk, finiteChunkedSliceBuf]
  · dsimp [finiteSourceBytes, next, chunk, finiteChunk]
    simp only [Slice.from_val, Slice.getElem!_val_drop, Slice.len_val,
      Slice.length]
    rw [List.length_take]
    have htakeDrop : state.rest.val.take
        (min state.width state.rest.val.length) ++
        state.rest.val.drop (min state.width state.rest.val.length) =
          state.rest.val :=
      List.take_append_drop (min state.width state.rest.val.length)
        state.rest.val
    simpa [Nat.min_eq_left (Nat.min_le_right state.width state.rest.val.length)]
      using htakeDrop.symm

/-- For any finite input slice split into positive-width chunks, the actual
    translated Vec BufMut::put body appends every input byte in order. The
    source callbacks are fully supplied by `finiteChunkedSliceBuf` and proved
    above; only reserve success and preservation of the Vec data are premises. -/
theorem vec_put_appends_finite_slice_chunks
    (self : alloc.vec.Vec Std.U8) (src : Slice Std.U8)
    (width : Nat) (hwidth : 0 < width)
    (hfits : self.val.length + src.val.length ≤ isizeMaxAsUsize.val)
    (reserved : alloc.vec.Vec Std.U8)
    (hreserve : alloc.vec.Vec.reserve alloc.alloc.Global self (Slice.len src) =
      ok reserved)
    (hreserveContents : reserved.val = self.val) :
    alloc.vec.VecU8.Insts.BytesBufBuf_mutBufMut.put finiteChunkedSliceBuf
      self ⟨src, width, hwidth⟩
      ⦃ result => result.val = self.val ++ src.val ⦄ := by
  have hhas : ∀ state, finiteChunkedSliceBuf.has_remaining state =
      ok (decide (finiteSourceBytes state ≠ [])) :=
    finiteSliceBuf_has_remaining
  have hchunk : ∀ state,
      finiteChunkedSliceBuf.has_remaining state = ok true →
        ∃ chunk, chunk.val ≠ [] ∧ ∃ next,
          finiteChunkedSliceBuf.chunk state = ok chunk ∧
          finiteChunkedSliceBuf.advance state (Slice.len chunk) = ok next ∧
          finiteSourceBytes state = chunk.val ++ finiteSourceBytes next :=
    finiteSliceBuf_chunk_advance_split
  have hremaining := finiteSliceBuf_remaining ⟨src, width, hwidth⟩
  have hput := vec_put_appends_the_source finiteChunkedSliceBuf
    finiteSourceBytes hhas hchunk self ⟨src, width, hwidth⟩ hfits reserved
    (Slice.len src) hremaining hreserve hreserveContents
  simpa [finiteSourceBytes] using hput

end BytesProofs.Extensions.ConcreteBuffers

#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_put_returns_self_for_empty_source
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_put_slice_appends
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_put_with_capacity_delegates_to_loop
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_put_zero_capacity_extends_when_bytes_are_shared
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_put_zero_capacity_reuses_unique_bytes
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_remaining_mut_exact_amount
#print axioms BytesProofs.Extensions.ConcreteBuffers.bytes_mut_remaining_mut_is_isize_max_minus_len
#print axioms BytesProofs.Extensions.ConcreteBuffers.vec_put_appends_finite_slice_chunks
#print axioms BytesProofs.Extensions.ConcreteBuffers.vec_put_appends_the_source
#print axioms BytesProofs.Extensions.ConcreteBuffers.vec_remaining_mut_exact_amount
#print axioms BytesProofs.Extensions.ConcreteBuffers.vec_remaining_mut_is_isize_max_minus_len
