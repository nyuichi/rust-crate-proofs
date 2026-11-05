import CallbackProofs
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
variable {T S : Type}
theorem as_ref_spec (e : Either L R) : e.as_ref = .ok e := by cases e <;> rfl
theorem as_mut_spec (e : Either L R) : e.as_mut = .ok (e, fun updated => match e, updated with | .Left _, .Left l => .Left l | .Right _, .Right r => .Right r | _, _ => e) := by
 cases e <;> simp only [Either.as_mut]
 all_goals
  apply congrArg Result.ok
  apply Prod.ext
  · rfl
  · funext updated; cases updated <;> rfl
theorem left_and_spec (e : Either L R) (other : Either S R) : e.left_and other = .ok (match e with | .Left _ => other | .Right r => .Right r) := by cases e <;> rfl
theorem right_and_spec (e : Either L R) (other : Either L S) : e.right_and other = .ok (match e with | .Left l => .Left l | .Right _ => other) := by cases e <;> rfl
theorem factor_first_spec (e : Either (T × L) (T × R)) : EitherPairPair.factor_first e = .ok (match e with | .Left (t,l) => (t,.Left l) | .Right (t,r) => (t,.Right r)) := by cases e <;> rfl
theorem factor_second_spec (e : Either (L × T) (R × T)) : EitherPairPair.factor_second e = .ok (match e with | .Left (l,t) => (.Left l,t) | .Right (r,t) => (.Right r,t)) := by cases e <;> rfl
theorem factor_none_spec (e : Either (Option L) (Option R)) : EitherOptionOption.factor_none e = .ok (match e with | .Left o => o.map Either.Left | .Right o => o.map Either.Right) := by
 cases e with
 | Left o => cases o <;> simp [EitherOptionOption.factor_none, core.option.Option.map, P.Insts.CoreOpsFunctionFnOnceTupleLEither, P.Insts.CoreOpsFunctionFnOnceTupleREither, P.Insts.CoreOpsFunctionFnOnceTupleLEither.call_once, P.Insts.CoreOpsFunctionFnOnceTupleREither.call_once, Either.Left.constructor, Either.Right.constructor]
 | Right o => cases o <;> simp [EitherOptionOption.factor_none, core.option.Option.map, P.Insts.CoreOpsFunctionFnOnceTupleLEither, P.Insts.CoreOpsFunctionFnOnceTupleREither, P.Insts.CoreOpsFunctionFnOnceTupleLEither.call_once, P.Insts.CoreOpsFunctionFnOnceTupleREither.call_once, Either.Left.constructor, Either.Right.constructor]
end Proofs
#print axioms Proofs.as_ref_spec
#print axioms Proofs.as_mut_spec
#print axioms Proofs.left_and_spec
#print axioms Proofs.right_and_spec
#print axioms Proofs.factor_first_spec
#print axioms Proofs.factor_second_spec
#print axioms Proofs.factor_none_spec

#print axioms Proofs.is_left_spec
#print axioms Proofs.is_right_spec
#print axioms Proofs.left_spec
#print axioms Proofs.right_spec
#print axioms Proofs.flip_spec
#print axioms Proofs.flip_twice
#print axioms Proofs.left_or_spec
#print axioms Proofs.right_or_spec

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
