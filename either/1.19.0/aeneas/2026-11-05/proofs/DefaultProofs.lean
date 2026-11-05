import Generated.Either
open Aeneas Aeneas.Std either
namespace DefaultProofs
theorem left_or_default_spec {L R : Type} (inst : core.default.Default L) (e : Either L R) :
 Either.left_or_default inst e = (match e with | .Left l => .ok l | .Right r => inst.default) := by
 cases e <;> rfl

theorem left_or_default_selected {L R : Type} (inst : core.default.Default L) (x : L) :
 Either.left_or_default (L:=L) (R:=R) inst (.Left x) = .ok x := by rfl

theorem left_or_default_normal {L R : Type} (inst : core.default.Default L) (e : Either L R) (d : L)
 (hd : inst.default = .ok d) :
 Either.left_or_default inst e = .ok (match e with | .Left l => l | .Right r => d) := by
 cases e <;> simp [Either.left_or_default, hd]

theorem left_or_default_failure {L R : Type} (inst : core.default.Default L) (x : R) (err : Error)
 (hd : inst.default = .fail err) :
 Either.left_or_default (L:=L) (R:=R) inst (.Right x) = .fail err := by
 simp [Either.left_or_default, hd]

theorem left_or_default_divergence {L R : Type} (inst : core.default.Default L) (x : R)
 (hd : inst.default = .div) :
 Either.left_or_default (L:=L) (R:=R) inst (.Right x) = .div := by
 simp [Either.left_or_default, hd]

theorem right_or_default_spec {L R : Type} (inst : core.default.Default R) (e : Either L R) :
 Either.right_or_default inst e = (match e with | .Left l => inst.default | .Right r => .ok r) := by
 cases e <;> rfl

theorem right_or_default_selected {L R : Type} (inst : core.default.Default R) (x : R) :
 Either.right_or_default (L:=L) (R:=R) inst (.Right x) = .ok x := by rfl

theorem right_or_default_normal {L R : Type} (inst : core.default.Default R) (e : Either L R) (d : R)
 (hd : inst.default = .ok d) :
 Either.right_or_default inst e = .ok (match e with | .Left l => d | .Right r => r) := by
 cases e <;> simp [Either.right_or_default, hd]

theorem right_or_default_failure {L R : Type} (inst : core.default.Default R) (x : L) (err : Error)
 (hd : inst.default = .fail err) :
 Either.right_or_default (L:=L) (R:=R) inst (.Left x) = .fail err := by
 simp [Either.right_or_default, hd]

theorem right_or_default_divergence {L R : Type} (inst : core.default.Default R) (x : L)
 (hd : inst.default = .div) :
 Either.right_or_default (L:=L) (R:=R) inst (.Left x) = .div := by
 simp [Either.right_or_default, hd]

end DefaultProofs
#print axioms DefaultProofs.left_or_default_spec
#print axioms DefaultProofs.left_or_default_selected
#print axioms DefaultProofs.left_or_default_normal
#print axioms DefaultProofs.left_or_default_failure
#print axioms DefaultProofs.left_or_default_divergence
#print axioms DefaultProofs.right_or_default_spec
#print axioms DefaultProofs.right_or_default_selected
#print axioms DefaultProofs.right_or_default_normal
#print axioms DefaultProofs.right_or_default_failure
#print axioms DefaultProofs.right_or_default_divergence
