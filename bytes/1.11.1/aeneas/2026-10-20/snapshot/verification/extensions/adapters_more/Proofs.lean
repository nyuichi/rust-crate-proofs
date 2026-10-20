import extensions.adapters_more.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Extensions.AdaptersMore

/-! These specifications unfold the extracted 1.11.1 bodies. Operations supplied
    by inner `Buf`/`BufMut` implementations remain explicit callback hypotheses. -/

theorem limit_new_fields {T : Type _} (inner : T) (limit : Std.Usize) :
    buf.limit.new inner limit = ok ({ inner := inner, limit := limit } : buf.limit.Limit T) := by
  rfl

theorem limit_into_inner_returns_field {T : Type _} (self : buf.limit.Limit T) :
    buf.limit.Limit.into_inner self = ok self.inner := by
  rfl

theorem limit_get_ref_returns_field {T : Type _} (self : buf.limit.Limit T) :
    buf.limit.Limit.get_ref self = ok self.inner := by
  rfl

theorem limit_get_mut_focuses_inner {T : Type _} (self : buf.limit.Limit T) (inner' : T) :
    ∃ back : T → buf.limit.Limit T,
      buf.limit.Limit.get_mut self = ok (self.inner, back) ∧
      back inner' = { inner := inner', limit := self.limit } := by
  refine ⟨fun inner'' => { self with inner := inner'' }, ?_, ?_⟩ <;> rfl

theorem limit_limit_returns_field {T : Type _} (self : buf.limit.Limit T) :
    buf.limit.Limit.impl.limit self = ok self.limit := by
  rfl

theorem limit_set_limit_changes_only_limit {T : Type _}
    (self : buf.limit.Limit T) (limit : Std.Usize) :
    buf.limit.Limit.set_limit self limit =
      ok ({ inner := self.inner, limit := limit } : buf.limit.Limit T) := by
  rfl

theorem limit_remaining_mut_uses_min {T : Type _}
    (bufInst : buf.buf_mut.BufMut T) (self : buf.limit.Limit T)
    (innerRemaining : Std.Usize)
    (hinner : bufInst.remaining_mut self.inner = ok innerRemaining) :
    buf.limit.Limit.Insts.BytesBufBuf_mutBufMut.remaining_mut bufInst self =
      ok (core.cmp.impls.OrdUsize.min innerRemaining self.limit) := by
  simp [buf.limit.Limit.Insts.BytesBufBuf_mutBufMut.remaining_mut, hinner,
    core.cmp.min]

theorem limit_remaining_mut_is_bounded {T : Type _}
    (bufInst : buf.buf_mut.BufMut T) (self : buf.limit.Limit T)
    (innerRemaining : Std.Usize)
    (hinner : bufInst.remaining_mut self.inner = ok innerRemaining) :
    ∃ remaining, buf.limit.Limit.Insts.BytesBufBuf_mutBufMut.remaining_mut bufInst self =
      ok remaining ∧ remaining ≤ innerRemaining ∧ remaining ≤ self.limit := by
  let remaining := core.cmp.impls.OrdUsize.min innerRemaining self.limit
  have hminValue : remaining.val = Nat.min innerRemaining.val self.limit.val := by
    simp [remaining]
  refine ⟨remaining, ?_, ?_, ?_⟩
  · exact limit_remaining_mut_uses_min bufInst self innerRemaining hinner
  · have hval : remaining.val ≤ innerRemaining.val := by
      rw [hminValue]
      exact Nat.min_le_left _ _
    scalar_tac
  · have hval : remaining.val ≤ self.limit.val := by
      rw [hminValue]
      exact Nat.min_le_right _ _
    scalar_tac

