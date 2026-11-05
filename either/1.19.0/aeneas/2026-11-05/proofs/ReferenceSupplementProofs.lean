import ReferenceSupplement.Either
open Aeneas Aeneas.Std
namespace ReferenceSupplementProofs
open ReferenceSupplementGenerated
theorem shared_copied {L R : Type} (cl : core.marker.Copy L) (cr : core.marker.Copy R) (x : Either L R) : EitherShared0LShared1R.copied cl cr x = .ok x := by cases x <;> rfl
theorem shared_cloned_left {L R : Type} (cl : core.clone.Clone L) (cr : core.clone.Clone R) (x : L) : EitherShared0LShared1R.cloned cl cr (Either.Left x) = (do let y ← cl.clone x; pure (Either.Left y)) := rfl
theorem shared_cloned_right {L R : Type} (cl : core.clone.Clone L) (cr : core.clone.Clone R) (x : R) : EitherShared0LShared1R.cloned cl cr (Either.Right x) = (do let y ← cr.clone x; pure (Either.Right y)) := rfl
theorem mut_copied {L R : Type} (cl : core.marker.Copy L) (cr : core.marker.Copy R) (x : Either L R) : EitherMut0LMut1R.copied cl cr x = .ok (x,x,x) := by cases x <;> rfl
theorem mut_cloned_left {L R : Type} (cl : core.clone.Clone L) (cr : core.clone.Clone R) (x : L) : EitherMut0LMut1R.cloned cl cr (Either.Left x) = (do let y ← cl.clone x; pure (Either.Left y, Either.Left x, Either.Left x)) := rfl
theorem mut_cloned_right {L R : Type} (cl : core.clone.Clone L) (cr : core.clone.Clone R) (x : R) : EitherMut0LMut1R.cloned cl cr (Either.Right x) = (do let y ← cr.clone x; pure (Either.Right y, Either.Right x, Either.Right x)) := rfl
end ReferenceSupplementProofs

#print axioms ReferenceSupplementProofs.shared_copied
#print axioms ReferenceSupplementProofs.shared_cloned_left
#print axioms ReferenceSupplementProofs.shared_cloned_right
#print axioms ReferenceSupplementProofs.mut_copied
#print axioms ReferenceSupplementProofs.mut_cloned_left
#print axioms ReferenceSupplementProofs.mut_cloned_right
