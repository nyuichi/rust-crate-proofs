import Generated.Either
open Aeneas Aeneas.Std either
namespace BothProofs

theorem either_spec {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (e : Either L R) (f : F) (g : G) :
 Either.either fl fr e f g = (match e with
 | .Left x => fl.call_once f x | .Right x => fr.call_once g x) := by
 cases e <;> rfl

theorem either_normal {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (e : Either L R) (f : F) (g : G) (p : L → T) (q : R → T)
 (h : (match e with
   | .Left x => fl.call_once f x = .ok (p x)
   | .Right x => fr.call_once g x = .ok (q x))) :
 Either.either fl fr e f g = .ok (match e with | .Left x => p x | .Right x => q x) := by
 cases e <;> simpa [Either.either] using h

theorem either_left_failure {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (x : L) (f : F) (g : G) (err : Error) (h : fl.call_once f x = .fail err) :
 Either.either fl fr (.Left x) f g = .fail err := by simpa [Either.either] using h

theorem either_right_failure {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (x : R) (f : F) (g : G) (err : Error) (h : fr.call_once g x = .fail err) :
 Either.either fl fr (.Right x) f g = .fail err := by simpa [Either.either] using h

theorem either_left_divergence {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (x : L) (f : F) (g : G) (h : fl.call_once f x = .div) :
 Either.either fl fr (.Left x) f g = .div := by simpa [Either.either] using h

theorem either_right_divergence {L R F G T : Type}
 (fl : core.ops.function.FnOnce F L T) (fr : core.ops.function.FnOnce G R T)
 (x : R) (f : F) (g : G) (h : fr.call_once g x = .div) :
 Either.either fl fr (.Right x) f g = .div := by simpa [Either.either] using h

theorem either_with_spec {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (e : Either L R) (ctx : Ctx) (f : F) (g : G) :
 Either.either_with fl fr e ctx f g = (match e with
 | .Left x => fl.call_once f (ctx,x) | .Right x => fr.call_once g (ctx,x)) := by
 cases e <;> rfl

theorem either_with_normal {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (e : Either L R) (ctx : Ctx) (f : F) (g : G) (p : Ctx → L → T) (q : Ctx → R → T)
 (h : (match e with
   | .Left x => fl.call_once f (ctx,x) = .ok (p ctx x)
   | .Right x => fr.call_once g (ctx,x) = .ok (q ctx x))) :
 Either.either_with fl fr e ctx f g = .ok (match e with | .Left x => p ctx x | .Right x => q ctx x) := by
 cases e <;> simpa [Either.either_with] using h

theorem either_with_left_failure {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (x : L) (ctx : Ctx) (f : F) (g : G) (err : Error) (h : fl.call_once f (ctx,x) = .fail err) :
 Either.either_with fl fr (.Left x) ctx f g = .fail err := by simpa [Either.either_with] using h

theorem either_with_right_failure {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (x : R) (ctx : Ctx) (f : F) (g : G) (err : Error) (h : fr.call_once g (ctx,x) = .fail err) :
 Either.either_with fl fr (.Right x) ctx f g = .fail err := by simpa [Either.either_with] using h

theorem either_with_left_divergence {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (x : L) (ctx : Ctx) (f : F) (g : G) (h : fl.call_once f (ctx,x) = .div) :
 Either.either_with fl fr (.Left x) ctx f g = .div := by simpa [Either.either_with] using h

theorem either_with_right_divergence {L R Ctx F G T : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) T) (fr : core.ops.function.FnOnce G (Ctx × R) T)
 (x : R) (ctx : Ctx) (f : F) (g : G) (h : fr.call_once g (ctx,x) = .div) :
 Either.either_with fl fr (.Right x) ctx f g = .div := by simpa [Either.either_with] using h


theorem either_into_spec {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (e : Either L R) :
 Either.either_into fl fr e = (match e with | .Left x => fl.into x | .Right x => fr.into x) := by
 cases e <;> rfl

theorem either_into_normal {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (e : Either L R) (p : L → T) (q : R → T)
 (h : (match e with
   | .Left x => fl.into x = .ok (p x)
   | .Right x => fr.into x = .ok (q x))) :
 Either.either_into fl fr e = .ok (match e with | .Left x => p x | .Right x => q x) := by
 cases e <;> simpa [Either.either_into] using h

theorem either_into_left_failure {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (x : L) (err : Error) (h : fl.into x = .fail err) :
 Either.either_into fl fr (.Left x) = .fail err := by simpa [Either.either_into] using h

theorem either_into_right_failure {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (x : R) (err : Error) (h : fr.into x = .fail err) :
 Either.either_into fl fr (.Right x) = .fail err := by simpa [Either.either_into] using h

theorem either_into_left_divergence {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (x : L) (h : fl.into x = .div) :
 Either.either_into fl fr (.Left x) = .div := by simpa [Either.either_into] using h

theorem either_into_right_divergence {L R T : Type}
 (fl : core.convert.Into L T) (fr : core.convert.Into R T) (x : R) (h : fr.into x = .div) :
 Either.either_into fl fr (.Right x) = .div := by simpa [Either.either_into] using h

end BothProofs
#print axioms BothProofs.either_spec
#print axioms BothProofs.either_normal
#print axioms BothProofs.either_left_failure
#print axioms BothProofs.either_right_failure
#print axioms BothProofs.either_left_divergence
#print axioms BothProofs.either_right_divergence
#print axioms BothProofs.either_with_spec
#print axioms BothProofs.either_with_normal
#print axioms BothProofs.either_with_left_failure
#print axioms BothProofs.either_with_right_failure
#print axioms BothProofs.either_with_left_divergence
#print axioms BothProofs.either_with_right_divergence

#print axioms BothProofs.either_into_spec
#print axioms BothProofs.either_into_normal
#print axioms BothProofs.either_into_left_failure
#print axioms BothProofs.either_into_right_failure
#print axioms BothProofs.either_into_left_divergence
#print axioms BothProofs.either_into_right_divergence
