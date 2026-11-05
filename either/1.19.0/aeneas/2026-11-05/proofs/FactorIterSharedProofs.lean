import Generated.Either
open Aeneas Aeneas.Std either
namespace FactorIterSharedProofs
theorem exact_delegation {L R A B LI RI : Type}
 (il : core.iter.traits.collect.IntoIterator L A LI)
 (ir : core.iter.traits.collect.IntoIterator R B RI) (x : Either L R) :
 Either.factor_iter il ir x = (match x with
 | .Left l => do let y ← il.into_iter l; Result.ok ({inner:=Either.Left y}:iterator.IterEither LI RI)
 | .Right r => do let y ← ir.into_iter r; Result.ok ({inner:=Either.Right y}:iterator.IterEither LI RI)) := by
 cases x with
 | Left l => cases h : il.into_iter l <;> simp [Either.factor_iter,iterator.IterEither.new,h]
 | Right r => cases h : ir.into_iter r <;> simp [Either.factor_iter,iterator.IterEither.new,h]
theorem left_normal {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : L) (y : LI) (h : il.into_iter x = .ok y) :
 Either.factor_iter il ir (Either.Left x) = .ok ({inner:=Either.Left y} : iterator.IterEither LI RI) := by
 rw [exact_delegation]; simp only [h, bind_ok]
theorem left_failure {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : L) (e : Error) (h : il.into_iter x = .fail e) :
 Either.factor_iter il ir (Either.Left x) = .fail e := by
 rw [exact_delegation]; simp only [h, bind_fail]
theorem left_divergence {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : L)  (h : il.into_iter x = .div) :
 Either.factor_iter il ir (Either.Left x) = .div := by
 rw [exact_delegation]; simp only [h, bind_div]
theorem right_normal {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : R) (y : RI) (h : ir.into_iter x = .ok y) :
 Either.factor_iter il ir (Either.Right x) = .ok ({inner:=Either.Right y} : iterator.IterEither LI RI) := by
 rw [exact_delegation]; simp only [h, bind_ok]
theorem right_failure {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : R) (e : Error) (h : ir.into_iter x = .fail e) :
 Either.factor_iter il ir (Either.Right x) = .fail e := by
 rw [exact_delegation]; simp only [h, bind_fail]
theorem right_divergence {L R A B LI RI : Type} (il : core.iter.traits.collect.IntoIterator L A LI) (ir : core.iter.traits.collect.IntoIterator R B RI) (x : R)  (h : ir.into_iter x = .div) :
 Either.factor_iter il ir (Either.Right x) = .div := by
 rw [exact_delegation]; simp only [h, bind_div]
#print axioms exact_delegation
#print axioms left_normal
#print axioms left_failure
#print axioms left_divergence
#print axioms right_normal
#print axioms right_failure
#print axioms right_divergence
end FactorIterSharedProofs
