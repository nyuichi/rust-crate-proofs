import ActualLoop32
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace PublicTrace32
open RangeComplete32
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U32.rMax U64.rMax I32.rMax I32.rMin

def initial (self : Rand32) (width : U32) := ActualLoop32.next width (self,0#u64,0#u32)

theorem initial_draw (self : Rand32) (width : U32) (positive : 0<width.val) :
 draw self width = .ok (initial self width) := by
 have hm : (core.convert.num.FromU64U32.from (output32 self.state) * core.convert.num.FromU64U32.from width : Result U64) =
  .ok (core.num.U64.wrapping_mul (core.convert.num.FromU64U32.from (output32 self.state)) (core.convert.num.FromU64U32.from width)) := by
  apply spec_eq_ok
  refine WP.spec_mono (RangeArithmetic.wide_product32 (output32 self.state) width positive) ?_
  intro m h
  apply UScalar.val_eq_imp
  rw [h,RangeArithmetic.wrapping_product32 _ _ positive]
 unfold draw
 rw [rand_u32_spec]
 simp only [bind_ok,bind_tc_ok,lift,Std.uncurry,hm,initial,ActualLoop32.next]

theorem afterDraw_finite (start width threshold : U32) (q : ActualLoop32.Candidate) (n : Nat)
 (threshold_call : (U32.wrapping_sub 0#u32 width % width : Result U32) = .ok threshold)
 (threshold_lt : threshold.val<width.val)
 (prior : ∀ k, k<n → ¬ActualLoop32.accepted threshold (GenericTrace.trace (ActualLoop32.next width) q k))
 (accept : ActualLoop32.accepted threshold (GenericTrace.trace (ActualLoop32.next width) q n)) :
 afterDraw start width q = finish start ((GenericTrace.trace (ActualLoop32.next width) q n).1,(GenericTrace.trace (ActualLoop32.next width) q n).2.1) := by
 unfold afterDraw
 split
 · simp only [core.num.U32.wrapping_neg,bind_ok]
   rw [threshold_call]
   simp only [bind_ok]
   rw [ActualLoop32.finite width threshold q n prior accept]
   simp only [bind_ok]
 · rename_i fast
   have hn : n=0 := by
    by_contra hn
    have h := prior 0 (Nat.pos_of_ne_zero hn)
    simp only [GenericTrace.trace,ActualLoop32.accepted,not_not] at h
    have : q.2.2.val < threshold.val := h
    have : ¬q.2.2.val < width.val := fast
    omega
   subst n
   rfl

theorem finish_exact (start finishBound width : U32) (valid : start.val < finishBound.val)
 (hw : width.val = finishBound.val-start.val) (q : Rand32 × U64)
 (bound : q.2.val < 2^32*width.val) :
 finish start q ⦃ r => r.1.val=q.2.val/2^32+start.val ∧ r.2=q.1 ⦄ := by
 have highlt : q.2.val / 2^32 < width.val := by
  exact (Nat.div_lt_iff_lt_mul (Nat.two_pow_pos 32)).2 (by simpa [Nat.mul_comm] using bound)
 have offset := RangeArithmetic.offset32 start finishBound valid (q.2.val/2^32) (by simpa [hw] using highlt)
 unfold finish
 refine WP.spec_bind (U64.ShiftRight_IScalar_spec q.2 (32#i32) (by decide) (by decide)) ?_
 intro shifted hshift
 have hs : shifted.val=q.2.val/2^32 := by simpa only [show (32#i32).toNat=32 from rfl, Nat.shiftRight_eq_div_pow] using hshift.1
 have hc : (UScalar.cast .U32 shifted).val=q.2.val/2^32 := by
  simp only [UScalar.cast_val_eq,UScalarTy.U32_numBits_eq,hs]
  exact Nat.mod_eq_of_lt offset.2.1
 simp only [lift,bind_ok,bind_tc_ok]
 refine WP.spec_bind (U32.add_spec (x:=UScalar.cast .U32 shifted) (y:=start) (by
  simp only [hc,U32.max,U32.numBits_eq];omega)) ?_
 intro out hout
 simp only [WP.spec_ok]
 rw [hc] at hout
 exact ⟨hout,True.intro⟩

theorem next_bound (width : U32) (positive : 0<width.val) (q : ActualLoop32.Candidate) :
 (ActualLoop32.next width q).2.1.val < 2^32*width.val := by
 change (core.num.U64.wrapping_mul _ _).val < _
 rw [RangeArithmetic.wrapping_product32 _ _ positive]
 exact (RangeArithmetic.product32 _ width positive).1

theorem candidate_bound (self : Rand32) (width : U32) (positive : 0<width.val) (n : Nat) :
 (GenericTrace.trace (ActualLoop32.next width) (initial self width) n).2.1.val < 2^32*width.val := by
 cases n with
 | zero => exact next_bound width positive _
 | succ n => rw [GenericTrace.trace_succ];exact next_bound width positive _

theorem public_finite (self : Rand32) (range : core.ops.range.Range U32) (width threshold : U32) (n : Nat)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (threshold_call : (U32.wrapping_sub 0#u32 width % width : Result U32) = .ok threshold)
 (threshold_lt : threshold.val<width.val)
 (prior : ∀ k, k<n → ¬ActualLoop32.accepted threshold (GenericTrace.trace (ActualLoop32.next width) (initial self width) k))
 (accept : ActualLoop32.accepted threshold (GenericTrace.trace (ActualLoop32.next width) (initial self width) n)) :
 Rand32.rand_range self range ⦃ out =>
  out.1.val=(GenericTrace.trace (ActualLoop32.next width) (initial self width) n).2.1.val/2^32+range.start.val ∧
  out.2=(GenericTrace.trace (ActualLoop32.next width) (initial self width) n).1 ⦄ := by
 have hv : range.start.val<range.end.val := valid
 have hs : 0<width.val := by omega
 have hsub : (range.end-range.start : Result U32)=.ok width := by
  apply spec_eq_ok
  refine WP.spec_mono (U32.sub_spec (x:=range.end) (y:=range.start) (by omega)) ?_
  intro actual ha
  apply UScalar.val_eq_imp
  omega
 rw [decompose]
 simp only [massert,if_pos valid,bind_ok,bind_tc_ok,hsub]
 rw [initial_draw self width hs]
 simp only [bind_ok]
 rw [afterDraw_finite range.start width threshold (initial self width) n threshold_call threshold_lt prior accept]
 exact finish_exact range.start range.end width hv hw _ (candidate_bound self width hs n)

def threshold (width : U32) : U32 := ⟨(U32.wrapping_sub 0#u32 width).bv % width.bv⟩

theorem threshold_call (width : U32) (positive : 0<width.val) :
 (U32.wrapping_sub 0#u32 width % width : Result U32)=.ok (threshold width) := by
 simp only [HMod.hMod,UScalar.rem,show (width.val != 0)=true from by simp [Nat.ne_of_gt positive],if_true,threshold,BitVec.umod_eq]

theorem threshold_value (width : U32) (positive : 0<width.val) :
 (threshold width).val=(2^32-width.val)%width.val := by
 have hb : 2^32-width.val<2^32 := by omega
 change ((U32.wrapping_sub 0#u32 width).bv % width.bv).toNat = _
 rw [BitVec.toNat_umod]
 change (U32.wrapping_sub 0#u32 width).val % width.val = _
 simp only [U32.wrapping_sub_val_eq,UScalar.size_def,UScalarTy.U32_numBits_eq,show (0#u32).val=0 from rfl,Nat.zero_add,Nat.mod_eq_of_lt hb]

theorem threshold_bound (width : U32) (positive : 0<width.val) :
 (threshold width).val<width.val := by
 rw [threshold_value width positive]
 exact Nat.mod_lt _ positive

theorem public_first_accept (self : Rand32) (range : core.ops.range.Range U32) (width : U32) (n : Nat)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (prior : ∀ k, k<n → ¬ActualLoop32.accepted (threshold width) (GenericTrace.trace (ActualLoop32.next width) (initial self width) k))
 (accept : ActualLoop32.accepted (threshold width) (GenericTrace.trace (ActualLoop32.next width) (initial self width) n)) :
 Rand32.rand_range self range ⦃ out =>
  out.1.val=(GenericTrace.trace (ActualLoop32.next width) (initial self width) n).2.1.val/2^32+range.start.val ∧
  out.2=(GenericTrace.trace (ActualLoop32.next width) (initial self width) n).1 ⦄ := by
 have hv : range.start.val<range.end.val := valid
 have positive : 0<width.val := by omega
 exact public_finite self range width (threshold width) n valid hw (threshold_call width positive) (threshold_bound width positive) prior accept

theorem public_all_rejected (self : Rand32) (range : core.ops.range.Range U32) (width : U32)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (reject : ∀ k, ¬ActualLoop32.accepted (threshold width) (GenericTrace.trace (ActualLoop32.next width) (initial self width) k)) :
 Rand32.rand_range self range = .div := by
 have hv : range.start.val<range.end.val := valid
 have positive : 0<width.val := by omega
 have hsub : (range.end-range.start : Result U32)=.ok width := by
  apply RangeProofs.spec_eq_ok
  refine WP.spec_mono (U32.sub_spec (x:=range.end) (y:=range.start) (by omega)) ?_
  intro actual ha
  apply UScalar.val_eq_imp
  omega
 rw [RangeComplete32.decompose]
 simp only [massert,if_pos valid,bind_ok,bind_tc_ok,hsub]
 rw [initial_draw self width positive]
 simp only [bind_ok]
 have hr : (initial self width).2.2 < threshold width := by
  simpa only [ActualLoop32.accepted,GenericTrace.trace,not_not] using reject 0
 have ht := threshold_bound width positive
 have fast : (initial self width).2.2 < width := by
  have : (initial self width).2.2.val < (threshold width).val := hr
  exact lt_trans this ht
 unfold afterDraw
 rw [if_pos fast]
 simp only [core.num.U32.wrapping_neg,bind_ok]
 rw [threshold_call width positive]
 simp only [bind_ok]
 rw [ActualLoop32.infinite width (threshold width) (initial self width) reject]
 simp only [bind_div]

#print axioms public_all_rejected
#print axioms threshold_value
#print axioms public_first_accept
#print axioms public_finite
#print axioms finish_exact
#print axioms initial_draw
#print axioms afterDraw_finite
end PublicTrace32
