import Generated.Either
open Aeneas Aeneas.Std either
namespace ResultProofs
theorem from_result_spec {L R : Type} (r : core.result.Result R L) :
 Either.Insts.CoreConvertFromResult.from r = .ok (match r with | .Ok x => Either.Right x | .Err e => Either.Left e) := by
 cases r <;> rfl
theorem from_either_spec {L R : Type} (e : Either L R) :
 either.core.result.Result.Insts.CoreConvertFromEither.from e = .ok (match e with | .Left x => core.result.Result.Err x | .Right y => core.result.Result.Ok y) := by
 cases e <;> rfl
theorem result_roundtrip {L R : Type} (r : core.result.Result R L) :
 (do
   let e ← Either.Insts.CoreConvertFromResult.from r
   either.core.result.Result.Insts.CoreConvertFromEither.from e) = .ok r := by
 cases r <;> simp [Either.Insts.CoreConvertFromResult.from, either.core.result.Result.Insts.CoreConvertFromEither.from]

theorem either_roundtrip {L R : Type} (e : Either L R) :
 (do
   let r ← either.core.result.Result.Insts.CoreConvertFromEither.from e
   Either.Insts.CoreConvertFromResult.from r) = .ok e := by
 cases e <;> simp [Either.Insts.CoreConvertFromResult.from, either.core.result.Result.Insts.CoreConvertFromEither.from]
end ResultProofs

#print axioms ResultProofs.from_result_spec
#print axioms ResultProofs.from_either_spec
#print axioms ResultProofs.result_roundtrip
#print axioms ResultProofs.either_roundtrip
