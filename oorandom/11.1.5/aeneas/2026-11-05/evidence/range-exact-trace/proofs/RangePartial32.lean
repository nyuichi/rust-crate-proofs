import Checkpoint
import NegBridge
import PartialLoop
import RangeArithmetic
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace RangePartial32
set_option maxHeartbeats 400000
attribute [local implicit_reducible] UScalar.cMax UScalar.rMax U32.rMax U64.rMax

def Inv (s : U32) (inc : U64) (q : Rand32 × U64 × U32) : Prop :=
 q.1.inc = inc ∧ q.2.1.val < 2^32*s.val

def Post (s : U32) (inc : U64) (q : Rand32 × U64) : Prop :=
 q.1.inc = inc ∧ q.2.val < 2^32*s.val

theorem body_safe_word (s threshold : U32) (inc : U64) (hs : 0 < s.val)
 (self self1 : Rand32) (w : U32) (m : U64) (leftover : U32)
 (valid : Inv s inc (self,m,leftover))
 (word_call : Rand32.rand_u32 self = .ok (w,self1)) (same_inc : self1.inc=self.inc) :
 WP.dspec (Rand32.rand_range_loop.body s threshold self m leftover)
 (fun r => match r with | .done q => Post s inc q | .cont q => Inv s inc q) := by
 apply WP.spec_dspec
 unfold Rand32.rand_range_loop.body
 split
 · rw [word_call]
   simp only [bind_ok,bind_tc_ok,Std.uncurry,lift]
   have exactMul := RangeArithmetic.wrapping_product32 w s hs
   have bound := (RangeArithmetic.product32 w s hs).1
   have hi : self1.inc=inc := same_inc.trans valid.1
   simpa only [WP.spec_ok,Inv,exactMul] using And.intro hi bound
 · simpa only [WP.spec_ok,Post,Inv] using valid

theorem body_safe (s threshold : U32) (inc : U64) (hs : 0 < s.val)
 (self : Rand32) (m : U64) (leftover : U32) (valid : Inv s inc (self,m,leftover)) :
 WP.dspec (Rand32.rand_range_loop.body s threshold self m leftover)
 (fun r => match r with | .done q => Post s inc q | .cont q => Inv s inc q) := by
 exact body_safe_word s threshold inc hs self
   {state:=advance32 self.state self.inc,inc:=self.inc} (output32 self.state)
   m leftover valid (rand_u32_spec self) rfl

theorem loop_safe (s threshold : U32) (inc : U64) (hs : 0 < s.val)
 (self : Rand32) (m : U64) (leftover : U32) (valid : Inv s inc (self,m,leftover)) :
 WP.dspec (Rand32.rand_range_loop self s m leftover threshold) (Post s inc) := by
 unfold Rand32.rand_range_loop
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
end RangePartial32
