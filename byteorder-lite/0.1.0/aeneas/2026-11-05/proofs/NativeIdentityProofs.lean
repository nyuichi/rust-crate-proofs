import ByteorderLite.Funs
open Aeneas Aeneas.Std
namespace NativeIdentityProofs
theorem le_from_slice_u16_identity (numbers : Slice U16) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_u16 numbers = .ok numbers := by
  rfl
theorem le_from_slice_u32_identity (numbers : Slice U32) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_u32 numbers = .ok numbers := by
  rfl
theorem le_from_slice_u64_identity (numbers : Slice U64) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_u64 numbers = .ok numbers := by
  rfl
theorem le_from_slice_u128_identity (numbers : Slice U128) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_u128 numbers = .ok numbers := by
  rfl
theorem le_from_slice_i16_identity (numbers : Slice I16) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_i16 numbers = .ok numbers := by
  rfl
theorem le_from_slice_i32_identity (numbers : Slice I32) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_i32 numbers = .ok numbers := by
  rfl
theorem le_from_slice_i64_identity (numbers : Slice I64) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_i64 numbers = .ok numbers := by
  rfl
theorem le_from_slice_i128_identity (numbers : Slice I128) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_i128 numbers = .ok numbers := by
  rfl
end NativeIdentityProofs
#print axioms NativeIdentityProofs.le_from_slice_u16_identity
#print axioms NativeIdentityProofs.le_from_slice_u32_identity
#print axioms NativeIdentityProofs.le_from_slice_u64_identity
#print axioms NativeIdentityProofs.le_from_slice_u128_identity
#print axioms NativeIdentityProofs.le_from_slice_i16_identity
#print axioms NativeIdentityProofs.le_from_slice_i32_identity
#print axioms NativeIdentityProofs.le_from_slice_i64_identity
#print axioms NativeIdentityProofs.le_from_slice_i128_identity