theorem limit_chunk_mut_composes_inner_update {T : Type _}
    (bufInst : buf.buf_mut.BufMut T) (self : buf.limit.Limit T)
    (innerChunk : buf.uninit_slice.UninitSlice)
    (innerBack : buf.uninit_slice.UninitSlice → T)
    (innerLen : Std.Usize) (indexChunk : buf.uninit_slice.UninitSlice)
    (indexBack : buf.uninit_slice.UninitSlice → buf.uninit_slice.UninitSlice)
    (hchunk : bufInst.chunk_mut self.inner = ok (innerChunk, innerBack))
    (hlen : buf.uninit_slice.UninitSlice.len innerChunk = ok innerLen)
    (hindex : buf.uninit_slice.UninitSlice.Insts.CoreOpsIndexIndexMutRangeToUsizeUninitSlice.index_mut
      innerChunk { «end» := core.cmp.impls.OrdUsize.min innerLen self.limit } =
      ok (indexChunk, indexBack)) :
    buf.limit.Limit.Insts.BytesBufBuf_mutBufMut.chunk_mut bufInst self =
      ok (indexChunk, fun replacement =>
        { self with inner := innerBack (indexBack replacement) }) := by
  simp [buf.limit.Limit.Insts.BytesBufBuf_mutBufMut.chunk_mut, hchunk, hlen,
    hindex, core.cmp.min]

theorem take_get_mut_focuses_inner {T : Type _} (self : buf.take.Take T) (inner' : T) :
    ∃ back : T → buf.take.Take T,
      buf.take.Take.get_mut self = ok (self.inner, back) ∧
      back inner' = { inner := inner', limit := self.limit } := by
  refine ⟨fun inner'' => { self with inner := inner'' }, ?_, ?_⟩ <;> rfl

theorem chain_chunk_uses_first_when_available {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (chunkA : Slice Std.U8)
    (hhas : bufInstA.has_remaining self.a = ok true)
    (hchunk : bufInstA.chunk self.a = ok chunkA) :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunk bufInstA bufInstB self = ok chunkA := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunk, hhas, hchunk]

theorem chain_chunk_uses_second_when_first_empty {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (chunkB : Slice Std.U8)
    (hhas : bufInstA.has_remaining self.a = ok false)
    (hchunk : bufInstB.chunk self.b = ok chunkB) :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunk bufInstA bufInstB self = ok chunkB := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunk, hhas, hchunk]

theorem chain_chunks_vectored_delegates_in_order {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (dst : Slice core.io.io_slice.IoSlice)
    (n : Std.Usize) (dstA : Slice core.io.io_slice.IoSlice)
    (s : Slice core.io.io_slice.IoSlice)
    (indexBack : Slice core.io.io_slice.IoSlice → Slice core.io.io_slice.IoSlice)
    (m : Std.Usize) (dstB : Slice core.io.io_slice.IoSlice) (total : Std.Usize)
    (hfirst : bufInstA.chunks_vectored self.a dst = ok (n, dstA))
    (hindex : core.slice.index.SliceIndexRangeFromUsizeSlice.index_mut
      { start := n } dstA = ok (s, indexBack))
    (hsecond : bufInstB.chunks_vectored self.b s = ok (m, dstB))
    (hadd : n + m = ok total) :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunks_vectored bufInstA bufInstB self dst =
      ok (total, indexBack dstB) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.chunks_vectored,
    hfirst, hindex, hsecond, hadd]

theorem chain_remaining_mut_is_saturating_sum {T U : Type _}
    (bufInstA : buf.buf_mut.BufMut T) (bufInstB : buf.buf_mut.BufMut U)
    (self : buf.chain.Chain T U) (aRemaining bRemaining : Std.Usize)
    (ha : bufInstA.remaining_mut self.a = ok aRemaining)
    (hb : bufInstB.remaining_mut self.b = ok bRemaining) :
    buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.remaining_mut bufInstA bufInstB self =
      ok (core.num.Usize.saturating_add aRemaining bRemaining) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.remaining_mut, ha, hb]

theorem chain_chunk_mut_uses_first_when_available {T U : Type _}
    (bufInstA : buf.buf_mut.BufMut T) (bufInstB : buf.buf_mut.BufMut U)
    (self : buf.chain.Chain T U) (chunkA : buf.uninit_slice.UninitSlice)
    (backA : buf.uninit_slice.UninitSlice → T)
    (hhas : bufInstA.has_remaining_mut self.a = ok true)
    (hchunk : bufInstA.chunk_mut self.a = ok (chunkA, backA)) :
    buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.chunk_mut bufInstA bufInstB self =
      ok (chunkA, fun replacement => { self with a := backA replacement }) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.chunk_mut, hhas, hchunk]

