import Fill.Funs
open Aeneas Aeneas.Std Result
namespace FillDelegation
open RngFillOriginal
-- Prepared only: not yet kernel checked. The theorem targets the actual external body.
theorem original_exact {R : Type} (d : rand_core.TryRng R Never) (rng : R) (buf : Slice U8) :
 rand_core.Rng.Blanket.fill_bytes d rng buf =
 (do let (_,rng1,buf1) ← d.try_fill_bytes rng buf; pure (rng1,buf1)) := by
 unfold rand_core.Rng.Blanket.fill_bytes
 apply congrArg (fun k => d.try_fill_bytes rng buf >>= k)
 funext triple
 rcases triple with ⟨result,rng1,buf1⟩
 cases result with
 | Ok u => rfl
 | Err e => cases e
theorem original_normal {R : Type} (d : rand_core.TryRng R Never) (rng rng1 : R) (buf buf1 : Slice U8)
 (h : d.try_fill_bytes rng buf = .ok (.Ok (),rng1,buf1)) :
 rand_core.Rng.Blanket.fill_bytes d rng buf = .ok (rng1,buf1) := by
 rw [original_exact,h]; rfl

theorem original_failure {R : Type} (d : rand_core.TryRng R Never) (rng : R) (buf : Slice U8) (e : Error)
 (h : d.try_fill_bytes rng buf = .fail e) :
 rand_core.Rng.Blanket.fill_bytes d rng buf = .fail e := by
 rw [original_exact,h]; simp only [bind_fail]

theorem original_divergence {R : Type} (d : rand_core.TryRng R Never) (rng : R) (buf : Slice U8)
 (h : d.try_fill_bytes rng buf = .div) :
 rand_core.Rng.Blanket.fill_bytes d rng buf = .div := by
 rw [original_exact,h]; simp only [bind_div]
#print axioms original_exact
#print axioms original_normal
#print axioms original_failure
#print axioms original_divergence
end FillDelegation
