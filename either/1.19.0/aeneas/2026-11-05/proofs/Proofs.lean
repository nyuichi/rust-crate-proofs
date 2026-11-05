import Generated.Either
open Aeneas Aeneas.Std
namespace Proofs
open either
variable {L R : Type}
theorem is_left_spec (e : Either L R) : e.is_left = .ok (match e with | .Left _ => true | .Right _ => false) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem is_right_spec (e : Either L R) : e.is_right = .ok (match e with | .Left _ => false | .Right _ => true) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem left_spec (e : Either L R) : e.left = .ok (match e with | .Left l => some l | .Right _ => none) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem right_spec (e : Either L R) : e.right = .ok (match e with | .Left _ => none | .Right r => some r) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem flip_spec (e : Either L R) : e.flip = .ok (match e with | .Left l => .Right l | .Right r => .Left r) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem flip_twice (e : Either L R) : (do let e' ← e.flip; e'.flip) = .ok e := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem left_or_spec (e : Either L R) (other : L) : e.left_or other = .ok (match e with | .Left l => l | .Right _ => other) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
theorem right_or_spec (e : Either L R) (other : R) : e.right_or other = .ok (match e with | .Left _ => other | .Right r => r) := by cases e <;> simp [Either.is_left, Either.is_right, Either.left, Either.right, Either.flip, Either.left_or, Either.right_or]
end Proofs
#print axioms Proofs.is_left_spec
#print axioms Proofs.is_right_spec
#print axioms Proofs.left_spec
#print axioms Proofs.right_spec
#print axioms Proofs.flip_spec
#print axioms Proofs.flip_twice
#print axioms Proofs.left_or_spec
#print axioms Proofs.right_or_spec
