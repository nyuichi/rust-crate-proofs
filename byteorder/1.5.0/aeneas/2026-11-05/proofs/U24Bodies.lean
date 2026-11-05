import ReadVariableProofs
import ByteorderU24Roots.Funs
open Aeneas Aeneas.Std
namespace U24Proofs
theorem be_read_uint_body_eq (buf : Slice U8) (count : Usize) :
    byteorder_u24_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf count =
      byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf count := by rfl
theorem le_read_uint_body_eq (buf : Slice U8) (count : Usize) :
    byteorder_u24_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf count =
      byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf count := by rfl
end U24Proofs

#print axioms U24Proofs.be_read_uint_body_eq
#print axioms U24Proofs.le_read_uint_body_eq
