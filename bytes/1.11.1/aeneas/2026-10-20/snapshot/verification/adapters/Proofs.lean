import AdaptersGenerated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Adapters

/-!
These statements unfold the adapter functions translated from bytes 1.11.1.
The hypotheses expose only the operations supplied by the input `Buf`
dictionary; they do not add assumptions about its representation or copy its
implementation into a model.
-/

/-- The scalar value represented by a successful unsigned subtraction. -/
def usizeSubValue (x y : Std.Usize) : Std.Usize :=
  Usize.ofNatCore (x.val - y.val)
    (Nat.lt_of_le_of_lt (Nat.sub_le _ _) (UScalar.hBounds x))

theorem usizeSubValue_value (x y : Std.Usize) :
    (usizeSubValue x y).val = x.val - y.val := by
  simp [usizeSubValue, Usize.ofNatCore_val_eq]

theorem usize_sub_eq_ok_subValue (x y : Std.Usize) (hle : y.val ≤ x.val) :
    x - y = ok (usizeSubValue x y) := by
  change UScalar.sub x y = ok (usizeSubValue x y)
  unfold UScalar.sub usizeSubValue
  have hnot : ¬ x.val < y.val := Nat.not_lt.mpr hle
  rw [ite_eq_right hnot]
  apply congrArg ok
  apply UScalar.eq_of_val_eq
  have hbound : x.val - y.val < 2 ^ UScalarTy.Usize.numBits :=
    Nat.lt_of_le_of_lt (Nat.sub_le _ _) x.hBounds
  have hboundBits : x.val - y.val < 2 ^ System.Platform.numBits := by
    simpa [UScalarTy.Usize_numBits_eq] using hbound
  change (BitVec.ofNat UScalarTy.Usize.numBits (x.val - y.val)).toNat =
    (Usize.ofNatCore (x.val - y.val) _).val
  rw [BitVec.toNat_ofNat, Nat.mod_eq_of_lt hbound]
  exact Usize.ofNatCore_val_eq hbound

theorem take_new_fields {T : Type _} (inner : T) (limit : Std.Usize) :
    buf.take.new inner limit = ok ({ inner := inner, limit := limit } : buf.take.Take T) := by
  rfl

theorem take_limit_returns_field {T : Type _} (self : buf.take.Take T) :
    buf.take.Take.impl.limit self = ok self.limit := by
  rfl

theorem take_into_inner_returns_field {T : Type _} (self : buf.take.Take T) :
    buf.take.Take.into_inner self = ok self.inner := by
  rfl

theorem take_get_ref_returns_field {T : Type _} (self : buf.take.Take T) :
    buf.take.Take.get_ref self = ok self.inner := by
  rfl

theorem take_set_limit_changes_only_limit {T : Type _}
    (self : buf.take.Take T) (limit : Std.Usize) :
    buf.take.Take.set_limit self limit =
      ok ({ inner := self.inner, limit := limit } : buf.take.Take T) := by
  rfl

/-- `Take.remaining` returns the pure `OrdUsize.min` result after obtaining the
    inner count from its `Buf` dictionary. -/
theorem take_remaining_uses_min {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.take.Take T)
    (innerRemaining : Std.Usize)
    (hinner : bufInst.remaining self.inner = ok innerRemaining) :
    buf.take.Take.Insts.BytesBufBuf_implBuf.remaining bufInst self =
      ok (core.cmp.impls.OrdUsize.min innerRemaining self.limit) := by
  simp [buf.take.Take.Insts.BytesBufBuf_implBuf.remaining, hinner,
    core.cmp.min]

theorem take_remaining_is_bounded {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.take.Take T)
    (innerRemaining : Std.Usize)
    (hinner : bufInst.remaining self.inner = ok innerRemaining) :
    ∃ remaining, buf.take.Take.Insts.BytesBufBuf_implBuf.remaining bufInst self =
      ok remaining ∧ remaining ≤ innerRemaining ∧ remaining ≤ self.limit := by
  let remaining := core.cmp.impls.OrdUsize.min innerRemaining self.limit
  have hminValue : remaining.val = Nat.min innerRemaining.val self.limit.val := by
    simp [remaining]
  refine ⟨remaining, ?_, ?_, ?_⟩
  · exact take_remaining_uses_min bufInst self innerRemaining hinner
  · have hval : remaining.val ≤ innerRemaining.val := by
      rw [hminValue]
      exact Nat.min_le_left _ _
    scalar_tac
  · have hval : remaining.val ≤ self.limit.val := by
      rw [hminValue]
      exact Nat.min_le_right _ _
    scalar_tac

/-- `Take.chunk` returns the prefix of the inner chunk whose length is the
    concrete scalar minimum of the inner length and the stored limit. -/
