import EndianReflexiveOriginal
open Aeneas Aeneas.Std Result
namespace ReflexiveProofs
open endian_reflexive_original
theorem big_from_exact {T : Type} (value : endian_type.BigEndian T) :
 big_from value = .ok value := by simp [big_from]
theorem big_borrow_exact {T : Type} (value : endian_type.BigEndian T) :
 big_borrow value = .ok value := by simp [big_borrow]
theorem big_borrow_mut_exact {T : Type} (value : endian_type.BigEndian T) :
 big_borrow_mut value = .ok (value,fun replacement => replacement) := by simp [big_borrow_mut]
theorem little_from_exact {T : Type} (value : endian_type.LittleEndian T) :
 little_from value = .ok value := by simp [little_from]
theorem little_borrow_exact {T : Type} (value : endian_type.LittleEndian T) :
 little_borrow value = .ok value := by simp [little_borrow]
theorem little_borrow_mut_exact {T : Type} (value : endian_type.LittleEndian T) :
 little_borrow_mut value = .ok (value,fun replacement => replacement) := by simp [little_borrow_mut]
#print axioms big_from_exact
#print axioms big_borrow_exact
#print axioms big_borrow_mut_exact
#print axioms little_from_exact
#print axioms little_borrow_exact
#print axioms little_borrow_mut_exact
end ReflexiveProofs
