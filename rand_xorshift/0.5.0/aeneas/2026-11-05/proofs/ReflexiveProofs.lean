import XorshiftReflexiveOriginal
open Aeneas Aeneas.Std Result xorshift_reflexive_original
namespace ReflexiveProofs
theorem from_exact(value:rand_xorshift.XorShiftRng): original_from value=.ok value := rfl
theorem borrow_exact(value:rand_xorshift.XorShiftRng): original_borrow value=.ok value := rfl
theorem borrow_mut_exact(value:rand_xorshift.XorShiftRng): original_borrow_mut value=.ok (value,fun replacement=>replacement) := rfl
#print axioms from_exact
#print axioms borrow_exact
#print axioms borrow_mut_exact
end ReflexiveProofs
