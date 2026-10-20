import extensions.ownership.Generated
open Aeneas Aeneas.Std Result
namespace BytesProofs.Ownership

theorem static_storage_reports_not_unique
    (refcount : bytes.core.sync.atomic.Atomic (MutRawPtr Unit)
      (bytes.core.sync.atomic.private.Align8 (MutRawPtr Unit))) :
    bytes.bytes.static_is_unique refcount = ok false := by rfl

theorem shared_uniqueness_tests_acquire_observation
    (self : bytes.bytes_mut.Shared) (observed : Std.Usize)
    (hload : bytes.core.sync.atomic.AtomicUsizeAlign8Usize.load self.ref_count
      bytes.core.sync.atomic.Ordering.Acquire = ok observed) :
    bytes.bytes_mut.Shared.is_unique self = ok (decide (observed.val = 1)) := by
  simp [bytes.bytes_mut.Shared.is_unique, hload]
  congr 1
  scalar_tac

theorem shared_unique_iff_observed_count_one
    (self : bytes.bytes_mut.Shared) (observed : Std.Usize)
    (hload : bytes.core.sync.atomic.AtomicUsizeAlign8Usize.load self.ref_count
      bytes.core.sync.atomic.Ordering.Acquire = ok observed) :
    bytes.bytes_mut.Shared.is_unique self = ok true ↔ observed.val = 1 := by
  rw [shared_uniqueness_tests_acquire_observation self observed hload]
  simp

end BytesProofs.Ownership
#print axioms BytesProofs.Ownership.static_storage_reports_not_unique
#print axioms BytesProofs.Ownership.shared_uniqueness_tests_acquire_observation
#print axioms BytesProofs.Ownership.shared_unique_iff_observed_count_one

#print axioms BytesProofs.Ownership.shared_unique_iff_observed_count_one
#print axioms BytesProofs.Ownership.shared_uniqueness_tests_acquire_observation
#print axioms BytesProofs.Ownership.static_storage_reports_not_unique
