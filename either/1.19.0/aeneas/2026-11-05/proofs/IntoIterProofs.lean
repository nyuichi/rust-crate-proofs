import Generated.Either
open Aeneas Aeneas.Std either
namespace IntoIterProofs
-- Original delegation and all six branch corollaries kernel-checked in the shared union.
theorem exact_delegation {L R A LI RI : Type}
 (il : core.iter.traits.collect.IntoIterator L A LI)
 (ir : core.iter.traits.collect.IntoIterator R A RI) (x : Either L R) :
 Either.into_iter il ir x = (match x with
 | .Left l => do let y ← il.into_iter l; Result.ok (Either.Left y)
 | .Right r => do let y ← ir.into_iter r; Result.ok (Either.Right y)) := by
 cases x <;> rfl
theorem left_normal {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : L) (y : LI) (h : il.into_iter x = .ok y) :
 Either.into_iter il ir (.Left x) = .ok (Either.Left y) := by
 rw [exact_delegation]; simp only [h, bind_ok]
theorem left_failure {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : L) (e : Error) (h : il.into_iter x = .fail e) :
 Either.into_iter il ir (Either.Left x) = .fail e := by
 rw [exact_delegation]; simp only [h, bind_fail]
theorem left_divergence {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : L)  (h : il.into_iter x = .div) :
 Either.into_iter il ir (Either.Left x) = .div := by
 rw [exact_delegation]; simp only [h, bind_div]
theorem right_normal {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : R) (y : RI) (h : ir.into_iter x = .ok y) :
 Either.into_iter il ir (.Right x) = .ok (Either.Right y) := by
 rw [exact_delegation]; simp only [h, bind_ok]
theorem right_failure {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : R) (e : Error) (h : ir.into_iter x = .fail e) :
 Either.into_iter il ir (Either.Right x) = .fail e := by
 rw [exact_delegation]; simp only [h, bind_fail]
theorem right_divergence {L R A LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R A RI) (x : R)  (h : ir.into_iter x = .div) :
 Either.into_iter il ir (Either.Right x) = .div := by
 rw [exact_delegation]; simp only [h, bind_div]

#print axioms exact_delegation
#print axioms left_normal
#print axioms left_failure
#print axioms left_divergence
#print axioms right_normal
#print axioms right_failure
#print axioms right_divergence
end IntoIterProofs
