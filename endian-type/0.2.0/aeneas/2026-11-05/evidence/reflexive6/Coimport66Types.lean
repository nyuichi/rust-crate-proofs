import Prior66.EndianType
import Prior66.DriverTypes
import ReflexiveProofs
namespace Coimport66Types
theorem big_ops_type_same (T : Type) : endian_type.BigEndian T = endian_reflexive_original.endian_type.BigEndian T := rfl
theorem little_ops_type_same (T : Type) : endian_type.LittleEndian T = endian_reflexive_original.endian_type.LittleEndian T := rfl
theorem big_native_type_same (T : Type) : endian_verification_driver.endian_type.BigEndian T = endian_reflexive_original.endian_type.BigEndian T := rfl
theorem little_native_type_same (T : Type) : endian_verification_driver.endian_type.LittleEndian T = endian_reflexive_original.endian_type.LittleEndian T := rfl
#print axioms ReflexiveProofs.big_from_exact
#print axioms ReflexiveProofs.big_borrow_exact
#print axioms ReflexiveProofs.big_borrow_mut_exact
#print axioms ReflexiveProofs.little_from_exact
#print axioms ReflexiveProofs.little_borrow_exact
#print axioms ReflexiveProofs.little_borrow_mut_exact
#print axioms big_ops_type_same
#print axioms little_ops_type_same
#print axioms big_native_type_same
#print axioms little_native_type_same
end Coimport66Types
