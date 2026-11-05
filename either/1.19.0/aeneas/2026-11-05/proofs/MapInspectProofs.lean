import Generated.Either
open Aeneas Aeneas.Std either
namespace MapInspectProofs

theorem map_either_spec {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (e : Either L R) (f : F) (g : G) :
 Either.map_either fl fr e f g = (match e with | .Left x => (do let y ← fl.call_once f x; pure (.Left y)) | .Right x => (do let y ← fr.call_once g x; pure (.Right y))) := by
 cases e <;> rfl

theorem map_either_normal {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (e : Either L R) (f : F) (g : G) (p : L → M) (q : R → S)
 (h : (match e with
  | .Left x => fl.call_once f x = .ok (p x)
  | .Right x => fr.call_once g x = .ok (q x))) :
 Either.map_either fl fr e f g = .ok (match e with | .Left x => .Left (p x) | .Right x => .Right (q x)) := by
 cases e <;> simp_all [Either.map_either]

theorem map_either_left_failure {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (x : L) (f : F) (g : G) (err : Error) (h : fl.call_once f x = .fail err) :
 Either.map_either fl fr (.Left x) f g = .fail err := by
 simp [Either.map_either, h]

theorem map_either_left_divergence {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (x : L) (f : F) (g : G) (h : fl.call_once f x = .div) :
 Either.map_either fl fr (.Left x) f g = .div := by
 simp [Either.map_either, h]

theorem map_either_right_failure {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (x : R) (f : F) (g : G) (err : Error) (h : fr.call_once g x = .fail err) :
 Either.map_either fl fr (.Right x) f g = .fail err := by
 simp [Either.map_either, h]

theorem map_either_right_divergence {L R F G M S : Type}
 (fl : core.ops.function.FnOnce F L M) (fr : core.ops.function.FnOnce G R S)
 (x : R) (f : F) (g : G) (h : fr.call_once g x = .div) :
 Either.map_either fl fr (.Right x) f g = .div := by
 simp [Either.map_either, h]

theorem map_either_with_spec {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (e : Either L R) (ctx : Ctx) (f : F) (g : G) :
 Either.map_either_with fl fr e ctx f g = (match e with | .Left x => (do let y ← fl.call_once f (ctx,x); pure (.Left y)) | .Right x => (do let y ← fr.call_once g (ctx,x); pure (.Right y))) := by
 cases e <;> rfl

theorem map_either_with_normal {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (e : Either L R) (ctx : Ctx) (f : F) (g : G) (p : Ctx → L → M) (q : Ctx → R → S)
 (h : (match e with
  | .Left x => fl.call_once f (ctx,x) = .ok (p ctx x)
  | .Right x => fr.call_once g (ctx,x) = .ok (q ctx x))) :
 Either.map_either_with fl fr e ctx f g = .ok (match e with | .Left x => .Left (p ctx x) | .Right x => .Right (q ctx x)) := by
 cases e <;> simp_all [Either.map_either_with]

theorem map_either_with_left_failure {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (x : L) (ctx : Ctx) (f : F) (g : G) (err : Error) (h : fl.call_once f (ctx,x) = .fail err) :
 Either.map_either_with fl fr (.Left x) ctx f g = .fail err := by
 simp [Either.map_either_with, h]

theorem map_either_with_left_divergence {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (x : L) (ctx : Ctx) (f : F) (g : G) (h : fl.call_once f (ctx,x) = .div) :
 Either.map_either_with fl fr (.Left x) ctx f g = .div := by
 simp [Either.map_either_with, h]

theorem map_either_with_right_failure {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (x : R) (ctx : Ctx) (f : F) (g : G) (err : Error) (h : fr.call_once g (ctx,x) = .fail err) :
 Either.map_either_with fl fr (.Right x) ctx f g = .fail err := by
 simp [Either.map_either_with, h]

theorem map_either_with_right_divergence {L R Ctx F G M S : Type}
 (fl : core.ops.function.FnOnce F (Ctx × L) M) (fr : core.ops.function.FnOnce G (Ctx × R) S)
 (x : R) (ctx : Ctx) (f : F) (g : G) (h : fr.call_once g (ctx,x) = .div) :
 Either.map_either_with fl fr (.Right x) ctx f g = .div := by
 simp [Either.map_either_with, h]

theorem inspect_left_spec {L R F : Type}
 (fl : core.ops.function.FnOnce F L Unit)
 (e : Either L R) (f : F) :
 Either.inspect_left fl e f = (match e with
 | .Left x => do fl.call_once f x; pure e
 | .Right _ => .ok e) := by
 cases e <;> rfl

theorem inspect_left_normal {L R F : Type}
 (fl : core.ops.function.FnOnce F L Unit)
 (e : Either L R) (f : F)
 (h : (match e with | .Left x => fl.call_once f x = .ok () | .Right _ => True)) :
 Either.inspect_left fl e f = .ok e := by
 cases e <;> simp_all [Either.inspect_left]

theorem inspect_left_failure {L R F : Type}
 (fl : core.ops.function.FnOnce F L Unit)
 (x : L) (f : F) (err : Error) (h : fl.call_once f x = .fail err) :
 Either.inspect_left (R:=R) (L:=L) fl (.Left x) f = .fail err := by
 simp [Either.inspect_left, h]

theorem inspect_left_divergence {L R F : Type}
 (fl : core.ops.function.FnOnce F L Unit)
 (x : L) (f : F) (h : fl.call_once f x = .div) :
 Either.inspect_left (R:=R) (L:=L) fl (.Left x) f = .div := by
 simp [Either.inspect_left, h]

theorem inspect_left_short_circuit {L R F : Type}
 (fl : core.ops.function.FnOnce F L Unit)
 (x : R) (f : F) :
 Either.inspect_left (R:=R) (L:=L) fl (.Right x) f = .ok (.Right x) := by rfl

theorem inspect_right_spec {L R F : Type}
 (fl : core.ops.function.FnOnce F R Unit)
 (e : Either L R) (f : F) :
 Either.inspect_right fl e f = (match e with
 | .Right x => do fl.call_once f x; pure e
 | .Left _ => .ok e) := by
 cases e <;> rfl

theorem inspect_right_normal {L R F : Type}
 (fl : core.ops.function.FnOnce F R Unit)
 (e : Either L R) (f : F)
 (h : (match e with | .Right x => fl.call_once f x = .ok () | .Left _ => True)) :
 Either.inspect_right fl e f = .ok e := by
 cases e <;> simp_all [Either.inspect_right]

theorem inspect_right_failure {L R F : Type}
 (fl : core.ops.function.FnOnce F R Unit)
 (x : R) (f : F) (err : Error) (h : fl.call_once f x = .fail err) :
 Either.inspect_right (R:=R) (L:=L) fl (.Right x) f = .fail err := by
 simp [Either.inspect_right, h]

theorem inspect_right_divergence {L R F : Type}
 (fl : core.ops.function.FnOnce F R Unit)
 (x : R) (f : F) (h : fl.call_once f x = .div) :
 Either.inspect_right (R:=R) (L:=L) fl (.Right x) f = .div := by
 simp [Either.inspect_right, h]

theorem inspect_right_short_circuit {L R F : Type}
 (fl : core.ops.function.FnOnce F R Unit)
 (x : L) (f : F) :
 Either.inspect_right (R:=R) (L:=L) fl (.Left x) f = .ok (.Left x) := by rfl

end MapInspectProofs

#print axioms MapInspectProofs.map_either_spec

#print axioms MapInspectProofs.map_either_normal

#print axioms MapInspectProofs.map_either_left_failure

#print axioms MapInspectProofs.map_either_left_divergence

#print axioms MapInspectProofs.map_either_right_failure

#print axioms MapInspectProofs.map_either_right_divergence

#print axioms MapInspectProofs.map_either_with_spec

#print axioms MapInspectProofs.map_either_with_normal

#print axioms MapInspectProofs.map_either_with_left_failure

#print axioms MapInspectProofs.map_either_with_left_divergence

#print axioms MapInspectProofs.map_either_with_right_failure

#print axioms MapInspectProofs.map_either_with_right_divergence

#print axioms MapInspectProofs.inspect_left_spec

#print axioms MapInspectProofs.inspect_left_normal

#print axioms MapInspectProofs.inspect_left_failure

#print axioms MapInspectProofs.inspect_left_divergence

#print axioms MapInspectProofs.inspect_left_short_circuit

#print axioms MapInspectProofs.inspect_right_spec

#print axioms MapInspectProofs.inspect_right_normal

#print axioms MapInspectProofs.inspect_right_failure

#print axioms MapInspectProofs.inspect_right_divergence

#print axioms MapInspectProofs.inspect_right_short_circuit
