import ByteorderLite.Funs
open Aeneas Aeneas.Std
namespace NativeFloatIdentityProofs
theorem le_from_slice_f32_bits_identity (numbers : Slice F32) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_f32 numbers = .ok numbers := by rfl
theorem le_from_slice_f64_bits_identity (numbers : Slice F64) :
    byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.from_slice_f64 numbers = .ok numbers := by rfl
end NativeFloatIdentityProofs
#print axioms NativeFloatIdentityProofs.le_from_slice_f32_bits_identity
#print axioms NativeFloatIdentityProofs.le_from_slice_f64_bits_identity
