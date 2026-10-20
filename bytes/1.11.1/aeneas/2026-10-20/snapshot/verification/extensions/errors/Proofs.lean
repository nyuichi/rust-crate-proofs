import extensions.errors.Generated
open Aeneas Aeneas.Std Result
namespace BytesProofs.Errors

theorem try_get_error_equality_compares_both_counts
    (a b : bytes.TryGetError) :
    bytes.TryGetError.Insts.CoreCmpPartialEqTryGetError.eq a b =
      ok (decide (a.requested = b.requested ∧ a.available = b.available)) := by
  by_cases h : a.requested = b.requested <;>
    simp [bytes.TryGetError.Insts.CoreCmpPartialEqTryGetError.eq, h]

theorem try_get_error_equal_iff_same_record (a b : bytes.TryGetError) :
    bytes.TryGetError.Insts.CoreCmpPartialEqTryGetError.eq a b = ok true ↔ a = b := by
  rw [try_get_error_equality_compares_both_counts]
  cases a
  cases b
  simp

theorem advance_panic_reports_panic (error : bytes.TryGetError) :
    bytes.panic_advance error = fail .panic := by
  unfold bytes.panic_advance
  change (do
    let _ ← (ok () : Result Unit)
    let _ ← (ok () : Result Unit)
    let _ ← (ok () : Result Unit)
    fail .panic) = (fail .panic : Result Never)
  simp

theorem oversized_integer_reports_panic (size nbytes : Std.Usize) :
    bytes.panic_does_not_fit size nbytes = fail .panic := by
  unfold bytes.panic_does_not_fit
  change (do
    let _ ← (ok () : Result Unit)
    let _ ← (ok () : Result Unit)
    let _ ← (ok () : Result Unit)
    fail .panic) = (fail .panic : Result Never)
  simp

end BytesProofs.Errors
#print axioms BytesProofs.Errors.try_get_error_equality_compares_both_counts
#print axioms BytesProofs.Errors.try_get_error_equal_iff_same_record
#print axioms BytesProofs.Errors.advance_panic_reports_panic
#print axioms BytesProofs.Errors.oversized_integer_reports_panic