theorem chain_chunk_mut_uses_second_when_first_full {T U : Type _}
    (bufInstA : buf.buf_mut.BufMut T) (bufInstB : buf.buf_mut.BufMut U)
    (self : buf.chain.Chain T U) (chunkB : buf.uninit_slice.UninitSlice)
    (backB : buf.uninit_slice.UninitSlice → U)
    (hhas : bufInstA.has_remaining_mut self.a = ok false)
    (hchunk : bufInstB.chunk_mut self.b = ok (chunkB, backB)) :
    buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.chunk_mut bufInstA bufInstB self =
      ok (chunkB, fun replacement => { self with b := backB replacement }) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_mutBufMut.chunk_mut, hhas, hchunk]

theorem chain_into_iter_wraps_self {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) :
    buf.chain.Chain.Insts.CoreIterTraitsCollectIntoIteratorU8IntoIterChain.into_iter
      bufInstA bufInstB self = ok ({ inner := self } : buf.iter.IntoIter (buf.chain.Chain T U)) := by
  simp [buf.chain.Chain.Insts.CoreIterTraitsCollectIntoIteratorU8IntoIterChain.into_iter,
    buf.iter.IntoIter.new]

theorem into_iter_new_field {T : Type _} (inner : T) :
    buf.iter.IntoIter.new inner = ok ({ inner := inner } : buf.iter.IntoIter T) := by
  rfl

theorem into_iter_into_inner_returns_field {T : Type _} (self : buf.iter.IntoIter T) :
    buf.iter.IntoIter.into_inner self = ok self.inner := by
  rfl

theorem into_iter_get_ref_returns_field {T : Type _} (self : buf.iter.IntoIter T) :
    buf.iter.IntoIter.get_ref self = ok self.inner := by
  rfl

theorem into_iter_get_mut_focuses_inner {T : Type _} (self : buf.iter.IntoIter T) (inner' : T) :
    ∃ back : T → buf.iter.IntoIter T,
      buf.iter.IntoIter.get_mut self = ok (self.inner, back) ∧ back inner' = { inner := inner' } := by
  refine ⟨fun inner'' => { inner := inner'' }, ?_, ?_⟩ <;> rfl

theorem into_iter_size_hint_is_exact {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.iter.IntoIter T)
    (remaining : Std.Usize)
    (hremaining : bufInst.remaining self.inner = ok remaining) :
    buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.size_hint bufInst self =
      ok (remaining, some remaining) := by
  simp [buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.size_hint, hremaining]

theorem into_iter_next_returns_none_when_empty {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.iter.IntoIter T)
    (hempty : bufInst.has_remaining self.inner = ok false) :
    buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.next bufInst self =
      ok (none, self) := by
  simp [buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.next, hempty]

