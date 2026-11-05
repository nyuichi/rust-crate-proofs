import Proofs
open Aeneas Aeneas.Std Result
namespace SupplementalProofs
/-- Actual handwritten Clone method with the official default clone_from model. -/
def debugCloneDictionary (T : Type) : core.clone.Clone (nohash_hasher.NoHashHasher T) :=
  { clone := nohash_hasher.NoHashHasher.Insts.CoreCloneClone.clone }
def releaseCloneDictionary (T : Type) : core.clone.Clone (nohash_hasher_release.NoHashHasher T) :=
  { clone := nohash_hasher_release.NoHashHasher.Insts.CoreCloneClone.clone }
def noStdDebugCloneDictionary (T : Type) : core.clone.Clone (nohash_hasher_no_std.NoHashHasher T) :=
  { clone := nohash_hasher_no_std.NoHashHasher.Insts.CoreCloneClone.clone }
def noStdReleaseCloneDictionary (T : Type) : core.clone.Clone (nohash_hasher_no_std_release.NoHashHasher T) :=
  { clone := nohash_hasher_no_std_release.NoHashHasher.Insts.CoreCloneClone.clone }
/-- Exact replacement of destination by source. This is composition with the
    official default model, not an independent std implementation body proof. -/
theorem clone_from_spec :
    (∀ (T : Type) (destination source : nohash_hasher.NoHashHasher T),
      core.clone.Clone.clone_from.default (debugCloneDictionary T) destination source = .ok source) ∧
    (∀ (T : Type) (destination source : nohash_hasher_release.NoHashHasher T),
      core.clone.Clone.clone_from.default (releaseCloneDictionary T) destination source = .ok source) ∧
    (∀ (T : Type) (destination source : nohash_hasher_no_std.NoHashHasher T),
      core.clone.Clone.clone_from.default (noStdDebugCloneDictionary T) destination source = .ok source) ∧
    (∀ (T : Type) (destination source : nohash_hasher_no_std_release.NoHashHasher T),
      core.clone.Clone.clone_from.default (noStdReleaseCloneDictionary T) destination source = .ok source) := by
  refine ⟨?_, ?_, ?_, ?_⟩ <;> intros <;> rfl
end SupplementalProofs
#print axioms SupplementalProofs.clone_from_spec
