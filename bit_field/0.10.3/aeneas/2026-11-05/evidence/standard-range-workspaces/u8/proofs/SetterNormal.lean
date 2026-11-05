import SetterMath
import EmptyNonzero
open Aeneas Aeneas.Std Result
open bitfield_range_u8_inline U8RangeFailure
namespace U8RangeSetter
set_option maxHeartbeats 700000
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U8.rMax

def Post (value replacement : U8) (start finish : Usize) (out : U8 × (U8 → U8)) : Prop :=
 (∀ i, i<8 → out.1.bv.getLsbD i =
  if start.val<=i ∧ i<finish.val then replacement.bv.getLsbD (i-start.val) else value.bv.getLsbD i) ∧
 (∀ writeback, out.2 writeback=writeback)

def store (value replacement : U8) (start finish : Usize) : Result (U8 × (U8 → U8)) := do
 let delta ← 8#usize-finish
 let left ← 255#u8 <<< delta
 let delta2 ← 8#usize-finish
 let right ← left >>> delta2
 let low ← right >>> start
 let win ← low <<< start
 let mask ← lift (~~~win)
 let kept ← lift (value &&& mask)
 let shifted ← replacement <<< start
 let result ← lift (kept ||| shifted)
 ok (result,fun v=>v)

theorem store_spec (value replacement : U8) (start finish : Usize)
 (hs : start.val<8) (he : finish.val<=8) (ho : start.val<finish.val)
 (fit : replacement.val<2^(finish.val-start.val)) :
 store value replacement start finish ⦃ Post value replacement start finish ⦄ := by
 unfold store
 refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=finish) (by scalar_tac)) ?_
 intro delta hd
 have hdv : delta.val=8-finish.val := by scalar_tac
 refine WP.spec_bind (U8.ShiftLeft_spec (255#u8) delta (by scalar_tac)) ?_
 intro left hl
 refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=finish) (by scalar_tac)) ?_
 intro delta2 hd2
 have hdv2 : delta2.val=8-finish.val := by scalar_tac
 refine WP.spec_bind (U8.ShiftRight_spec left delta2 (by scalar_tac)) ?_
 intro right hr
 refine WP.spec_bind (U8.ShiftRight_spec right start (by scalar_tac)) ?_
 intro low hlo
 refine WP.spec_bind (U8.ShiftLeft_spec low start (by scalar_tac)) ?_
 intro win hw
 simp only [lift,bind_ok]
 refine WP.spec_bind (U8.ShiftLeft_spec replacement start (by scalar_tac)) ?_
 intro shifted hshift
 simp only [lift,bind_ok,WP.spec_ok,Post]
 simp only [U8.bv] at hl hr hlo hw hshift ⊢
 have hwindow : win.bv=U8RangeSetterMath.window start.val finish.val := by
  rw [hw.2,hlo.2,hr.2,hl.2,hdv,hdv2]
  unfold U8RangeSetterMath.window
  rfl
 refine ⟨?_,fun writeback=>True.intro⟩
 intro i hi
 simp only [UScalar.bv_or,UScalar.bv_and,UScalar.bv_not]
 rw [hwindow,hshift.2]
 exact U8RangeSetterMath.update_bits value.bv replacement.bv start.val finish.val i he (by omega)
  (by simpa only [UScalar.val] using fit) hi

theorem set_valid (value replacement : U8) (start finish : Usize)
 (hs : start<8#usize) (he : finish<=8#usize) (ho : start<=finish)
 (fit : replacement.val<2^(finish.val-start.val)) :
 original_u8_set_range value start finish replacement ⦃ Post value replacement start finish ⦄ := by
 have hsn : start.val<8 := by scalar_tac
 have hen : finish.val<=8 := by scalar_tac
 by_cases same : start=finish
 · subst finish
   have hr0 : replacement=0#u8 := by
    have hf : replacement.val<1 := by simpa only [Nat.sub_self,Nat.pow_zero] using fit
    apply UScalar.val_eq_imp
    scalar_tac
   subst replacement
   rw [set_empty_zero value start hs]
   simp only [WP.spec_ok]
   change (∀ i, i<8 → value.bv.getLsbD i =
    if start.val<=i ∧ i<start.val then (0#u8).bv.getLsbD (i-start.val) else value.bv.getLsbD i) ∧
    (∀ writeback : U8, writeback=writeback)
   constructor
   · intro i hi
     have hn : ¬(start.val<=i ∧ i<start.val) := by omega
     simp only [if_neg hn]
   · intro writeback; rfl
 · have positive : start.val<finish.val := by scalar_tac
   have hne : (start != finish)=true := by simp [same]
   simp only [original_u8_set_range,bit_length,bind_ok,massert,if_pos hs,if_pos he,if_pos ho,if_neg same]
   refine WP.spec_bind (Usize.sub_spec (x:=finish) (y:=start) (by scalar_tac)) ?_
   intro width hw
   have hwv : width.val=finish.val-start.val := by scalar_tac
   have hwbound : width.val<=8 := by omega
   have hwpos : 0<width.val := by omega
   have hfitwidth : replacement.val<2^width.val := by rw [hwv];exact fit
   refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=width) (by scalar_tac)) ?_
   intro delta hd
   have hdv : delta.val=8-width.val := by scalar_tac
   refine WP.spec_bind (U8.ShiftLeft_spec replacement delta (by scalar_tac)) ?_
   intro left hl
   refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=width) (by scalar_tac)) ?_
   intro delta2 hd2
   have hdv2 : delta2.val=8-width.val := by scalar_tac
   refine WP.spec_bind (U8.ShiftRight_spec left delta2 (by scalar_tac)) ?_
   intro fitted hf
   have heq : fitted=replacement := by
    apply U8.bv_eq_imp_eq
    simp only [U8.bv] at hl hf ⊢
    rw [hf.2,hl.2,hdv,hdv2]
    exact U8RangeSetterMath.fit_low_word replacement.bv width.val hwbound
     (by simpa only [UScalar.val] using hfitwidth)
   rw [heq]
   simp only [massert,ite_true,bind_ok,if_pos hne]
   change store value replacement start finish ⦃ Post value replacement start finish ⦄
   exact store_spec value replacement start finish hsn hen positive fit

#print axioms set_valid
#print axioms store_spec
end U8RangeSetter
