import Generated.Either
open Aeneas Aeneas.Std either
namespace IntoEitherProofs
open either.into_either

theorem into_either_default_spec {S : Type} (x : S) (b : Bool) :
 IntoEither.into_either.default x b = .ok (if b then Either.Left x else Either.Right x) := by
 cases b <;> rfl

theorem into_either_with_default_spec {S F : Type} (inst : IntoEither S)
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (p : S → Bool)
 (hc : callback.call_once f x = .ok (p x))
 (hd : ∀ value branch, inst.into_either value branch = .ok (if branch then Either.Left value else Either.Right value)) :
 IntoEither.into_either_with.default inst callback x f = .ok (if p x then Either.Left x else Either.Right x) := by
 simp [IntoEither.into_either_with.default, hc, hd]

theorem callback_failure {S F : Type} (inst : IntoEither S)
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (err : Error)
 (hc : callback.call_once f x = .fail err) :
 IntoEither.into_either_with.default inst callback x f = .fail err := by
 simp [IntoEither.into_either_with.default, hc]

theorem selected_dispatch {S F : Type} (inst : IntoEither S)
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (b : Bool)
 (hc : callback.call_once f x = .ok b) :
 IntoEither.into_either_with.default inst callback x f = inst.into_either x b := by
 simp [IntoEither.into_either_with.default, hc]

theorem dispatch_failure {S F : Type} (inst : IntoEither S)
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (b : Bool) (err : Error)
 (hc : callback.call_once f x = .ok b) (hd : inst.into_either x b = .fail err) :
 IntoEither.into_either_with.default inst callback x f = .fail err := by
 simp [IntoEither.into_either_with.default, hc, hd]
theorem blanket_dispatch_is_original_default {S : Type} :
 (IntoEither.Blanket S).into_either = IntoEither.into_either.default := by rfl

theorem into_either_blanket_spec {S : Type} (x : S) (b : Bool) :
 (IntoEither.Blanket S).into_either x b = .ok (if b then Either.Left x else Either.Right x) := by
 exact into_either_default_spec x b

theorem into_either_with_blanket_spec {S F : Type}
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (p : S → Bool)
 (hc : callback.call_once f x = .ok (p x)) :
 (IntoEither.Blanket S).into_either_with callback x f = .ok (if p x then Either.Left x else Either.Right x) := by
 exact into_either_with_default_spec (IntoEither.Blanket S) callback x f p hc
   (fun value branch => into_either_default_spec value branch)

theorem blanket_callback_failure {S F : Type}
 (callback : core.ops.function.FnOnce F S Bool) (x : S) (f : F) (err : Error)
 (hc : callback.call_once f x = .fail err) :
 (IntoEither.Blanket S).into_either_with callback x f = .fail err := by
 exact callback_failure (IntoEither.Blanket S) callback x f err hc

end IntoEitherProofs
#print axioms IntoEitherProofs.into_either_default_spec
#print axioms IntoEitherProofs.into_either_with_default_spec
#print axioms IntoEitherProofs.callback_failure
#print axioms IntoEitherProofs.selected_dispatch
#print axioms IntoEitherProofs.dispatch_failure

#print axioms IntoEitherProofs.blanket_dispatch_is_original_default
#print axioms IntoEitherProofs.into_either_blanket_spec
#print axioms IntoEitherProofs.into_either_with_blanket_spec
#print axioms IntoEitherProofs.blanket_callback_failure
