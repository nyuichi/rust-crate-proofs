import Fill.Funs
open Aeneas Aeneas.Std Result
namespace FillDelegation
open RngFillOriginal
-- Prepared only: not yet kernel checked. The theorem targets the actual external body.
theorem original_exact {R : Type} (d : rand_core.TryRng R Never) (rng : R) (buf : Slice U8) :
 rand_core.Rng.Blanket.fill_bytes d rng buf =
 (do let (_,rng1,buf1) ← d.try_fill_bytes rng buf; pure (rng1,buf1)) := by
 unfold rand_core.Rng.Blanket.fill_bytes
 cases h : d.try_fill_bytes rng buf with
 | fail e => rfl
 | div => rfl
 | ok triple =>
   rcases triple with ⟨result,rng1,buf1⟩
   cases result with
   | Ok u => rfl
   | Err e => cases e
#print axioms original_exact
end FillDelegation