theorem into_iter_next_consumes_first_chunk_byte {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.iter.IntoIter T)
    (chunk : Slice Std.U8) (byte : Std.U8) (inner' : T)
    (hhas : bufInst.has_remaining self.inner = ok true)
    (hchunk : bufInst.chunk self.inner = ok chunk)
    (hindex : Slice.index_usize chunk 0#usize = ok byte)
    (hadvance : bufInst.advance self.inner 1#usize = ok inner') :
    buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.next bufInst self =
      ok (some byte, { inner := inner' }) := by
  simp [buf.iter.IntoIter.Insts.CoreIterTraitsIteratorIteratorU8.next,
    hhas, hchunk, hindex, hadvance]

theorem vecdeque_remaining_delegates_to_len
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (remaining : Std.Usize)
    (hlen : alloc.collections.vec_deque.VecDeque.len self = ok remaining) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.remaining self =
      ok remaining := by
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.remaining, hlen]

theorem vecdeque_chunk_uses_second_when_first_empty
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (s1 s2 : Slice Std.U8)
    (hslices : alloc.collections.vec_deque.VecDeque.as_slices self = ok (s1, s2))
    (hempty : core.slice.Slice.is_empty s1 = ok true) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunk self = ok s2 := by
  have hlen : s1.length = 0 := by simpa [core.slice.Slice.is_empty] using hempty
  have hnil : s1.val = [] := by
    simpa [Slice.length] using hlen
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunk,
    hslices, core.slice.Slice.is_empty, hnil]

theorem vecdeque_chunk_uses_first_when_nonempty
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (s1 s2 : Slice Std.U8)
    (hslices : alloc.collections.vec_deque.VecDeque.as_slices self = ok (s1, s2))
    (hnonempty : core.slice.Slice.is_empty s1 = ok false) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunk self = ok s1 := by
  have hlen : s1.length ≠ 0 := by simpa [core.slice.Slice.is_empty] using hnonempty
  have hnotnil : ¬ s1.val = [] := by
    intro h
    apply hlen
    simp [Slice.length, h]
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunk,
    hslices, core.slice.Slice.is_empty, hnotnil]

theorem vecdeque_chunks_vectored_returns_zero_when_empty
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (dst : Slice core.io.io_slice.IoSlice)
    (hempty : alloc.collections.vec_deque.VecDeque.is_empty self = ok true) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored self dst =
      ok (0#usize, dst) := by
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored, hempty]

theorem vecdeque_chunks_vectored_returns_zero_for_empty_dst
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (dst : Slice core.io.io_slice.IoSlice)
    (hnotempty : alloc.collections.vec_deque.VecDeque.is_empty self = ok false)
    (hdstempty : core.slice.Slice.is_empty dst = ok true) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored self dst =
      ok (0#usize, dst) := by
  have hdstLen : dst.length = 0 := by simpa [core.slice.Slice.is_empty] using hdstempty
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored,
    hnotempty, core.slice.Slice.is_empty, hdstLen]

theorem vecdeque_chunks_vectored_writes_first_slice_when_second_empty
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (dst : Slice core.io.io_slice.IoSlice)
    (s1 s2 : Slice Std.U8) (firstIo : core.io.io_slice.IoSlice)
    (old : core.io.io_slice.IoSlice)
    (indexBack : core.io.io_slice.IoSlice → Slice core.io.io_slice.IoSlice)
    (hnotempty : alloc.collections.vec_deque.VecDeque.is_empty self = ok false)
    (hdstnonempty : core.slice.Slice.is_empty dst = ok false)
    (hslices : alloc.collections.vec_deque.VecDeque.as_slices self = ok (s1, s2))
    (hnew1 : core.io.io_slice.IoSlice.new s1 = ok firstIo)
    (hindex : Slice.index_mut_usize dst 0#usize = ok (old, indexBack))
    (hsecondempty : core.slice.Slice.is_empty s2 = ok true) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored self dst =
      ok (1#usize, indexBack firstIo) := by
  have hdstLen : dst.length ≠ 0 := by
    simp [core.slice.Slice.is_empty] at hdstnonempty
    simpa [core.slice.Slice.is_empty] using hdstnonempty
  have hsecondLen : s2.length = 0 := by
    simpa [core.slice.Slice.is_empty] using hsecondempty
  have hsecondNil : s2.val = [] := by
    simpa [Slice.length] using hsecondLen
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored,
    hnotempty, core.slice.Slice.is_empty, hdstLen, hslices, hnew1, hindex, hsecondNil]

theorem vecdeque_chunks_vectored_writes_two_slices_when_space_remains
    (self : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (dst : Slice core.io.io_slice.IoSlice)
    (s1 s2 : Slice Std.U8) (firstIo secondIo : core.io.io_slice.IoSlice)
    (old : core.io.io_slice.IoSlice)
    (indexBack : core.io.io_slice.IoSlice → Slice core.io.io_slice.IoSlice)
    (slots : Std.Usize) (dst2 : Slice core.io.io_slice.IoSlice)
    (hnotempty : alloc.collections.vec_deque.VecDeque.is_empty self = ok false)
    (hdstnonempty : core.slice.Slice.is_empty dst = ok false)
    (hslices : alloc.collections.vec_deque.VecDeque.as_slices self = ok (s1, s2))
    (hnew1 : core.io.io_slice.IoSlice.new s1 = ok firstIo)
    (hindex : Slice.index_mut_usize dst 0#usize = ok (old, indexBack))
    (hsecondnonempty : core.slice.Slice.is_empty s2 = ok false)
    (hslots : Slice.len (indexBack firstIo) = slots)
    (hmoreSlots : slots ≠ 1#usize)
    (hnew2 : core.io.io_slice.IoSlice.new s2 = ok secondIo)
    (hupdate : Slice.update (indexBack firstIo) 1#usize secondIo = ok dst2) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored self dst =
      ok (2#usize, dst2) := by
  have hdstLen : dst.length ≠ 0 := by
    simp [core.slice.Slice.is_empty] at hdstnonempty
    simpa [core.slice.Slice.is_empty] using hdstnonempty
  have hsecondNotNil : ¬ s2.val = [] := by
    intro hempty
    have hlen : s2.length = 0 := by
      simp [Slice.length, hempty]
    have : s2.length ≠ 0 := by
      simpa [core.slice.Slice.is_empty] using hsecondnonempty
    exact this hlen
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.chunks_vectored,
    hnotempty, core.slice.Slice.is_empty, hdstLen, hslices, hnew1, hindex,
    hsecondNotNil, hslots, hmoreSlots, hnew2, hupdate]

/-- The vector deque `advance` delegates the state update to the standard
    library's `drain(..cnt)` callback. -/
theorem vecdeque_advance_returns_drain_state
    (self self' : alloc.collections.vec_deque.VecDeque Std.U8 alloc.alloc.Global)
    (cnt : Std.Usize) (drained : alloc.collections.vec_deque.drain.Drain Std.U8 alloc.alloc.Global)
    (hdrain : alloc.collections.vec_deque.VecDeque.drain
      (core.ops.range.RangeTo.Insts.CoreOpsRangeRangeBounds Std.Usize) self { «end» := cnt } =
      ok (drained, self')) :
    alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.advance self cnt =
      ok self' := by
  simp [alloc.collections.vec_deque.VecDequeU8Global.Insts.BytesBufBuf_implBuf.advance, hdrain]

#print axioms limit_new_fields
#print axioms limit_into_inner_returns_field
#print axioms limit_get_ref_returns_field
#print axioms limit_get_mut_focuses_inner
#print axioms limit_limit_returns_field
#print axioms limit_set_limit_changes_only_limit
#print axioms limit_remaining_mut_uses_min
#print axioms limit_remaining_mut_is_bounded
#print axioms limit_chunk_mut_composes_inner_update
#print axioms take_get_mut_focuses_inner
#print axioms chain_chunk_uses_first_when_available
#print axioms chain_chunk_uses_second_when_first_empty
#print axioms chain_chunks_vectored_delegates_in_order
#print axioms chain_remaining_mut_is_saturating_sum
#print axioms chain_chunk_mut_uses_first_when_available
#print axioms chain_chunk_mut_uses_second_when_first_full
#print axioms chain_into_iter_wraps_self
#print axioms into_iter_new_field
#print axioms into_iter_into_inner_returns_field
#print axioms into_iter_get_ref_returns_field
#print axioms into_iter_get_mut_focuses_inner
#print axioms into_iter_size_hint_is_exact
#print axioms into_iter_next_returns_none_when_empty
#print axioms into_iter_next_consumes_first_chunk_byte
#print axioms vecdeque_remaining_delegates_to_len
#print axioms vecdeque_chunk_uses_second_when_first_empty
#print axioms vecdeque_chunk_uses_first_when_nonempty
#print axioms vecdeque_chunks_vectored_returns_zero_when_empty
#print axioms vecdeque_chunks_vectored_returns_zero_for_empty_dst
#print axioms vecdeque_chunks_vectored_writes_first_slice_when_second_empty
#print axioms vecdeque_chunks_vectored_writes_two_slices_when_space_remains
#print axioms vecdeque_advance_returns_drain_state

end BytesProofs.Extensions.AdaptersMore