theorem take_chunk_returns_bounded_prefix {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.take.Take T)
    (innerChunk : Slice Std.U8)
    (hchunk : bufInst.chunk self.inner = ok innerChunk) :
    ∃ visible : Slice Std.U8,
      buf.take.Take.Insts.BytesBufBuf_implBuf.chunk bufInst self = ok visible ∧
      visible.val = innerChunk.val.take
        (core.cmp.impls.OrdUsize.min (Slice.len innerChunk) self.limit).val ∧
      visible.length =
        (core.cmp.impls.OrdUsize.min (Slice.len innerChunk) self.limit).val ∧
      core.cmp.impls.OrdUsize.min (Slice.len innerChunk) self.limit ≤ self.limit := by
  let n := core.cmp.impls.OrdUsize.min (Slice.len innerChunk) self.limit
  have hminValue : n.val = Nat.min innerChunk.length self.limit.val := by
    simp [n]
  have hnNat : n.val ≤ innerChunk.length := by
    rw [hminValue]
    exact Nat.min_le_left _ _
  have hnLimitNat : n.val ≤ self.limit.val := by
    rw [hminValue]
    exact Nat.min_le_right _ _
  have hnChunk : n ≤ innerChunk.length := by
    have hval : n.val ≤ innerChunk.length := hnNat
    scalar_tac
  have hnLimit : n ≤ self.limit := by
    have hval : n.val ≤ self.limit.val := hnLimitNat
    scalar_tac
  have hprefix : innerChunk.val.slice 0 n.val = innerChunk.val.take n.val := by
    simp [List.slice]
  have hsliceLength : (innerChunk.val.slice 0 n.val).length = n.val := by
    rw [List.slice_length]
    simp only [Nat.sub_zero]
    exact Nat.min_eq_right hnNat
  have hsliceBound : (innerChunk.val.slice 0 n.val).length ≤ Usize.max := by
    exact Nat.le_trans (List.slice_length_le 0 n.val innerChunk.val)
      (Slice.length_ineq innerChunk)
  let visible := Slice.from (innerChunk.val.slice 0 n.val) hsliceBound
  have hresult : buf.take.Take.Insts.BytesBufBuf_implBuf.chunk bufInst self =
      ok visible := by
    simp [buf.take.Take.Insts.BytesBufBuf_implBuf.chunk, hchunk, n,
      core.cmp.min, core.slice.index.Slice.index,
      core.slice.index.SliceIndexRangeToUsizeSlice.index, visible]
  refine ⟨visible, hresult, ?_, ?_, hnLimit⟩
  · simp only [visible, Slice.from_val]
    rw [hprefix, hminValue]
  · simp only [visible, Slice.length, Slice.from_val]
    rw [hsliceLength, hminValue]

