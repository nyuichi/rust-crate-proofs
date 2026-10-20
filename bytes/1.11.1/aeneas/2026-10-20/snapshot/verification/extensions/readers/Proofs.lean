import extensions.readers.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Readers

theorem has_remaining_returns_positive_iff {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (n : Std.Usize)
    (hremaining : bufInst.remaining self = ok n) :
    buf.buf_impl.Buf.has_remaining.default bufInst self = ok true ↔
      0 < n.val := by
  simp [buf.buf_impl.Buf.has_remaining.default, hremaining,
    ]

theorem has_remaining_returns_false_iff {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (n : Std.Usize)
    (hremaining : bufInst.remaining self = ok n) :
    buf.buf_impl.Buf.has_remaining.default bufInst self = ok false ↔
      n.val = 0 := by
  simp [buf.buf_impl.Buf.has_remaining.default, hremaining,
    ]

theorem chunks_vectored_empty_preserves_destination {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T)
    (dst : Slice core.io.io_slice.IoSlice)
    (hempty : core.slice.Slice.is_empty dst = ok true) :
    buf.buf_impl.Buf.chunks_vectored.default bufInst self dst =
      ok (0#usize, dst) := by
  have hlen : dst.length = 0 := by
    simpa [core.slice.Slice.is_empty] using hempty
  simp [buf.buf_impl.Buf.chunks_vectored.default,
    core.slice.Slice.is_empty, hlen]

theorem chunks_vectored_emits_first_chunk {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T)
    (dst : Slice core.io.io_slice.IoSlice)
    (src : Slice Std.U8) (entry : core.io.io_slice.IoSlice)
    (dst' : Slice core.io.io_slice.IoSlice)
    (hnonempty : core.slice.Slice.is_empty dst = ok false)
    (hhas : bufInst.has_remaining self = ok true)
    (hchunk : bufInst.chunk self = ok src)
    (hnew : core.io.io_slice.IoSlice.new src = ok entry)
    (hupdate : Slice.update dst 0#usize entry = ok dst') :
    buf.buf_impl.Buf.chunks_vectored.default bufInst self dst =
      ok (1#usize, dst') := by
  have hlen : dst.length ≠ 0 := by
    simp [core.slice.Slice.is_empty] at hnonempty
    simpa [core.slice.Slice.is_empty] using hnonempty
  have hnotnil : ¬ dst.val = [] := by
    intro h
    apply hlen
    simp [Slice.length, h]
  simp [buf.buf_impl.Buf.chunks_vectored.default,
    core.slice.Slice.is_empty, hnotnil, hhas,
    hchunk, hnew, hupdate]

theorem take_default_constructs_capped_view {T : Type _}
    (self : T) (limit : Std.Usize) :
    buf.buf_impl.Buf.take.default self limit =
      ok ({ inner := self, limit := limit } : buf.take.Take T) := by
  rfl

theorem chain_default_constructs_ordered_pair {T U : Type _}
    (self : T) (next : U) :
    buf.buf_impl.Buf.chain.default self next =
      ok ({ a := self, b := next } : buf.chain.Chain T U) := by
  rfl

theorem reader_default_forwards_factory {T : Type _}
    (self : T) (reader : buf.reader.Reader T)
    (hnew : buf.reader.new self = ok reader) :
    buf.buf_impl.Buf.reader.default self = ok reader := by
  simp [buf.buf_impl.Buf.reader.default, hnew]

theorem try_get_u8_insufficient_preserves_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (n : Std.Usize)
    (hremaining : Mut0T.Insts.BytesBufBuf_implBuf.remaining bufInst self = ok n)
    (hshort : n < 1#usize) :
    buf.buf_impl.Buf.try_get_u8.default bufInst self =
      ok (core.result.Result.Err
        { requested := 1#usize, available := n }, self) := by
  have hnzero : n.val = 0 := by scalar_tac
  simp [buf.buf_impl.Buf.try_get_u8.default, hremaining, hnzero]

theorem try_get_u8_reads_head_and_advances {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T) (n : Std.Usize)
    (src : Slice Std.U8) (byte : Std.U8)
    (hremaining : Mut0T.Insts.BytesBufBuf_implBuf.remaining bufInst self = ok n)
    (henough : ¬ n < 1#usize)
    (hchunk : Mut0T.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok src)
    (hindex : Slice.index_usize src 0#usize = ok byte)
    (hadvance : bufInst.advance self 1#usize = ok self') :
    buf.buf_impl.Buf.try_get_u8.default bufInst self =
      ok (core.result.Result.Ok byte, self') := by
  have hnzero : n.val ≠ 0 := by
    intro hn
    apply henough
    scalar_tac
  simp [buf.buf_impl.Buf.try_get_u8.default, hremaining, hnzero,
    hchunk, hindex, hadvance]

theorem try_copy_to_slice_insufficient_preserves_inputs {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (dst : Slice Std.U8)
    (n : Std.Usize)
    (hremaining : bufInst.remaining self = ok n)
    (hshort : n.val < dst.val.length) :
    buf.buf_impl.Buf.try_copy_to_slice.default bufInst self dst =
      ok (core.result.Result.Err
        { requested := Slice.len dst, available := n }, self, dst) := by
  have hnotEnough : ¬ dst.val.length ≤ n.val := Nat.not_le_of_gt hshort
  simp [buf.buf_impl.Buf.try_copy_to_slice.default,
    Mut0T.Insts.BytesBufBuf_implBuf.remaining, hremaining, hnotEnough]

theorem try_copy_loop_empty_body_preserves_inputs {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (back : Slice Std.U8 → Slice Std.U8)
    (self : T) (dst : Slice Std.U8)
    (hempty : core.slice.Slice.is_empty dst = ok true) :
    buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst back self dst =
      ok (ControlFlow.done (self, back dst)) := by
  have hlen : dst.length = 0 := by
    simpa [core.slice.Slice.is_empty] using hempty
  simp [buf.buf_impl.Buf.try_copy_to_slice.default_loop.body,
    core.slice.Slice.is_empty, hlen]

theorem try_copy_loop_nonempty_copies_one_chunk_prefix {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (back : Slice Std.U8 → Slice Std.U8)
    (self self' : T) (dst src : Slice Std.U8) (cnt : Std.Usize)
    (dstPrefix srcPrefix dstAfter : Slice Std.U8)
    (backPrefix backSuffix : Slice Std.U8 → Slice Std.U8)
    (hnonempty : core.slice.Slice.is_empty dst = ok false)
    (hchunk : Mut0T.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok src)
    (hmin : core.cmp.impls.OrdUsize.min (Slice.len src) (Slice.len dst) = cnt)
    (hdstPrefix : core.slice.index.Slice.index_mut
      (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) dst
      { «end» := cnt } = ok (dstPrefix, backPrefix))
    (hsrcPrefix : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
      { «end» := cnt } = ok srcPrefix)
    (hcopyLength : Slice.len dstPrefix = Slice.len srcPrefix)
    (hdstSuffix : core.slice.index.Slice.index_mut
      (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8)
      (backPrefix srcPrefix) { start := cnt } = ok (dstAfter, backSuffix))
    (hadvance : bufInst.advance self cnt = ok self') :
    buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst back self dst =
      ok (ControlFlow.cont
        (fun s => back (backSuffix s), self', dstAfter)) := by
  have hlen : dst.length ≠ 0 := by
    simpa [core.slice.Slice.is_empty] using hnonempty
  have hnotnil : ¬ dst.val = [] := by
    intro h
    apply hlen
    simp [Slice.length, h]
  have hdstPrefix' : core.slice.index.SliceIndexRangeToUsizeSlice.index_mut
      { «end» := cnt } dst = ok (dstPrefix, backPrefix) := by
    simpa using hdstPrefix
  have hsrcPrefix' : core.slice.index.SliceIndexRangeToUsizeSlice.index
      { «end» := cnt } src = ok srcPrefix := by
    simpa using hsrcPrefix
  have hdstSuffix' : core.slice.index.SliceIndexRangeFromUsizeSlice.index_mut
      { start := cnt } (backPrefix srcPrefix) = ok (dstAfter, backSuffix) := by
    simpa using hdstSuffix
  simp [buf.buf_impl.Buf.try_copy_to_slice.default_loop.body,
    core.slice.Slice.is_empty, hnotnil, hchunk, hmin, hdstPrefix',
    hsrcPrefix', core.slice.Slice.copy_from_slice, hcopyLength,
    hdstSuffix', hadvance, lift]

theorem try_copy_suffix_back_appends_source_prefix
    (dst src dstPrefix srcPrefix dstAfter : Slice Std.U8)
    (cnt : Std.Usize) (backPrefix backSuffix : Slice Std.U8 → Slice Std.U8)
    (hdstPrefix : core.slice.index.Slice.index_mut
      (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) dst
      { «end» := cnt } = ok (dstPrefix, backPrefix))
    (hsrcPrefix : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
      { «end» := cnt } = ok srcPrefix)
    (hdstSuffix : core.slice.index.Slice.index_mut
      (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8)
      (backPrefix srcPrefix) { start := cnt } = ok (dstAfter, backSuffix)) :
    ∀ s : Slice Std.U8, s.length = dstAfter.length →
      (backSuffix s).val = srcPrefix.val ++ s.val ∧
      srcPrefix.val.length = cnt.val := by
  have hp := hdstPrefix
  simp only [core.slice.index.Slice.index_mut,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut] at hp
  have hs := hsrcPrefix
  simp only [core.slice.index.Slice.index,
    core.slice.index.SliceIndexRangeToUsizeSlice.index] at hs
  have hsf := hdstSuffix
  simp only [core.slice.index.Slice.index_mut,
    core.slice.index.SliceIndexRangeFromUsizeSlice.index_mut] at hsf
  have hpc : (↑cnt : Nat) ≤ dst.length := by
    by_contra h
    simp [h] at hp
  have hsc : (↑cnt : Nat) ≤ src.length := by
    by_contra h
    simp [h] at hs
  simp [hpc] at hp
  simp [hsc] at hs
  have hsrcValLen : src.val.length = src.leng := by
    simp [Slice.val, Aeneas.Data.ListN.ListN_length]
  have hsc' : cnt.val ≤ src.val.length := by omega
  have hsc'' : cnt.val ≤ src.leng := by omega
  have hsVal : srcPrefix.val = List.take cnt.val src.val := by
    have hval := congrArg Slice.val hs
    simpa [Slice.from_val] using hval.symm
  have hcntSrc : srcPrefix.val.length = cnt.val := by
    rw [hsVal]
    simp [List.length_take, Nat.min_eq_left hsc']
  have hbpVal : (backPrefix srcPrefix).val =
      srcPrefix.val ++ dst.val.drop cnt.val := by
    rw [← hp.2]
    simp [Slice.from_val, List.setSlice!, hpc, hcntSrc]
  have hdlen : dst.val.length = dst.length := by
    simp [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length]
  have hfullVal : (backPrefix srcPrefix).val.length = dst.val.length := by
    rw [hbpVal]
    simp [List.length_append, List.length_drop]
    omega
  have hfullLen : (backPrefix srcPrefix).length = dst.length := by
    simpa [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length] using hfullVal
  have hsfcond : (↑cnt : Nat) ≤ (backPrefix srcPrefix).length := by omega
  simp [hsfcond] at hsf
  have hdrop := congrArg (fun sl : Slice Std.U8 => sl.val) hsf.1
  have hafterVal : dstAfter.val = dst.val.drop cnt.val := by
    simp [Slice.drop, hbpVal, hcntSrc] at hdrop
    simpa using hdrop.symm
  intro s hslen
  have hbackSuffixVal : (backSuffix s).val =
      (backPrefix srcPrefix).val.setSlice! cnt.val s.val := by
    have hfun := congrArg (fun f : Slice Std.U8 → Slice Std.U8 => (f s).val) hsf.2
    simpa [Slice.from_val] using hfun.symm
  have hsvalLen : s.val.length = dstAfter.val.length := by
    simpa [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length] using hslen
  have htailLen : s.val.length = (dst.val.drop cnt.val).length := by
    rw [hsvalLen, hafterVal]
  rw [hbackSuffixVal, hbpVal]
  constructor
  · simp [List.setSlice!, hcntSrc, htailLen, hpc]
  · exact hcntSrc

theorem try_copy_source_prefix_is_model_prefix (src srcPrefix : Slice Std.U8)
    (cnt : Std.Usize) (view : List Std.U8)
    (hsrc : src.val = view.take src.length)
    (hsrcPrefix : core.slice.index.Slice.index
      (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
      { «end» := cnt } = ok srcPrefix) :
    srcPrefix.val = view.take cnt.val := by
  have hs := hsrcPrefix
  simp only [core.slice.index.Slice.index,
    core.slice.index.SliceIndexRangeToUsizeSlice.index] at hs
  have hsc : cnt.val ≤ src.length := by
    by_contra h
    simp [h] at hs
  simp [hsc] at hs
  have hprefix : srcPrefix.val = src.val.take cnt.val := by
    have hval := congrArg Slice.val hs
    simpa [Slice.from_val] using hval.symm
  calc
    srcPrefix.val = src.val.take cnt.val := hprefix
    _ = (view.take src.length).take cnt.val := by rw [hsrc]
    _ = view.take cnt.val := by
      rw [List.take_take, Nat.min_eq_left hsc]

private theorem list_take_drop_take_prefix {α : Type _}
    (values : List α) (prefixLength count : Nat) :
    values.take prefixLength ++ (values.drop prefixLength).take count =
      values.take (prefixLength + count) := by
  induction prefixLength generalizing values with
  | zero => simp
  | succ prefixLength ih =>
      cases values with
      | nil => simp
      | cons head tail => simp [List.take, List.drop, ih]

theorem try_copy_loop_copies_model_prefix_under_coherent_callbacks
    {T : Type _} (bufInst : buf.buf_impl.Buf T)
    (view : T → List Std.U8) (initialView : List Std.U8)
    (initialDst : Slice Std.U8)
    (hcallbacks : ∀ (self : T) (dst : Slice Std.U8),
      core.slice.Slice.is_empty dst = ok false →
      dst.length ≤ (view self).length →
      ∃ (src : Slice Std.U8) (cnt : Std.Usize)
        (dstPrefix srcPrefix dstAfter : Slice Std.U8)
        (self' : T) (backPrefix backSuffix : Slice Std.U8 → Slice Std.U8),
        Mut0T.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok src ∧
        src.val = (view self).take src.length ∧
        core.cmp.impls.OrdUsize.min (Slice.len src) (Slice.len dst) = cnt ∧
        core.slice.index.Slice.index_mut
          (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) dst
          { «end» := cnt } = ok (dstPrefix, backPrefix) ∧
        core.slice.index.Slice.index
          (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
          { «end» := cnt } = ok srcPrefix ∧
        Slice.len dstPrefix = Slice.len srcPrefix ∧
        core.slice.index.Slice.index_mut
          (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8)
          (backPrefix srcPrefix) { start := cnt } = ok (dstAfter, backSuffix) ∧
        bufInst.advance self cnt = ok self' ∧
        0 < cnt.val ∧
        dstAfter.length + cnt.val = dst.length ∧
        view self' = (view self).drop cnt.val) :
    ∀ (back : Slice Std.U8 → Slice Std.U8) (self : T)
      (dst : Slice Std.U8) (consumed : Nat),
      dst.length + consumed = initialDst.length →
      dst.length ≤ (view self).length →
      view self = initialView.drop consumed →
      (∀ s, s.length = dst.length →
        (back s).val = initialView.take consumed ++ s.val) →
      ∃ (finalSelf : T) (output : Slice Std.U8),
        buf.buf_impl.Buf.try_copy_to_slice.default_loop bufInst back self dst =
          ok (finalSelf, output) ∧
        output.val = initialView.take (consumed + dst.length) ∧
        view finalSelf = initialView.drop (consumed + dst.length) := by
  intro back self dst consumed
  induction hd : dst.length using Nat.strong_induction_on
    generalizing back self dst consumed with
  | h n ih =>
      intro hstateLength hstateEnough hstateView hback
      by_cases hzero : dst.length = 0
      · have hempty : core.slice.Slice.is_empty dst = ok true := by
          simp [core.slice.Slice.is_empty, hzero]
        have hbody := try_copy_loop_empty_body_preserves_inputs
          bufInst back self dst hempty
        have hdstValLength : dst.val.length = 0 := by
          simpa [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length]
            using hzero
        have hdstVal : dst.val = [] := by
          cases h : dst.val with
          | nil => rfl
          | cons head tail => simp [h] at hdstValLength
        have hconsumed : consumed = initialDst.length := by omega
        have hnzero : n = 0 := by omega
        have hbackDst := hback dst hd
        refine ⟨self, back dst, ?_, ?_, ?_⟩
        · change loop
            (fun state =>
              buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst
                state.1 state.2.1 state.2.2)
            (back, self, dst) = ok (self, back dst)
          rw [loop.eq_1, hbody]
          simp
        · rw [hbackDst, hdstVal, hconsumed, hnzero]
          simp
        · rw [hstateView, hnzero]
          simp
      · have hnonempty : core.slice.Slice.is_empty dst = ok false := by
          have hlen : dst.length ≠ 0 := hzero
          simp [core.slice.Slice.is_empty, hlen]
        obtain ⟨src, cnt, dstPrefix, srcPrefix, dstAfter, self',
          backPrefix, backSuffix, hchunk, hsrcView, hmin, hdstPrefix,
          hsrcPrefix, hcopyLength, hdstSuffix, hadvance, hpositive,
          hlength, hviewStep⟩ :=
            hcallbacks self dst hnonempty (by simpa [hd] using hstateEnough)
        have hbody := try_copy_loop_nonempty_copies_one_chunk_prefix
          bufInst back self self' dst src cnt dstPrefix srcPrefix dstAfter
          backPrefix backSuffix hnonempty hchunk hmin hdstPrefix hsrcPrefix
          hcopyLength hdstSuffix hadvance
        have hsourcePrefix := try_copy_source_prefix_is_model_prefix
          src srcPrefix cnt (view self) hsrcView hsrcPrefix
        have hsrcLengthBound : src.length ≤ (view self).length := by
          have hsrcValLength : src.val.length = src.length := by
            simp [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length]
          rw [hsrcView, List.length_take] at hsrcValLength
          omega
        have hcntMin : cnt.val = min src.length dst.length := by
          have hm := congrArg (fun x : Std.Usize => x.val) hmin
          simpa [Slice.len, Slice.length] using hm.symm
        have hcntView : cnt.val ≤ (view self).length := by
          rw [hcntMin]
          omega
        have hnextEnough : dstAfter.length ≤ (view self').length := by
          rw [hviewStep, List.length_drop]
          omega
        have ⟨_hsuffixContent, hprefixCount⟩ :=
          try_copy_suffix_back_appends_source_prefix dst src dstPrefix
            srcPrefix dstAfter cnt backPrefix backSuffix hdstPrefix hsrcPrefix
            hdstSuffix dstAfter rfl
        have hdecrease : dstAfter.length < n := by omega
        have hnextLength :
            dstAfter.length + (consumed + cnt.val) = initialDst.length := by
          omega
        have hnextView : view self' = initialView.drop (consumed + cnt.val) := by
          calc
            view self' = (view self).drop cnt.val := hviewStep
            _ = (initialView.drop consumed).drop cnt.val := by rw [hstateView]
            _ = initialView.drop (consumed + cnt.val) := by rw [List.drop_drop]
        let nextBack : Slice Std.U8 → Slice Std.U8 :=
          fun s => back (backSuffix s)
        have hnextBack : ∀ s, s.length = dstAfter.length →
            (nextBack s).val = initialView.take (consumed + cnt.val) ++ s.val := by
          intro s hslen
          obtain ⟨hsuffix, hcount⟩ :=
            try_copy_suffix_back_appends_source_prefix dst src dstPrefix
              srcPrefix dstAfter cnt backPrefix backSuffix hdstPrefix hsrcPrefix
              hdstSuffix s hslen
          have hsvalLen : s.val.length = s.length := by
            simp [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length]
          have hbackSuffixValLen :
              (backSuffix s).val.length = cnt.val + s.val.length := by
            rw [hsuffix]
            simp [List.length_append, hcount]
          have hbackSuffixLen : (backSuffix s).length = dst.length := by
            have hlenVal : (backSuffix s).length = (backSuffix s).val.length := by
              simp [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length]
            rw [hlenVal, hbackSuffixValLen, hsvalLen, hslen]
            omega
          have hprior := hback (backSuffix s) (by simpa [hd] using hbackSuffixLen)
          change (back (backSuffix s)).val = _
          rw [hprior, hsuffix, hsourcePrefix]
          rw [hstateView]
          calc
            initialView.take consumed ++
                ((initialView.drop consumed).take cnt.val ++ s.val) =
                (initialView.take consumed ++
                  (initialView.drop consumed).take cnt.val) ++ s.val := by
                    rw [List.append_assoc]
            _ = initialView.take (consumed + cnt.val) ++ s.val := by
              rw [list_take_drop_take_prefix]
        have hnext :
            ∃ (finalSelf : T) (output : Slice Std.U8),
              buf.buf_impl.Buf.try_copy_to_slice.default_loop bufInst nextBack
                self' dstAfter = ok (finalSelf, output) ∧
              output.val = initialView.take
                (consumed + cnt.val + dstAfter.length) ∧
              view finalSelf = initialView.drop
                (consumed + cnt.val + dstAfter.length) := by
          exact ih dstAfter.length hdecrease nextBack self' dstAfter
            (consumed + cnt.val) rfl hnextLength hnextEnough hnextView hnextBack
        obtain ⟨finalSelf, output, hloop, hcontent, hfinalView⟩ := hnext
        have htotal : consumed + cnt.val + dstAfter.length =
            consumed + n := by omega
        refine ⟨finalSelf, output, ?_, ?_, ?_⟩
        · change loop
            (fun state =>
              buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst
                state.1 state.2.1 state.2.2)
            (back, self, dst) = ok (finalSelf, output)
          rw [loop.eq_1, hbody]
          simpa [buf.buf_impl.Buf.try_copy_to_slice.default_loop, nextBack]
            using hloop
        · rw [← htotal]
          exact hcontent
        · rw [← htotal]
          exact hfinalView

theorem try_copy_to_slice_copies_model_prefix_under_coherent_callbacks
    {T : Type _} (bufInst : buf.buf_impl.Buf T) (view : T → List Std.U8)
    (self : T) (dst : Slice Std.U8) (n : Std.Usize)
    (hremaining : bufInst.remaining self = ok n)
    (hremainingView : n.val = (view self).length)
    (henough : dst.val.length ≤ (view self).length)
    (hcallbacks : ∀ (self : T) (dst : Slice Std.U8),
      core.slice.Slice.is_empty dst = ok false →
      dst.length ≤ (view self).length →
      ∃ (src : Slice Std.U8) (cnt : Std.Usize)
        (dstPrefix srcPrefix dstAfter : Slice Std.U8)
        (self' : T) (backPrefix backSuffix : Slice Std.U8 → Slice Std.U8),
        Mut0T.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok src ∧
        src.val = (view self).take src.length ∧
        core.cmp.impls.OrdUsize.min (Slice.len src) (Slice.len dst) = cnt ∧
        core.slice.index.Slice.index_mut
          (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) dst
          { «end» := cnt } = ok (dstPrefix, backPrefix) ∧
        core.slice.index.Slice.index
          (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
          { «end» := cnt } = ok srcPrefix ∧
        Slice.len dstPrefix = Slice.len srcPrefix ∧
        core.slice.index.Slice.index_mut
          (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8)
          (backPrefix srcPrefix) { start := cnt } = ok (dstAfter, backSuffix) ∧
        bufInst.advance self cnt = ok self' ∧
        0 < cnt.val ∧
        dstAfter.length + cnt.val = dst.length ∧
        view self' = (view self).drop cnt.val) :
    ∃ (finalSelf : T) (output : Slice Std.U8),
      buf.buf_impl.Buf.try_copy_to_slice.default bufInst self dst =
        ok (core.result.Result.Ok (), finalSelf, output) ∧
      output.val = (view self).take dst.length ∧
      view finalSelf = (view self).drop dst.length := by
  have hwithin : dst.val.length ≤ n.val := by omega
  have hstateEnough : dst.length ≤ (view self).length := by
    simpa [Slice.length, Slice.val, Aeneas.Data.ListN.ListN_length] using henough
  obtain ⟨finalSelf, output, hloop, hcontent, hview⟩ :=
    try_copy_loop_copies_model_prefix_under_coherent_callbacks
      bufInst view (view self) dst hcallbacks (fun s => s) self dst 0
      (by simp) hstateEnough (by simp) (by intro s _; simp)
  have hloop' : buf.buf_impl.Buf.try_copy_to_slice.default_loop
      bufInst (fun s => s) self dst = ok (finalSelf, output) := hloop
  have hcontent' : output.val = (view self).take dst.length := by
    simpa [Nat.zero_add] using hcontent
  have hview' : view finalSelf = (view self).drop dst.length := by
    simpa [Nat.zero_add] using hview
  refine ⟨finalSelf, output, ?_, hcontent', hview'⟩
  simp only [buf.buf_impl.Buf.try_copy_to_slice.default,
    Mut0T.Insts.BytesBufBuf_implBuf.remaining, hremaining]
  simp [hwithin, hloop']

theorem try_copy_to_slice_empty_destination_succeeds {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (dst : Slice Std.U8)
    (n : Std.Usize)
    (hremaining : bufInst.remaining self = ok n)
    (hempty : core.slice.Slice.is_empty dst = ok true) :
    buf.buf_impl.Buf.try_copy_to_slice.default bufInst self dst =
      ok (core.result.Result.Ok (), self, dst) := by
  have hzero : Slice.len dst = 0#usize := by
    have hlen : dst.length = 0 := by
      simpa [core.slice.Slice.is_empty] using hempty
    apply UScalar.eq_of_val_eq
    simp [hlen]
  have hbody :
      buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst
        (fun s => s) self dst =
        ok (ControlFlow.done (self, dst)) := by
    simpa using try_copy_loop_empty_body_preserves_inputs bufInst
      (fun s => s) self dst hempty
  have hloop : buf.buf_impl.Buf.try_copy_to_slice.default_loop bufInst
      (fun s => s) self dst = ok (self, dst) := by
    change loop
      (fun state =>
        buf.buf_impl.Buf.try_copy_to_slice.default_loop.body bufInst
          state.1 state.2.1 state.2.2)
      ((fun s => s), self, dst) = ok (self, dst)
    rw [loop.eq_1]
    simp [hbody]
  have hnot : ¬ n < Slice.len dst := by
    rw [hzero]
    scalar_tac
  have hwithin : dst.val.length ≤ n.val := by
    scalar_tac
  simp only [buf.buf_impl.Buf.try_copy_to_slice.default,
    Mut0T.Insts.BytesBufBuf_implBuf.remaining, hremaining]
  simp [hwithin, hloop]

theorem try_get_i8_insufficient_preserves_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : T) (n : Std.Usize)
    (hremaining : Mut0T.Insts.BytesBufBuf_implBuf.remaining bufInst self = ok n)
    (hshort : n < 1#usize) :
    buf.buf_impl.Buf.try_get_i8.default bufInst self =
      ok (core.result.Result.Err
        { requested := 1#usize, available := n }, self) := by
  have hnzero : n.val = 0 := by scalar_tac
  simp [buf.buf_impl.Buf.try_get_i8.default, hremaining, hnzero]

theorem try_get_i8_reads_head_and_advances {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T) (n : Std.Usize)
    (src : Slice Std.U8) (byte : Std.U8) (value : Std.I8)
    (hremaining : Mut0T.Insts.BytesBufBuf_implBuf.remaining bufInst self = ok n)
    (henough : ¬ n < 1#usize)
    (hchunk : Mut0T.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok src)
    (hindex : Slice.index_usize src 0#usize = ok byte)
    (hcast : UScalar.hcast .I8 byte = value)
    (hadvance : bufInst.advance self 1#usize = ok self') :
    buf.buf_impl.Buf.try_get_i8.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  have hnzero : n.val ≠ 0 := by
    intro hn
    apply henough
    scalar_tac
  simp [buf.buf_impl.Buf.try_get_i8.default, hremaining, hnzero,
    hchunk, hindex, hcast, hadvance, lift]

theorem try_get_uint_ne_forwards_native_endian {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T) (nbytes : Std.Usize)
    (result : core.result.Result Std.U64 TryGetError)
    (hle : bufInst.try_get_uint_le self nbytes = ok (result, self')) :
    buf.buf_impl.Buf.try_get_uint_ne.default bufInst self nbytes =
      ok (result, self') := by
  simp [buf.buf_impl.Buf.try_get_uint_ne.default, hle]

theorem try_get_int_ne_forwards_native_endian {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T) (nbytes : Std.Usize)
    (result : core.result.Result Std.I64 TryGetError)
    (hle : bufInst.try_get_int_le self nbytes = ok (result, self')) :
    buf.buf_impl.Buf.try_get_int_ne.default bufInst self nbytes =
      ok (result, self') := by
  simp [buf.buf_impl.Buf.try_get_int_ne.default, hle]

theorem try_get_f32_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U32) (value : F32)
    (hread : bufInst.try_get_u32 self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f32.F32.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f32.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f32.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

theorem try_get_f32_le_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U32) (value : F32)
    (hread : bufInst.try_get_u32_le self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f32.F32.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f32_le.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f32_le.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

theorem try_get_f32_ne_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U32) (value : F32)
    (hread : bufInst.try_get_u32_ne self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f32.F32.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f32_ne.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f32_ne.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

theorem try_get_f64_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U64) (value : F64)
    (hread : bufInst.try_get_u64 self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f64.F64.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f64.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f64.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

theorem try_get_f64_le_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U64) (value : F64)
    (hread : bufInst.try_get_u64_le self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f64.F64.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f64_le.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f64_le.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

theorem try_get_f64_ne_returns_bitcast_and_inner_state {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self self' : T)
    (bits : Std.U64) (value : F64)
    (hread : bufInst.try_get_u64_ne self =
      ok (core.result.Result.Ok bits, self'))
    (hfrom : core.f64.F64.from_bits bits = ok value) :
    buf.buf_impl.Buf.try_get_f64_ne.default bufInst self =
      ok (core.result.Result.Ok value, self') := by
  simp [buf.buf_impl.Buf.try_get_f64_ne.default,
    core.result.Result.Insts.CoreOpsTry.branch, hread, hfrom]

/-- Total error stubs for `Buf` fields not read by `try_copy_to_slice.default`. -/
private def failed {α : Type _} : Result α := fail .undef

/-- A Lean finite-slice cursor whose chunks contain one byte. -/
def finiteSliceCursorBuf : buf.buf_impl.Buf (Slice Std.U8) := {
  remaining := fun self => ok (Slice.len self)
  chunk := fun self => ok (Slice.from (self.val.take 1) (by scalar_tac))
  chunks_vectored := fun _ _ => failed
  advance := fun self cnt => ok (self.drop cnt)
  has_remaining := fun _ => failed
  copy_to_slice := fun _ _ => failed
  get_u8 := fun _ => failed
  get_i8 := fun _ => failed
  get_u16 := fun _ => failed
  get_u16_le := fun _ => failed
  get_u16_ne := fun _ => failed
  get_i16 := fun _ => failed
  get_i16_le := fun _ => failed
  get_i16_ne := fun _ => failed
  get_u32 := fun _ => failed
  get_u32_le := fun _ => failed
  get_u32_ne := fun _ => failed
  get_i32 := fun _ => failed
  get_i32_le := fun _ => failed
  get_i32_ne := fun _ => failed
  get_u64 := fun _ => failed
  get_u64_le := fun _ => failed
  get_u64_ne := fun _ => failed
  get_i64 := fun _ => failed
  get_i64_le := fun _ => failed
  get_i64_ne := fun _ => failed
  get_u128 := fun _ => failed
  get_u128_le := fun _ => failed
  get_u128_ne := fun _ => failed
  get_i128 := fun _ => failed
  get_i128_le := fun _ => failed
  get_i128_ne := fun _ => failed
  get_uint := fun _ _ => failed
  get_uint_le := fun _ _ => failed
  get_uint_ne := fun _ _ => failed
  get_int := fun _ _ => failed
  get_int_le := fun _ _ => failed
  get_int_ne := fun _ _ => failed
  get_f32 := fun _ => failed
  get_f32_le := fun _ => failed
  get_f32_ne := fun _ => failed
  get_f64 := fun _ => failed
  get_f64_le := fun _ => failed
  get_f64_ne := fun _ => failed
  try_copy_to_slice := fun _ _ => failed
  try_get_u8 := fun _ => failed
  try_get_i8 := fun _ => failed
  try_get_u16 := fun _ => failed
  try_get_u16_le := fun _ => failed
  try_get_u16_ne := fun _ => failed
  try_get_i16 := fun _ => failed
  try_get_i16_le := fun _ => failed
  try_get_i16_ne := fun _ => failed
  try_get_u32 := fun _ => failed
  try_get_u32_le := fun _ => failed
  try_get_u32_ne := fun _ => failed
  try_get_i32 := fun _ => failed
  try_get_i32_le := fun _ => failed
  try_get_i32_ne := fun _ => failed
  try_get_u64 := fun _ => failed
  try_get_u64_le := fun _ => failed
  try_get_u64_ne := fun _ => failed
  try_get_i64 := fun _ => failed
  try_get_i64_le := fun _ => failed
  try_get_i64_ne := fun _ => failed
  try_get_u128 := fun _ => failed
  try_get_u128_le := fun _ => failed
  try_get_u128_ne := fun _ => failed
  try_get_i128 := fun _ => failed
  try_get_i128_le := fun _ => failed
  try_get_i128_ne := fun _ => failed
  try_get_uint := fun _ _ => failed
  try_get_uint_le := fun _ _ => failed
  try_get_uint_ne := fun _ _ => failed
  try_get_int := fun _ _ => failed
  try_get_int_le := fun _ _ => failed
  try_get_int_ne := fun _ _ => failed
  try_get_f32 := fun _ => failed
  try_get_f32_le := fun _ => failed
  try_get_f32_ne := fun _ => failed
  try_get_f64 := fun _ => failed
  try_get_f64_le := fun _ => failed
  try_get_f64_ne := fun _ => failed
  copy_to_bytes := fun _ _ => failed
  take := fun _ _ => failed
  chain := fun _ _ => failed
  reader := fun _ => failed
}


/-- The one-byte cursor satisfies every callback law used by the loop theorem. -/
theorem finite_slice_cursor_callbacks (self dst : Slice Std.U8)
    (hnonempty : core.slice.Slice.is_empty dst = ok false)
    (henough : dst.length ≤ self.val.length) :
    ∃ (src : Slice Std.U8) (cnt : Std.Usize)
      (dstPrefix srcPrefix dstAfter : Slice Std.U8)
      (self' : Slice Std.U8) (backPrefix backSuffix : Slice Std.U8 → Slice Std.U8),
      Mut0T.Insts.BytesBufBuf_implBuf.chunk finiteSliceCursorBuf self = ok src ∧
      src.val = self.val.take src.length ∧
      core.cmp.impls.OrdUsize.min (Slice.len src) (Slice.len dst) = cnt ∧
      core.slice.index.Slice.index_mut
        (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) dst
        { «end» := cnt } = ok (dstPrefix, backPrefix) ∧
      core.slice.index.Slice.index
        (core.slice.index.SliceIndexRangeToUsizeSlice Std.U8) src
        { «end» := cnt } = ok srcPrefix ∧
      Slice.len dstPrefix = Slice.len srcPrefix ∧
      core.slice.index.Slice.index_mut
        (core.slice.index.SliceIndexRangeFromUsizeSlice Std.U8)
        (backPrefix srcPrefix) { start := cnt } = ok (dstAfter, backSuffix) ∧
      finiteSliceCursorBuf.advance self cnt = ok self' ∧
      0 < cnt.val ∧
      dstAfter.length + cnt.val = dst.length ∧
      self'.val = self.val.drop cnt.val := by
  have hdstNotNil : dst.val ≠ [] := by
    simpa [core.slice.Slice.is_empty] using hnonempty
  have hdstpos : 0 < dst.length := by
    cases hval : dst.val with
    | nil => contradiction
    | cons hd tl => simp [Slice.length, hval]
  have hselfpos : 0 < self.val.length := by omega
  have hdstValPos : 0 < dst.val.length := by
    simpa [Slice.length] using hdstpos
  have htakeLen : (self.val.take 1).length = 1 := by
    simp [List.length_take, Nat.min_eq_left (by omega : 1 ≤ self.val.length)]
  let src : Slice Std.U8 := Slice.from (self.val.take 1) (by scalar_tac)
  have hsrcLen : src.length = 1 := by
    simp [src, Slice.from_val, Slice.length, htakeLen]
  have hsrcValPos : 0 < src.val.length := by
    have h : 0 < src.length := by rw [hsrcLen]; omega
    simpa [Slice.length] using h
  let cnt : Std.Usize := 1#usize
  let dstPrefix : Slice Std.U8 :=
    Slice.from (dst.val.slice 0 1) (by scalar_tac)
  let backPrefix : Slice Std.U8 → Slice Std.U8 :=
    fun s => Slice.from (List.setSlice! dst.val 0 s.val) (by simp)
  let srcPrefix : Slice Std.U8 :=
    Slice.from (src.val.slice 0 1) (by scalar_tac)
  let dst1 := backPrefix srcPrefix
  let dstAfter : Slice Std.U8 := dst1.drop cnt
  let backSuffix : Slice Std.U8 → Slice Std.U8 :=
    fun s => Slice.from (dst1.val.setSlice! cnt s.val) (by simp)
  let self' : Slice Std.U8 := self.drop cnt
  have hdst1len : dst1.length = dst.length := by
    simp [dst1, backPrefix, Slice.length, Slice.from_val,
      List.length_setSlice!]
  have hcntDst : cnt ≤ dst.length := by scalar_tac
  have hcntSrc : cnt ≤ src.length := by scalar_tac
  have hcntDst1 : cnt ≤ dst1.length := by rw [hdst1len]; exact hcntDst
  refine ⟨src, cnt, dstPrefix, srcPrefix, dstAfter, self', backPrefix,
    backSuffix, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_, ?_⟩
  · simp [Mut0T.Insts.BytesBufBuf_implBuf.chunk, finiteSliceCursorBuf, src]
  · simp [src, Slice.from_val, hsrcLen]
  · apply UScalar.eq_of_val_eq
    simp [cnt, Slice.len_val, hsrcLen, core.cmp.impls.OrdUsize.min_val,
      Nat.min_eq_left (by omega : 1 ≤ dst.length)]
  · simp only [core.slice.index.Slice.index_mut,
      core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
    rw [if_pos hcntDst]
    rfl
  · simp only [core.slice.index.Slice.index,
      core.slice.index.SliceIndexRangeToUsizeSlice.index]
    rw [if_pos hcntSrc]
    rfl
  · apply UScalar.eq_of_val_eq
    simp [dstPrefix, srcPrefix, Slice.len_val, List.slice,
      List.length_take, Nat.min_eq_left (by omega : 1 ≤ dst.val.length),
      Nat.min_eq_left (by omega : 1 ≤ src.val.length), hsrcLen,
      hdstValPos, hsrcValPos]
  · simp only [core.slice.index.Slice.index_mut,
      core.slice.index.SliceIndexRangeFromUsizeSlice.index_mut]
    rw [if_pos hcntDst1]
  · simp [finiteSliceCursorBuf, self', cnt]
  · simp [cnt]
  · have hcntNat : cnt.val ≤ dst1.val.length := by
      simpa [Slice.length] using hcntDst1
    have hafterLen : dstAfter.length + cnt.val = dst1.length := by
      simp only [dstAfter, Slice.length, Slice.drop, Slice.from_val,
        List.length_drop]
      omega
    calc
      dstAfter.length + cnt.val = dst1.length := hafterLen
      _ = dst.length := hdst1len
  · simp [self', Slice.drop, cnt]


/-- The translated default copies a finite-slice cursor prefix through repeated one-byte chunks. -/
theorem finite_slice_cursor_try_copy_to_slice_prefix
    (self dst : Slice Std.U8)
    (henough : dst.length ≤ self.val.length) :
    ∃ (finalSelf output : Slice Std.U8),
      buf.buf_impl.Buf.try_copy_to_slice.default finiteSliceCursorBuf self dst =
        ok (core.result.Result.Ok (), finalSelf, output) ∧
      output.val = self.val.take dst.length ∧
      finalSelf.val = self.val.drop dst.length := by
  apply try_copy_to_slice_copies_model_prefix_under_coherent_callbacks
    finiteSliceCursorBuf (fun s => s.val) self dst (Slice.len self)
  · rfl
  · simp
  · simpa [Slice.length] using henough
  · intro state destination hnonempty hstateEnough
    exact finite_slice_cursor_callbacks state destination hnonempty hstateEnough

end BytesProofs.Readers
