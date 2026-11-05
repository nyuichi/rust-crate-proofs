import GenericTrace
import RangeComplete32
open Aeneas Aeneas.Std Result
open oorandom_range_original RangeProofs
namespace ActualLoop32
abbrev Candidate := Rand32 × U64 × U32

def next (width : U32) (q : Candidate) : Candidate :=
 let word := output32 q.1.state
 let self1 : Rand32 := ⟨advance32 q.1.state q.1.inc,q.1.inc⟩
 let product := core.num.U64.wrapping_mul (core.convert.num.FromU64U32.from word) (core.convert.num.FromU64U32.from width)
 (self1,product,UScalar.cast .U32 (product &&& 4294967295#u64))

def accepted (threshold : U32) (q : Candidate) : Prop := ¬q.2.2 < threshold

theorem body_equation (width threshold : U32) (q : Candidate) :
 Rand32.rand_range_loop.body width threshold q.1 q.2.1 q.2.2 =
 if q.2.2 < threshold then .ok (.cont (next width q)) else .ok (.done (q.1,q.2.1)) := by
 unfold Rand32.rand_range_loop.body
 split
 · rw [rand_u32_spec]
   simp only [bind_ok,bind_tc_ok,lift,Std.uncurry,next]
 · rfl

theorem finite (width threshold : U32) (q : Candidate) (n : Nat)
 (prior : ∀ k, k<n → ¬ accepted threshold (GenericTrace.trace (next width) q k))
 (accept : accepted threshold (GenericTrace.trace (next width) q n)) :
 Rand32.rand_range_loop q.1 width q.2.1 q.2.2 threshold =
 .ok ((GenericTrace.trace (next width) q n).1,(GenericTrace.trace (next width) q n).2.1) := by
 unfold Rand32.rand_range_loop
 apply GenericTrace.finite (fun q => Rand32.rand_range_loop.body width threshold q.1 q.2.1 q.2.2) (next width) q n
 · intro k hk
   rw [body_equation]
   have h : (GenericTrace.trace (next width) q k).2.2 < threshold := by
    have := prior k hk
    simpa only [accepted,not_not] using this
   rw [if_pos h,GenericTrace.trace_succ]
 · rw [body_equation,if_neg accept]

theorem infinite (width threshold : U32) (q : Candidate) 
 (reject : ∀ k, ¬accepted threshold (GenericTrace.trace (next width) q k)) :
 Rand32.rand_range_loop q.1 width q.2.1 q.2.2 threshold = .div := by
 unfold Rand32.rand_range_loop
 have h := GenericTrace.infinite
  (fun q : Candidate => Rand32.rand_range_loop.body width threshold q.1 q.2.1 q.2.2)
  (fun k => GenericTrace.trace (next width) q k) (by
   intro k
   rw [body_equation]
   have hr : (GenericTrace.trace (next width) q k).2.2 < threshold := by
    simpa only [accepted,not_not] using reject k
   rw [if_pos hr,GenericTrace.trace_succ]) 0
 simpa only [GenericTrace.trace] using h

#print axioms infinite
#print axioms body_equation
#print axioms finite
end ActualLoop32
