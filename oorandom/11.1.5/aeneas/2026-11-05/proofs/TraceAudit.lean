import PublicTrace32
import PublicTrace64
import PublicDomain
open Aeneas Aeneas.Std Result
namespace TraceAudit

theorem first_or_none (p : Nat → Prop) :
 (∃ n, p n ∧ ∀ k, k<n → ¬p k) ∨ (∀ n, ¬p n) := by
 classical
 by_cases exists_accept : ∃ n, p n
 · left
   exact ⟨Nat.find exists_accept,Nat.find_spec exists_accept,fun k hk => Nat.find_min exists_accept hk⟩
 · right
   simpa only [not_exists] using exists_accept

#print axioms first_or_none
#print axioms PublicTrace32.public_first_accept
#print axioms PublicTrace64.public_first_accept
#print axioms PublicTrace32.public_all_rejected
#print axioms PublicTrace64.public_all_rejected
#print axioms PublicDomain.range32_invalid
#print axioms PublicDomain.range64_invalid
end TraceAudit
