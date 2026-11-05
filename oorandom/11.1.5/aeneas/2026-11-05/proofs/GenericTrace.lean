import Aeneas
open Aeneas Aeneas.Std Result
namespace GenericTrace
unseal Result
attribute [local implicit_reducible] Result

def trace {α : Type} (next : α → α) (x : α) : Nat → α
 | 0 => x
 | n+1 => trace next (next x) n

theorem trace_succ {α : Type} (next : α → α) (x : α) (n : Nat) :
 trace next x (n+1) = next (trace next x n) := by
 induction n generalizing x with
 | zero => rfl
 | succ n ih => exact ih (next x)

theorem finite {α β : Type} (body : α → Result (ControlFlow α β))
 (next : α → α) (x : α) (n : Nat) (out : β)
 (prior : ∀ k, k<n → body (trace next x k) = .ok (.cont (trace next x (k+1))))
 (accept : body (trace next x n) = .ok (.done out)) :
 loop body x = .ok out := by
 induction n generalizing x with
 | zero =>
  unfold loop
  simp only [trace] at accept
  rw [accept]
  simp only [bind_ok]
 | succ n ih =>
  have first := prior 0 (by omega)
  simp only [trace] at first
  unfold loop
  rw [first]
  simp only [bind_ok]
  apply ih (next x)
  · intro k hk
    exact prior (k+1) (by omega)
  · exact accept

theorem infinite {α β : Type} (body : α → Result (ControlFlow α β))
 (states : Nat → α)
 (steps : ∀ n, body (states n) = .ok (.cont (states (n+1)))) :
 ∀ n, loop body (states n) = .div := by
 have bound : ∀ n, Lean.Order.PartialOrder.rel (loop body (states n)) (.div : Result β) := by
  refine loop.fixpoint_induct body (fun f => ∀ n, Lean.Order.PartialOrder.rel (f (states n)) (.div : Result β)) ?_ ?_
  · apply Lean.Order.admissible_pi
    intro n
    exact Lean.Order.admissible_apply (fun _ y => Lean.Order.PartialOrder.rel y (.div : Result β)) (states n)
      (fun _ hc h => Lean.Order.csup_le hc h)
  · intro f ih n
    simpa only [steps n,bind_ok] using ih (n+1)
 intro n
 apply Data.Coinductive.ITree.le_div_is_div
 exact bound n
#print axioms finite
#print axioms infinite
end GenericTrace
