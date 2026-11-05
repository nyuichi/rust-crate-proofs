import RangePartial32
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace RangeComplete32
set_option maxHeartbeats 400000
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin U32.rMax U64.rMax I32.rMax I32.rMin

def draw (self : Rand32) (s : U32) : Result (Rand32 × U64 × U32) := do
 let (w,self1) ← Rand32.rand_u32 self
 let a ← lift (core.convert.num.FromU64U32.from w)
 let b ← lift (core.convert.num.FromU64U32.from s)
 let m ← a*b
 let mask ← lift (m &&& 4294967295#u64)
 let low ← lift (UScalar.cast .U32 mask)
 ok (self1,m,low)

theorem draw_word_safe (self self1 : Rand32) (w s : U32) (hs : 0<s.val)
 (word_call : Rand32.rand_u32 self = .ok (w,self1)) (same_inc : self1.inc=self.inc) :
 draw self s ⦃ q => RangePartial32.Inv s self.inc q ⦄ := by
 unfold draw
 rw [word_call]
 simp only [bind_ok,bind_tc_ok,Std.uncurry,lift]
 refine WP.spec_bind (RangeArithmetic.wide_product32 w s hs) ?_
 intro product hproduct
 simp only [bind_ok,bind_tc_ok,Std.uncurry,lift,WP.spec_ok,RangePartial32.Inv]
 refine ⟨same_inc,?_⟩
 rw [hproduct]
 exact (RangeArithmetic.product32 w s hs).1

theorem draw_safe (self : Rand32) (s : U32) (hs : 0<s.val) :
 draw self s ⦃ q => RangePartial32.Inv s self.inc q ⦄ := by
 exact draw_word_safe self {state:=advance32 self.state self.inc,inc:=self.inc}
  (output32 self.state) s hs (rand_u32_spec self) rfl

def finish (start : U32) (q : Rand32 × U64) : Result (U32 × Rand32) := do
 let shifted ← q.2 >>> 32#i32
 let high ← lift (UScalar.cast .U32 shifted)
 let out ← high + start
 ok (out,q.1)

theorem finish_safe (start finishBound width : U32) (valid : start.val < finishBound.val)
 (hw : width.val = finishBound.val-start.val) (q : Rand32 × U64)
 (bound : q.2.val < 2^32*width.val) :
 finish start q ⦃ r => start.val ≤ r.1.val ∧ r.1.val < finishBound.val ∧ r.2=q.1 ⦄ := by
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
 exact ⟨by omega,by omega,True.intro⟩

def afterDraw (start width : U32) (q : Rand32 × U64 × U32) : Result (U32 × Rand32) := do
 if q.2.2 < width then
  let neg ← core.num.U32.wrapping_neg width
  let threshold ← neg % width
  let done ← Rand32.rand_range_loop q.1 width q.2.1 q.2.2 threshold
  finish start done
 else finish start (q.1,q.2.1)

theorem afterDraw_safe (start finishBound width : U32) (valid : start.val < finishBound.val)
 (hw : width.val=finishBound.val-start.val) (inc : U64) (q : Rand32 × U64 × U32)
 (hq : RangePartial32.Inv width inc q) :
 WP.dspec (afterDraw start width q)
 (fun r => start.val ≤ r.1.val ∧ r.1.val < finishBound.val ∧ r.2.inc=inc) := by
 have hs : 0<width.val := by omega
 unfold afterDraw
 split
 · simp only [core.num.U32.wrapping_neg,bind_ok]
   refine WP.dspec_bind (WP.spec_dspec _ _ _ (UScalar.rem_spec _ (y:=width) (by omega))) ?_
   intro threshold ht
   refine WP.dspec_bind (RangePartial32.loop_safe width threshold inc hs q.1 q.2.1 q.2.2 hq) ?_
   intro done hd
   refine WP.dspec_mono (WP.spec_dspec _ _ _ (finish_safe start finishBound width valid hw done hd.2)) ?_
   intro out ho
   exact ⟨ho.1,ho.2.1,by rw [ho.2.2];exact hd.1⟩
 · refine WP.dspec_mono (WP.spec_dspec _ _ _ (finish_safe start finishBound width valid hw (q.1,q.2.1) hq.2)) ?_
   intro out ho
   exact ⟨ho.1,ho.2.1,by rw [ho.2.2];exact hq.1⟩

theorem decompose (self : Rand32) (range : core.ops.range.Range U32) :
 Rand32.rand_range self range = (do
  massert (range.start < range.end)
  let width ← range.end-range.start
  let q ← draw self width
  afterDraw range.start width q) := by
 simp only [Rand32.rand_range,draw,afterDraw,finish,Std.bind_assoc,Std.bind_assoc_eq,bind_ok,bind_tc_ok,Std.uncurry]
 all_goals first | rfl | simp only [Std.bind_assoc,bind_ok,bind_tc_ok,Std.uncurry]

theorem public_valid_partial (self : Rand32) (range : core.ops.range.Range U32)
 (valid : range.start < range.end) :
 WP.dspec (Rand32.rand_range self range)
 (fun out => range.start.val ≤ out.1.val ∧ out.1.val < range.end.val ∧ out.2.inc=self.inc) := by
 have hv : range.start.val < range.end.val := valid
 rw [decompose]
 simp only [massert,if_pos valid,bind_ok,bind_tc_ok]
 refine WP.dspec_bind (WP.spec_dspec _ _ _ (U32.sub_spec (x:=range.end) (y:=range.start) (by omega))) ?_
 intro width hw
 have hs : 0<width.val := by omega
 refine WP.dspec_bind (WP.spec_dspec _ _ _ (draw_safe self width hs)) ?_
 intro q hq
 exact afterDraw_safe range.start range.end width hv hw.1 self.inc q hq

#print axioms public_valid_partial
#print axioms afterDraw_safe
#print axioms finish_safe
#print axioms draw_safe
end RangeComplete32
