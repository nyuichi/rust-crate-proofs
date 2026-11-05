import Generated.Either
open Aeneas Aeneas.Std either
namespace FactorProofs
theorem exact_delegation {L R A B LI RI : Type}
 (il : core.iter.traits.collect.IntoIterator L A LI)
 (ir : core.iter.traits.collect.IntoIterator R B RI) (x : Either L R) :
 Either.factor_into_iter il ir x = (match x with
 | .Left l => do let y ← il.into_iter l; Result.ok ({inner:=Either.Left y}:iterator.IterEither LI RI)
 | .Right r => do let y ← ir.into_iter r; Result.ok ({inner:=Either.Right y}:iterator.IterEither LI RI)) := by
 cases x with
 | Left l => cases h : il.into_iter l <;> simp [Either.factor_into_iter,iterator.IterEither.new,h]
 | Right r => cases h : ir.into_iter r <;> simp [Either.factor_into_iter,iterator.IterEither.new,h]
#print axioms exact_delegation
end FactorProofs
