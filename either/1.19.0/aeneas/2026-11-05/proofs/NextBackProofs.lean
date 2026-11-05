import Generated.Either
open Aeneas Aeneas.Std either
namespace NextBackProofs
theorem left {L R A : Type} (il : core.iter.traits.double_ended.DoubleEndedIterator L A) (ir : core.iter.traits.double_ended.DoubleEndedIterator R A) (x : L) : Either.Insts.CoreIterTraitsDouble_endedDoubleEndedIterator.next_back il ir (Either.Left x) = (do let (o,y) ← il.next_back x; pure (o, Either.Left y)) := rfl
theorem right {L R A : Type} (il : core.iter.traits.double_ended.DoubleEndedIterator L A) (ir : core.iter.traits.double_ended.DoubleEndedIterator R A) (x : R) : Either.Insts.CoreIterTraitsDouble_endedDoubleEndedIterator.next_back il ir (Either.Right x) = (do let (o,y) ← ir.next_back x; pure (o, Either.Right y)) := rfl
theorem next_back_spec {L R A : Type} (il : core.iter.traits.double_ended.DoubleEndedIterator L A) (ir : core.iter.traits.double_ended.DoubleEndedIterator R A) (x : Either L R) :
 Either.Insts.CoreIterTraitsDouble_endedDoubleEndedIterator.next_back il ir x =
 (match x with
 | .Left l => do let (o,y) ← il.next_back l; pure (o, Either.Left y)
 | .Right r => do let (o,y) ← ir.next_back r; pure (o, Either.Right y)) := by cases x <;> rfl
end NextBackProofs

#print axioms NextBackProofs.left
#print axioms NextBackProofs.right
#print axioms NextBackProofs.next_back_spec
