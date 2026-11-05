import Generated.Either
open Aeneas Aeneas.Std either
namespace IteratorProofs

theorem next_spec {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (e : Either L R) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir e =
 (match e with
 | .Left x => do let (o,x') ← il.next x; pure (o, .Left x')
 | .Right x => do let (o,x') ← ir.next x; pure (o, .Right x')) := by
 cases e <;> rfl

theorem next_left_step {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x x' : L) (o : Option A) (h : il.next x = .ok (o,x')) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Left x) = .ok (o, .Left x') := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

theorem next_right_step {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x x' : R) (o : Option A) (h : ir.next x = .ok (o,x')) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Right x) = .ok (o, .Right x') := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

theorem next_left_none_writeback {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x x' : L) (h : il.next x = .ok (.none,x')) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Left x) = .ok (.none, .Left x') := by
 exact next_left_step il ir x x' .none h

theorem next_right_none_writeback {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x x' : R) (h : ir.next x = .ok (.none,x')) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Right x) = .ok (.none, .Right x') := by
 exact next_right_step il ir x x' .none h

theorem next_left_failure {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x : L) (err : Error) (h : il.next x = .fail err) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Left x) = .fail err := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

theorem next_right_failure {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x : R) (err : Error) (h : ir.next x = .fail err) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Right x) = .fail err := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

theorem next_left_divergence {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x : L) (h : il.next x = .div) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Left x) = .div := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

theorem next_right_divergence {L R A : Type}
 (il : core.iter.traits.iterator.Iterator L A) (ir : core.iter.traits.iterator.Iterator R A)
 (x : R) (h : ir.next x = .div) :
 Either.Insts.CoreIterTraitsIteratorIterator.next il ir (.Right x) = .div := by
 simp [Either.Insts.CoreIterTraitsIteratorIterator.next, h]

end IteratorProofs
#print axioms IteratorProofs.next_spec
#print axioms IteratorProofs.next_left_step
#print axioms IteratorProofs.next_right_step
#print axioms IteratorProofs.next_left_none_writeback
#print axioms IteratorProofs.next_right_none_writeback
#print axioms IteratorProofs.next_left_failure
#print axioms IteratorProofs.next_right_failure
#print axioms IteratorProofs.next_left_divergence
#print axioms IteratorProofs.next_right_divergence
