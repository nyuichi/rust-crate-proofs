import Byteorder.Funs
open Aeneas Aeneas.Std
namespace NativeIdentityProofs
theorem le_from_slice_u16_identity (numbers : Slice U16) :
    byteorder.LittleEndian.Insts.ByteorderByteOrder.from_slice_u16 numbers = .ok numbers := by rfl
theorem le_from_slice_u32_identity (numbers : Slice U32) :
    byteorder.LittleEndian.Insts.ByteorderByteOrder.from_slice_u32 numbers = .ok numbers := by rfl
theorem le_from_slice_u64_identity (numbers : Slice U64) :
    byteorder.LittleEndian.Insts.ByteorderByteOrder.from_slice_u64 numbers = .ok numbers := by rfl
theorem le_from_slice_u128_identity (numbers : Slice U128) :
    byteorder.LittleEndian.Insts.ByteorderByteOrder.from_slice_u128 numbers = .ok numbers := by rfl
end NativeIdentityProofs
#print axioms NativeIdentityProofs.le_from_slice_u16_identity
#print axioms NativeIdentityProofs.le_from_slice_u32_identity
#print axioms NativeIdentityProofs.le_from_slice_u64_identity
#print axioms NativeIdentityProofs.le_from_slice_u128_identity
