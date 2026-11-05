import PublicDomain
import RangeComplete32
import RangeComplete64
open Aeneas Aeneas.Std Result
open oorandom_range_original
namespace EstablishedRange
theorem rand32_contract :
 (∀ (r : Rand32) (q : core.ops.range.Range U32), q.start < q.end →
  WP.dspec (Rand32.rand_range r q) (fun out => q.start.val ≤ out.1.val ∧ out.1.val < q.end.val ∧ out.2.inc=r.inc)) ∧
 (∀ (r : Rand32) (q : core.ops.range.Range U32), ¬q.start < q.end → Rand32.rand_range r q = .fail .assertionFailure) ∧
 (do let r ← Rand32.from_state (0#u64,0#u64); Rand32.rand_range r ⟨0#u32,3#u32⟩) = .div :=
 ⟨RangeComplete32.public_valid_partial, PublicDomain.range32_invalid, PublicDomain.from_state32_range3_div⟩
theorem rand64_contract :
 (∀ (r : Rand64) (q : core.ops.range.Range U64), q.start < q.end →
  WP.dspec (Rand64.rand_range r q) (fun out => q.start.val ≤ out.1.val ∧ out.1.val < q.end.val ∧ out.2.inc=r.inc)) ∧
 (∀ (r : Rand64) (q : core.ops.range.Range U64), ¬q.start < q.end → Rand64.rand_range r q = .fail .assertionFailure) ∧
 (do let r ← Rand64.from_state (0#u128,0#u128); Rand64.rand_range r ⟨0#u64,3#u64⟩) = .div :=
 ⟨RangeComplete64.public_valid_partial, PublicDomain.range64_invalid, PublicDomain.from_state64_range3_div⟩
#print axioms rand32_contract
#print axioms rand64_contract
end EstablishedRange
