import Oorandom.Funs
open Aeneas Aeneas.Std Result
namespace PartialLoop
unseal Result
attribute [local implicit_reducible] Result
theorem loop_dspec {α β : Type} (body : α → Result (ControlFlow α β))
 (inv : α → Prop) (post : β → Prop)
 (body_safe : ∀ x, inv x → WP.dspec (body x) (fun r => match r with | .done y => post y | .cont x' => inv x'))
 (x : α) (valid : inv x) : WP.dspec (loop body x) post := by
 refine loop.fixpoint_induct body (fun f => ∀ x, inv x → WP.dspec (f x) post) ?_ ?_ x valid
 · apply Lean.Order.admissible_pi
   intro x
   apply Lean.Order.admissible_pi
   intro h
   exact WP.dspec_func_admissible x post
 · intro f ih y hy
   apply WP.dspec_bind (body_safe y hy)
   intro result safe
   cases result with
   | done v => simpa only [WP.dspec_ok] using safe
   | cont next => exact ih next safe
#print axioms loop_dspec
end PartialLoop
