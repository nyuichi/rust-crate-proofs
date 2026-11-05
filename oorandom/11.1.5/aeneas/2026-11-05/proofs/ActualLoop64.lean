import GenericTrace
import RangeComplete64
open Aeneas Aeneas.Std Result
open oorandom_range_original Zero64
namespace ActualLoop64
abbrev Candidate := Rand64 × U128 × U64

def next (width : U64) (q : Candidate) : Candidate :=
 let word := output64 q.1.state
 let self1 : Rand64 := ⟨advance64 q.1.state q.1.inc,q.1.inc⟩
 let product := core.num.U128.wrapping_mul (core.convert.num.FromU128U64.from word) (core.convert.num.FromU128U64.from width)
 (self1,product,UScalar.cast .U64 (product &&& 18446744073709551615#u128))

def accepted (threshold : U64) (q : Candidate) : Prop := ¬q.2.2 < threshold

theorem wrapping_product64 (w width : U64) (positive : 0<width.val) :
 (core.num.U128.wrapping_mul (core.convert.num.FromU128U64.from w) (core.convert.num.FromU128U64.from width)).val=w.val*width.val := by
 simp only [core.num.U128.wrapping_mul_val_eq,core.convert.num.FromU128U64.from_val_eq,UScalar.size_def,UScalarTy.U128_numBits_eq]
 exact Nat.mod_eq_of_lt (RangeArithmetic.product64 w width positive).2.1

theorem checked_product (w width : U64) (positive : 0<width.val) :
 (core.convert.num.FromU128U64.from w * core.convert.num.FromU128U64.from width : Result U128) =
 .ok (core.num.U128.wrapping_mul (core.convert.num.FromU128U64.from w) (core.convert.num.FromU128U64.from width)) := by
 apply RangeProofs.spec_eq_ok
 refine WP.spec_mono (RangeArithmetic.wide_product64 w width positive) ?_
 intro m h
 apply UScalar.val_eq_imp
 rw [h,wrapping_product64 _ _ positive]

theorem body_equation (width threshold : U64) (q : Candidate) (positive : 0<width.val) :
 Rand64.rand_range_loop.body width threshold q.1 q.2.1 q.2.2 =
 if q.2.2 < threshold then .ok (.cont (next width q)) else .ok (.done (q.1,q.2.1)) := by
 unfold Rand64.rand_range_loop.body
 split
 · rw [next_spec]
   simp only [bind_ok,bind_tc_ok,lift,Std.uncurry]
   rw [checked_product _ _ positive]
   simp only [bind_ok,lift,Std.uncurry,next]
 · rfl

theorem finite (width threshold : U64) (q : Candidate) (n : Nat) (positive : 0<width.val)
 (prior : ∀ k, k<n → ¬ accepted threshold (GenericTrace.trace (next width) q k))
 (accept : accepted threshold (GenericTrace.trace (next width) q n)) :
 Rand64.rand_range_loop q.1 width q.2.1 q.2.2 threshold =
 .ok ((GenericTrace.trace (next width) q n).1,(GenericTrace.trace (next width) q n).2.1) := by
 unfold Rand64.rand_range_loop
 apply GenericTrace.finite (fun q => Rand64.rand_range_loop.body width threshold q.1 q.2.1 q.2.2) (next width) q n
 · intro k hk
   rw [body_equation _ _ _ positive]
   have h : (GenericTrace.trace (next width) q k).2.2 < threshold := by
    have := prior k hk
    simpa only [accepted,not_not] using this
   rw [if_pos h,GenericTrace.trace_succ]
 · rw [body_equation _ _ _ positive,if_neg accept]

theorem infinite (width threshold : U64) (q : Candidate) (positive : 0<width.val)
 (reject : ∀ k, ¬accepted threshold (GenericTrace.trace (next width) q k)) :
 Rand64.rand_range_loop q.1 width q.2.1 q.2.2 threshold = .div := by
 unfold Rand64.rand_range_loop
 have h := GenericTrace.infinite
  (fun q : Candidate => Rand64.rand_range_loop.body width threshold q.1 q.2.1 q.2.2)
  (fun k => GenericTrace.trace (next width) q k) (by
   intro k
   rw [body_equation _ _ _ positive]
   have hr : (GenericTrace.trace (next width) q k).2.2 < threshold := by
    simpa only [accepted,not_not] using reject k
   rw [if_pos hr,GenericTrace.trace_succ]) 0
 simpa only [GenericTrace.trace] using h

#print axioms infinite
#print axioms body_equation
#print axioms finite
end ActualLoop64
