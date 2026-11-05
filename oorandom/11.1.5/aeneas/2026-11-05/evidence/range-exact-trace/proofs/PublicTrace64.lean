import ActualLoop64
open Aeneas Aeneas.Std Result
open oorandom_range_original Zero64
namespace PublicTrace64
open RangeComplete64
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U64.rMax U128.rMax I64.rMax I64.rMin

def initial (self : Rand64) (width : U64) := ActualLoop64.next width (self,0#u128,0#u64)

theorem initial_draw (self : Rand64) (width : U64) (positive : 0<width.val) :
 draw self width = .ok (initial self width) := by
 have hm : (core.convert.num.FromU128U64.from (output64 self.state) * core.convert.num.FromU128U64.from width : Result U128) =
  .ok (core.num.U128.wrapping_mul (core.convert.num.FromU128U64.from (output64 self.state)) (core.convert.num.FromU128U64.from width)) := by
  apply RangeProofs.spec_eq_ok
  refine WP.spec_mono (RangeArithmetic.wide_product64 (output64 self.state) width positive) ?_
  intro m h
  apply UScalar.val_eq_imp
  rw [h,ActualLoop64.wrapping_product64 _ _ positive]
 unfold draw
 rw [next_spec]
 simp only [bind_ok,bind_tc_ok,lift,Std.uncurry,hm,initial,ActualLoop64.next]

theorem afterDraw_finite (start width threshold : U64) (q : ActualLoop64.Candidate) (n : Nat)
 (threshold_call : (U64.wrapping_sub 0#u64 width % width : Result U64) = .ok threshold)
 (threshold_lt : threshold.val<width.val)
 (prior : ∀ k, k<n → ¬ActualLoop64.accepted threshold (GenericTrace.trace (ActualLoop64.next width) q k))
 (accept : ActualLoop64.accepted threshold (GenericTrace.trace (ActualLoop64.next width) q n)) :
 afterDraw start width q = finish start ((GenericTrace.trace (ActualLoop64.next width) q n).1,(GenericTrace.trace (ActualLoop64.next width) q n).2.1) := by
 unfold afterDraw
 split
 · simp only [core.num.U64.wrapping_neg,bind_ok]
   rw [threshold_call]
   simp only [bind_ok]
   rw [ActualLoop64.finite width threshold q n (by omega) prior accept]
   simp only [bind_ok]
 · rename_i fast
   have hn : n=0 := by
    by_contra hn
    have h := prior 0 (Nat.pos_of_ne_zero hn)
    simp only [GenericTrace.trace,ActualLoop64.accepted,not_not] at h
    have : q.2.2.val < threshold.val := h
    have : ¬q.2.2.val < width.val := fast
    omega
   subst n
   rfl

theorem finish_exact (start finishBound width : U64) (valid : start.val < finishBound.val)
 (hw : width.val = finishBound.val-start.val) (q : Rand64 × U128)
 (bound : q.2.val < 2^64*width.val) :
 finish start q ⦃ r => r.1.val=q.2.val/2^64+start.val ∧ r.2=q.1 ⦄ := by
 have highlt : q.2.val / 2^64 < width.val := by
  exact (Nat.div_lt_iff_lt_mul (Nat.two_pow_pos 64)).2 (by simpa [Nat.mul_comm] using bound)
 have offset := RangeArithmetic.offset64 start finishBound valid (q.2.val/2^64) (by simpa [hw] using highlt)
 unfold finish
 refine WP.spec_bind (U128.ShiftRight_IScalar_spec q.2 (64#i64) (by decide) (by decide)) ?_
 intro shifted hshift
 have hs : shifted.val=q.2.val/2^64 := by simpa only [show (64#i64).toNat=64 from rfl, Nat.shiftRight_eq_div_pow] using hshift.1
 have hc : (UScalar.cast .U64 shifted).val=q.2.val/2^64 := by
  simp only [UScalar.cast_val_eq,UScalarTy.U64_numBits_eq,hs]
  exact Nat.mod_eq_of_lt offset.2.1
 simp only [lift,bind_ok,bind_tc_ok]
 refine WP.spec_bind (U64.add_spec (x:=UScalar.cast .U64 shifted) (y:=start) (by
  simp only [hc,U64.max,U64.numBits_eq];omega)) ?_
 intro out hout
 simp only [WP.spec_ok]
 rw [hc] at hout
 exact ⟨hout,True.intro⟩

theorem next_bound (width : U64) (positive : 0<width.val) (q : ActualLoop64.Candidate) :
 (ActualLoop64.next width q).2.1.val < 2^64*width.val := by
 change (core.num.U128.wrapping_mul _ _).val < _
 rw [ActualLoop64.wrapping_product64 _ _ positive]
 exact (RangeArithmetic.product64 _ width positive).1

theorem candidate_bound (self : Rand64) (width : U64) (positive : 0<width.val) (n : Nat) :
 (GenericTrace.trace (ActualLoop64.next width) (initial self width) n).2.1.val < 2^64*width.val := by
 cases n with
 | zero => exact next_bound width positive _
 | succ n => rw [GenericTrace.trace_succ];exact next_bound width positive _

theorem public_finite (self : Rand64) (range : core.ops.range.Range U64) (width threshold : U64) (n : Nat)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (threshold_call : (U64.wrapping_sub 0#u64 width % width : Result U64) = .ok threshold)
 (threshold_lt : threshold.val<width.val)
 (prior : ∀ k, k<n → ¬ActualLoop64.accepted threshold (GenericTrace.trace (ActualLoop64.next width) (initial self width) k))
 (accept : ActualLoop64.accepted threshold (GenericTrace.trace (ActualLoop64.next width) (initial self width) n)) :
 Rand64.rand_range self range ⦃ out =>
  out.1.val=(GenericTrace.trace (ActualLoop64.next width) (initial self width) n).2.1.val/2^64+range.start.val ∧
  out.2=(GenericTrace.trace (ActualLoop64.next width) (initial self width) n).1 ⦄ := by
 have hv : range.start.val<range.end.val := valid
 have hs : 0<width.val := by omega
 have hsub : (range.end-range.start : Result U64)=.ok width := by
  apply RangeProofs.spec_eq_ok
  refine WP.spec_mono (U64.sub_spec (x:=range.end) (y:=range.start) (by omega)) ?_
  intro actual ha
  apply UScalar.val_eq_imp
  omega
 rw [RangeComplete64.decompose]
 simp only [massert,if_pos valid,bind_ok,bind_tc_ok,hsub]
 rw [initial_draw self width hs]
 simp only [bind_ok]
 rw [afterDraw_finite range.start width threshold (initial self width) n threshold_call threshold_lt prior accept]
 exact finish_exact range.start range.end width hv hw _ (candidate_bound self width hs n)

def threshold (width : U64) : U64 := ⟨(U64.wrapping_sub 0#u64 width).bv % width.bv⟩

theorem threshold_call (width : U64) (positive : 0<width.val) :
 (U64.wrapping_sub 0#u64 width % width : Result U64)=.ok (threshold width) := by
 simp only [HMod.hMod,UScalar.rem,show (width.val != 0)=true from by simp [Nat.ne_of_gt positive],if_true,threshold,BitVec.umod_eq]

theorem threshold_value (width : U64) (positive : 0<width.val) :
 (threshold width).val=(2^64-width.val)%width.val := by
 have hb : 2^64-width.val<2^64 := by omega
 change ((U64.wrapping_sub 0#u64 width).bv % width.bv).toNat = _
 rw [BitVec.toNat_umod]
 change (U64.wrapping_sub 0#u64 width).val % width.val = _
 simp only [U64.wrapping_sub_val_eq,UScalar.size_def,UScalarTy.U64_numBits_eq,show (0#u64).val=0 from rfl,Nat.zero_add,Nat.mod_eq_of_lt hb]

theorem threshold_bound (width : U64) (positive : 0<width.val) :
 (threshold width).val<width.val := by
 rw [threshold_value width positive]
 exact Nat.mod_lt _ positive

theorem public_first_accept (self : Rand64) (range : core.ops.range.Range U64) (width : U64) (n : Nat)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (prior : ∀ k, k<n → ¬ActualLoop64.accepted (threshold width) (GenericTrace.trace (ActualLoop64.next width) (initial self width) k))
 (accept : ActualLoop64.accepted (threshold width) (GenericTrace.trace (ActualLoop64.next width) (initial self width) n)) :
 Rand64.rand_range self range ⦃ out =>
  out.1.val=(GenericTrace.trace (ActualLoop64.next width) (initial self width) n).2.1.val/2^64+range.start.val ∧
  out.2=(GenericTrace.trace (ActualLoop64.next width) (initial self width) n).1 ⦄ := by
 have hv : range.start.val<range.end.val := valid
 have positive : 0<width.val := by omega
 exact public_finite self range width (threshold width) n valid hw (threshold_call width positive) (threshold_bound width positive) prior accept

theorem public_all_rejected (self : Rand64) (range : core.ops.range.Range U64) (width : U64)
 (valid : range.start<range.end) (hw : width.val=range.end.val-range.start.val)
 (reject : ∀ k, ¬ActualLoop64.accepted (threshold width) (GenericTrace.trace (ActualLoop64.next width) (initial self width) k)) :
 Rand64.rand_range self range = .div := by
 have hv : range.start.val<range.end.val := valid
 have positive : 0<width.val := by omega
 have hsub : (range.end-range.start : Result U64)=.ok width := by
  apply RangeProofs.spec_eq_ok
  refine WP.spec_mono (U64.sub_spec (x:=range.end) (y:=range.start) (by omega)) ?_
  intro actual ha
  apply UScalar.val_eq_imp
  omega
 rw [RangeComplete64.decompose]
 simp only [massert,if_pos valid,bind_ok,bind_tc_ok,hsub]
 rw [initial_draw self width positive]
 simp only [bind_ok]
 have hr : (initial self width).2.2 < threshold width := by
  simpa only [ActualLoop64.accepted,GenericTrace.trace,not_not] using reject 0
 have ht := threshold_bound width positive
 have fast : (initial self width).2.2 < width := by
  have : (initial self width).2.2.val < (threshold width).val := hr
  exact lt_trans this ht
 unfold afterDraw
 rw [if_pos fast]
 simp only [core.num.U64.wrapping_neg,bind_ok]
 rw [threshold_call width positive]
 simp only [bind_ok]
 rw [ActualLoop64.infinite width (threshold width) (initial self width) positive reject]
 simp only [bind_div]

#print axioms public_all_rejected
#print axioms threshold_value
#print axioms public_first_accept
#print axioms public_finite
#print axioms finish_exact
#print axioms initial_draw
#print axioms afterDraw_finite
end PublicTrace64