theorem take_advance_updates_inner_and_limit {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.take.Take T)
    (cnt : Std.Usize) (inner' : T)
    (hwithin : cnt ≤ self.limit)
    (hinner : bufInst.advance self.inner cnt = ok inner') :
    ∃ newLimit : Std.Usize,
      newLimit.val = self.limit.val - cnt.val ∧
      buf.take.Take.Insts.BytesBufBuf_implBuf.advance bufInst self cnt =
        ok ({ inner := inner', limit := newLimit } : buf.take.Take T) := by
  have hcnt : cnt.val ≤ self.limit.val := by scalar_tac
  let newLimit := usizeSubValue self.limit cnt
  have hnewLimit : newLimit.val = self.limit.val - cnt.val :=
    usizeSubValue_value self.limit cnt
  have hsub : self.limit - cnt = ok newLimit :=
    usize_sub_eq_ok_subValue self.limit cnt hcnt
  refine ⟨newLimit, hnewLimit, ?_⟩
  simp [buf.take.Take.Insts.BytesBufBuf_implBuf.advance, hwithin, hinner, hsub]

/-- Under an explicit observation law for the inner Buf, advancing a `Take`
    drops exactly that prefix from the previously visible contents. -/
theorem take_advance_drops_observed_prefix {T : Type _}
    (bufInst : buf.buf_impl.Buf T) (self : buf.take.Take T)
    (cnt : Std.Usize) (inner' : T) (view : T → List Std.U8)
    (hwithin : cnt ≤ self.limit)
    (hinner : bufInst.advance self.inner cnt = ok inner')
    (hview : view inner' = (view self.inner).drop cnt.val) :
    ∃ newLimit : Std.Usize,
      buf.take.Take.Insts.BytesBufBuf_implBuf.advance bufInst self cnt =
        ok ({ inner := inner', limit := newLimit } : buf.take.Take T) ∧
      newLimit.val = self.limit.val - cnt.val ∧
      (view inner').take newLimit.val =
        ((view self.inner).take self.limit.val).drop cnt.val := by
  obtain ⟨newLimit, hnewLimit, hadvance⟩ :=
    take_advance_updates_inner_and_limit bufInst self cnt inner' hwithin hinner
  refine ⟨newLimit, hadvance, hnewLimit, ?_⟩
  rw [hview, hnewLimit]
  exact (List.drop_take (i := cnt.val) (j := self.limit.val)
    (l := view self.inner)).symm

theorem chain_new_fields {T U : Type _} (a : T) (b : U) :
    buf.chain.Chain.new a b = ok ({ a := a, b := b } : buf.chain.Chain T U) := by
  rfl

/-- `Chain.remaining` adds the two counts with the translated saturating add. -/
theorem chain_remaining_is_saturating_sum {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (aRemaining bRemaining : Std.Usize)
    (ha : bufInstA.remaining self.a = ok aRemaining)
    (hb : bufInstB.remaining self.b = ok bRemaining) :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.remaining bufInstA bufInstB self =
      ok (core.num.Usize.saturating_add aRemaining bRemaining) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.remaining, ha, hb]

/- The three advance lemmas below characterize each branch in the translated
   implementation. The input Buf operations remain quantified dictionary
   arguments with explicit successful-result hypotheses. -/

theorem chain_advance_within_first {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (cnt aRemaining : Std.Usize) (a' : T)
    (hremaining : bufInstA.remaining self.a = ok aRemaining)
    (hnonzero : aRemaining ≠ 0#usize)
    (hwithin : cnt ≤ aRemaining)
    (hadvance : bufInstA.advance self.a cnt = ok a') :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance bufInstA bufInstB self cnt =
      ok ({ a := a', b := self.b } : buf.chain.Chain T U) := by
  have hnonzeroVal : aRemaining.val ≠ 0 := by
    intro hval
    apply hnonzero
    apply UScalar.eq_of_val_eq
    simpa using hval
  have hwithinVal : cnt.val ≤ aRemaining.val := by scalar_tac
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance, hremaining,
    hnonzeroVal, hwithinVal, hadvance]

theorem chain_advance_crosses_boundary {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (cnt aRemaining : Std.Usize)
    (a' : T) (b' : U)
    (hremaining : bufInstA.remaining self.a = ok aRemaining)
    (hnonzero : aRemaining ≠ 0#usize)
    (hcroses : ¬ cnt ≤ aRemaining)
    (ha : bufInstA.advance self.a aRemaining = ok a')
    (hb : bufInstB.advance self.b (usizeSubValue cnt aRemaining) = ok b') :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance bufInstA bufInstB self cnt =
      ok ({ a := a', b := b' } : buf.chain.Chain T U) := by
  have hle : aRemaining.val ≤ cnt.val := by scalar_tac
  have hnonzeroVal : aRemaining.val ≠ 0 := by
    intro hval
    apply hnonzero
    apply UScalar.eq_of_val_eq
    simpa using hval
  have hcrossesVal : ¬ cnt.val ≤ aRemaining.val := by
    intro hval
    apply hcroses
    scalar_tac
  have hsub : cnt - aRemaining = ok (usizeSubValue cnt aRemaining) :=
    usize_sub_eq_ok_subValue cnt aRemaining hle
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance, hremaining,
    hnonzeroVal, hcrossesVal, ha, hsub, hb]

theorem chain_advance_starts_in_second {T U : Type _}
    (bufInstA : buf.buf_impl.Buf T) (bufInstB : buf.buf_impl.Buf U)
    (self : buf.chain.Chain T U) (cnt aRemaining : Std.Usize) (b' : U)
    (hremaining : bufInstA.remaining self.a = ok aRemaining)
    (hzero : aRemaining = 0#usize)
    (hadvance : bufInstB.advance self.b cnt = ok b') :
    buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance bufInstA bufInstB self cnt =
      ok ({ a := self.a, b := b' } : buf.chain.Chain T U) := by
  simp [buf.chain.Chain.Insts.BytesBufBuf_implBuf.advance, hremaining,
    hzero, hadvance]

#print axioms take_new_fields
#print axioms usize_sub_eq_ok_subValue
#print axioms usizeSubValue_value
#print axioms take_limit_returns_field
#print axioms take_into_inner_returns_field
#print axioms take_get_ref_returns_field
#print axioms take_set_limit_changes_only_limit
#print axioms take_remaining_uses_min
#print axioms take_remaining_is_bounded
#print axioms take_chunk_returns_bounded_prefix
#print axioms take_advance_updates_inner_and_limit
#print axioms take_advance_drops_observed_prefix
#print axioms chain_new_fields
#print axioms chain_remaining_is_saturating_sum
#print axioms chain_advance_within_first
#print axioms chain_advance_crosses_boundary
#print axioms chain_advance_starts_in_second
#print axioms take_advance_drops_observed_prefix
#print axioms chain_new_fields
#print axioms chain_remaining_is_saturating_sum
#print axioms chain_advance_within_first
#print axioms chain_advance_crosses_boundary
#print axioms chain_advance_starts_in_second

end BytesProofs.Adapters
