import Zero64
import NegBridge
import PartialLoop
import RangeArithmetic
open Aeneas Aeneas.Std Result
open oorandom_range_original Zero64
namespace RangePartial64
set_option maxHeartbeats 400000
attribute [local implicit_reducible] UScalar.cMax UScalar.rMax U64.rMax U128.rMax

def Inv (s : U64) (inc : U128) (q : Rand64 × U128 × U64) : Prop :=
 q.1.inc = inc ∧ q.2.1.val < 2^64*s.val

def Post (s : U64) (inc : U128) (q : Rand64 × U128) : Prop :=
 q.1.inc = inc ∧ q.2.val < 2^64*s.val

theorem body_safe_word (s threshold : U64) (inc : U128) (hs : 0 < s.val)
 (self self1 : Rand64) (w : U64) (m : U128) (leftover : U64)
 (valid : Inv s inc (self,m,leftover))
 (word_call : Rand64.rand_u64 self = .ok (w,self1)) (same_inc : self1.inc=self.inc) :
 WP.dspec (Rand64.rand_range_loop.body s threshold self m leftover)
 (fun r => match r with | .done q => Post s inc q | .cont q => Inv s inc q) := by
 apply WP.spec_dspec
 unfold Rand64.rand_range_loop.body
 split
 · rw [word_call]
   simp only [bind_ok,bind_tc_ok,Std.uncurry,lift]
   refine WP.spec_bind (RangeArithmetic.wide_product64 w s hs) ?_
   intro product hproduct
   simp only [bind_ok,bind_tc_ok,Std.uncurry,lift,WP.spec_ok,Inv]
   refine ⟨same_inc.trans valid.1,?_⟩
   rw [hproduct]
   exact (RangeArithmetic.product64 w s hs).1
 · simpa only [WP.spec_ok,Post,Inv] using valid

theorem body_safe (s threshold : U64) (inc : U128) (hs : 0 < s.val)
 (self : Rand64) (m : U128) (leftover : U64) (valid : Inv s inc (self,m,leftover)) :
 WP.dspec (Rand64.rand_range_loop.body s threshold self m leftover)
 (fun r => match r with | .done q => Post s inc q | .cont q => Inv s inc q) := by
 exact body_safe_word s threshold inc hs self
   {state:=advance64 self.state self.inc,inc:=self.inc} (output64 self.state)
   m leftover valid (next_spec self) rfl

theorem loop_safe (s threshold : U64) (inc : U128) (hs : 0 < s.val)
 (self : Rand64) (m : U128) (leftover : U64) (valid : Inv s inc (self,m,leftover)) :
 WP.dspec (Rand64.rand_range_loop self s m leftover threshold) (Post s inc) := by
 unfold Rand64.rand_range_loop
 apply PartialLoop.loop_dspec _ (Inv s inc) (Post s inc)
 · intro q hq
   obtain ⟨self,m,leftover⟩ := q
   simp only [Std.uncurry]
   refine WP.dspec_mono (body_safe s threshold inc hs self m leftover hq) ?_
   intro result hresult
   cases result <;> exact hresult
 · exact valid
#print axioms body_safe
#print axioms loop_safe
end RangePartial64
