import Generated.Either
open Aeneas Aeneas.Std either
set_option linter.unusedSimpArgs false
namespace CallbackProofs
theorem is_left_and_spec {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (e : Either L R) (f : F) (p : L → Bool)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.is_left_and inst e f = .ok (match e with | .Left l => p l | .Right r => false) := by
 cases e <;> simp [Either.is_left_and, h]

theorem is_left_and_failure {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (x : L) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.is_left_and (L:=L) (R:=R) inst (.Left x) f = .fail err := by
 simp [Either.is_left_and, h]

theorem is_left_and_short_circuit {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (x : R) (f : F) :
 Either.is_left_and (L:=L) (R:=R) inst (.Right x) f = .ok false := by rfl

theorem is_right_and_spec {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (e : Either L R) (f : F) (p : R → Bool)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.is_right_and inst e f = .ok (match e with | .Left l => false | .Right r => p r) := by
 cases e <;> simp [Either.is_right_and, h]

theorem is_right_and_failure {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (x : R) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.is_right_and (L:=L) (R:=R) inst (.Right x) f = .fail err := by
 simp [Either.is_right_and, h]

theorem is_right_and_short_circuit {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (x : L) (f : F) :
 Either.is_right_and (L:=L) (R:=R) inst (.Left x) f = .ok false := by rfl

theorem is_left_or_spec {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (e : Either L R) (f : F) (p : R → Bool)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.is_left_or inst e f = .ok (match e with | .Left l => true | .Right r => p r) := by
 cases e <;> simp [Either.is_left_or, h]

theorem is_left_or_failure {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (x : R) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.is_left_or (L:=L) (R:=R) inst (.Right x) f = .fail err := by
 simp [Either.is_left_or, h]

theorem is_left_or_short_circuit {L R F : Type} (inst : core.ops.function.FnOnce F R Bool)
 (x : L) (f : F) :
 Either.is_left_or (L:=L) (R:=R) inst (.Left x) f = .ok true := by rfl

theorem is_right_or_spec {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (e : Either L R) (f : F) (p : L → Bool)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.is_right_or inst e f = .ok (match e with | .Left l => p l | .Right r => true) := by
 cases e <;> simp [Either.is_right_or, h]

theorem is_right_or_failure {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (x : L) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.is_right_or (L:=L) (R:=R) inst (.Left x) f = .fail err := by
 simp [Either.is_right_or, h]

theorem is_right_or_short_circuit {L R F : Type} (inst : core.ops.function.FnOnce F L Bool)
 (x : R) (f : F) :
 Either.is_right_or (L:=L) (R:=R) inst (.Right x) f = .ok true := by rfl

theorem map_left_spec {L R F M : Type} (inst : core.ops.function.FnOnce F L M)
 (e : Either L R) (f : F) (p : L → M)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.map_left inst e f = .ok (match e with | .Left l => .Left (p l) | .Right r => .Right r) := by
 cases e <;> simp [Either.map_left, h]

theorem map_left_failure {L R F M : Type} (inst : core.ops.function.FnOnce F L M)
 (x : L) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.map_left (L:=L) (R:=R) inst (.Left x) f = .fail err := by
 simp [Either.map_left, h]

theorem map_left_short_circuit {L R F M : Type} (inst : core.ops.function.FnOnce F L M)
 (x : R) (f : F) :
 Either.map_left (L:=L) (R:=R) inst (.Right x) f = .ok (.Right x) := by rfl

theorem map_right_spec {L R F M : Type} (inst : core.ops.function.FnOnce F R M)
 (e : Either L R) (f : F) (p : R → M)
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.map_right inst e f = .ok (match e with | .Left l => .Left l | .Right r => .Right (p r)) := by
 cases e <;> simp [Either.map_right, h]

theorem map_right_failure {L R F M : Type} (inst : core.ops.function.FnOnce F R M)
 (x : R) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.map_right (L:=L) (R:=R) inst (.Right x) f = .fail err := by
 simp [Either.map_right, h]

theorem map_right_short_circuit {L R F M : Type} (inst : core.ops.function.FnOnce F R M)
 (x : L) (f : F) :
 Either.map_right (L:=L) (R:=R) inst (.Left x) f = .ok (.Left x) := by rfl

theorem map_left_or_spec {L R F S : Type} (inst : core.ops.function.FnOnce F L (S))
 (e : Either L R) (d : S) (f : F) (p : L → (S))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.map_left_or inst e d f = .ok (match e with | .Left l => p l | .Right r => d) := by
 cases e <;> simp [Either.map_left_or, h]

theorem map_left_or_failure {L R F S : Type} (inst : core.ops.function.FnOnce F L (S))
 (x : L) (d : S) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.map_left_or (L:=L) (R:=R) inst (.Left x) d f = .fail err := by
 simp [Either.map_left_or, h]

theorem map_left_or_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F L (S))
 (x : R) (d : S) (f : F) :
 Either.map_left_or (L:=L) (R:=R) inst (.Right x) d f = .ok (d) := by rfl

theorem map_right_or_spec {L R F S : Type} (inst : core.ops.function.FnOnce F R (S))
 (e : Either L R) (d : S) (f : F) (p : R → (S))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.map_right_or inst e d f = .ok (match e with | .Left l => d | .Right r => p r) := by
 cases e <;> simp [Either.map_right_or, h]

theorem map_right_or_failure {L R F S : Type} (inst : core.ops.function.FnOnce F R (S))
 (x : R) (d : S) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.map_right_or (L:=L) (R:=R) inst (.Right x) d f = .fail err := by
 simp [Either.map_right_or, h]

theorem map_right_or_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F R (S))
 (x : L) (d : S) (f : F) :
 Either.map_right_or (L:=L) (R:=R) inst (.Left x) d f = .ok (d) := by rfl

theorem left_or_else_spec {L R F S : Type} (inst : core.ops.function.FnOnce F R (L))
 (e : Either L R) (f : F) (p : R → (L))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.left_or_else inst e f = .ok (match e with | .Left l => l | .Right r => p r) := by
 cases e <;> simp [Either.left_or_else, h]

theorem left_or_else_failure {L R F S : Type} (inst : core.ops.function.FnOnce F R (L))
 (x : R) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.left_or_else (L:=L) (R:=R) inst (.Right x) f = .fail err := by
 simp [Either.left_or_else, h]

theorem left_or_else_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F R (L))
 (x : L) (f : F) :
 Either.left_or_else (L:=L) (R:=R) inst (.Left x) f = .ok (x) := by rfl

theorem right_or_else_spec {L R F S : Type} (inst : core.ops.function.FnOnce F L (R))
 (e : Either L R) (f : F) (p : L → (R))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.right_or_else inst e f = .ok (match e with | .Left l => p l | .Right r => r) := by
 cases e <;> simp [Either.right_or_else, h]

theorem right_or_else_failure {L R F S : Type} (inst : core.ops.function.FnOnce F L (R))
 (x : L) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.right_or_else (L:=L) (R:=R) inst (.Left x) f = .fail err := by
 simp [Either.right_or_else, h]

theorem right_or_else_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F L (R))
 (x : R) (f : F) :
 Either.right_or_else (L:=L) (R:=R) inst (.Right x) f = .ok (x) := by rfl

theorem left_and_then_spec {L R F S : Type} (inst : core.ops.function.FnOnce F L (Either S R))
 (e : Either L R) (f : F) (p : L → (Either S R))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.left_and_then inst e f = .ok (match e with | .Left l => p l | .Right r => .Right r) := by
 cases e <;> simp [Either.left_and_then, h]

theorem left_and_then_failure {L R F S : Type} (inst : core.ops.function.FnOnce F L (Either S R))
 (x : L) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.left_and_then (L:=L) (R:=R) inst (.Left x) f = .fail err := by
 simp [Either.left_and_then, h]

theorem left_and_then_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F L (Either S R))
 (x : R) (f : F) :
 Either.left_and_then (L:=L) (R:=R) inst (.Right x) f = .ok (.Right x) := by rfl

theorem right_and_then_spec {L R F S : Type} (inst : core.ops.function.FnOnce F R (Either L S))
 (e : Either L R) (f : F) (p : R → (Either L S))
 (h : ∀ x, inst.call_once f x = .ok (p x)) :
 Either.right_and_then inst e f = .ok (match e with | .Left l => .Left l | .Right r => p r) := by
 cases e <;> simp [Either.right_and_then, h]

theorem right_and_then_failure {L R F S : Type} (inst : core.ops.function.FnOnce F R (Either L S))
 (x : R) (f : F) (err : Error) (h : inst.call_once f x = .fail err) :
 Either.right_and_then (L:=L) (R:=R) inst (.Right x) f = .fail err := by
 simp [Either.right_and_then, h]

theorem right_and_then_short_circuit {L R F S : Type} (inst : core.ops.function.FnOnce F R (Either L S))
 (x : L) (f : F) :
 Either.right_and_then (L:=L) (R:=R) inst (.Left x) f = .ok (.Left x) := by rfl

end CallbackProofs
#print axioms CallbackProofs.is_left_and_spec
#print axioms CallbackProofs.is_left_and_failure
#print axioms CallbackProofs.is_left_and_short_circuit
#print axioms CallbackProofs.is_right_and_spec
#print axioms CallbackProofs.is_right_and_failure
#print axioms CallbackProofs.is_right_and_short_circuit
#print axioms CallbackProofs.is_left_or_spec
#print axioms CallbackProofs.is_left_or_failure
#print axioms CallbackProofs.is_left_or_short_circuit
#print axioms CallbackProofs.is_right_or_spec
#print axioms CallbackProofs.is_right_or_failure
#print axioms CallbackProofs.is_right_or_short_circuit
#print axioms CallbackProofs.map_left_spec
#print axioms CallbackProofs.map_left_failure
#print axioms CallbackProofs.map_left_short_circuit
#print axioms CallbackProofs.map_right_spec
#print axioms CallbackProofs.map_right_failure
#print axioms CallbackProofs.map_right_short_circuit

#print axioms CallbackProofs.map_left_or_spec
#print axioms CallbackProofs.map_left_or_failure
#print axioms CallbackProofs.map_left_or_short_circuit
#print axioms CallbackProofs.map_right_or_spec
#print axioms CallbackProofs.map_right_or_failure
#print axioms CallbackProofs.map_right_or_short_circuit
#print axioms CallbackProofs.left_or_else_spec
#print axioms CallbackProofs.left_or_else_failure
#print axioms CallbackProofs.left_or_else_short_circuit
#print axioms CallbackProofs.right_or_else_spec
#print axioms CallbackProofs.right_or_else_failure
#print axioms CallbackProofs.right_or_else_short_circuit
#print axioms CallbackProofs.left_and_then_spec
#print axioms CallbackProofs.left_and_then_failure
#print axioms CallbackProofs.left_and_then_short_circuit
#print axioms CallbackProofs.right_and_then_spec
#print axioms CallbackProofs.right_and_then_failure
#print axioms CallbackProofs.right_and_then_short_circuit
