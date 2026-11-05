import Prior66.EndianType
import Prior66.DriverTypes
import ConditionalProofs
namespace Coimport66Types
theorem big_ops_type_same (T : Type) : endian_type.BigEndian T = endian_conditional_original.endian_type.BigEndian T := rfl
theorem little_ops_type_same (T : Type) : endian_type.LittleEndian T = endian_conditional_original.endian_type.LittleEndian T := rfl
theorem big_native_type_same (T : Type) : endian_verification_driver.endian_type.BigEndian T = endian_conditional_original.endian_type.BigEndian T := rfl
theorem little_native_type_same (T : Type) : endian_verification_driver.endian_type.LittleEndian T = endian_conditional_original.endian_type.LittleEndian T := rfl
#print axioms ConditionalProofs.big_into_exact
#print axioms ConditionalProofs.big_try_into_exact
#print axioms ConditionalProofs.big_try_from_exact
#print axioms ConditionalProofs.little_into_exact
#print axioms ConditionalProofs.little_try_into_exact
#print axioms ConditionalProofs.little_try_from_exact
#print axioms big_ops_type_same
#print axioms little_ops_type_same
#print axioms big_native_type_same
#print axioms little_native_type_same
end Coimport66Types
