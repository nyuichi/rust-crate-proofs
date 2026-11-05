import ReadVariableProofs
import ByteorderRead128Roots.Funs
open Aeneas Aeneas.Std Result Std.WP
namespace Byte128Support
theorem be_read_uint128_body (buf : Slice U8) (nbytes : Usize) : byteorder_read128_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes = byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes := by rfl
theorem be_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    byteorder_read128_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (16 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄ := by
 simpa only [be_read_uint128_body] using ReadVariableProofs.be_read_uint128_spec buf nbytes minimum maximum buffer
theorem be_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    byteorder_read128_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
 simpa only [be_read_uint128_body] using ReadVariableProofs.be_read_uint128_invalid buf nbytes bad
theorem le_read_uint128_body (buf : Slice U8) (nbytes : Usize) : byteorder_read128_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes = byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes := by rfl
theorem le_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    byteorder_read128_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (16 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄ := by
 simpa only [le_read_uint128_body] using ReadVariableProofs.le_read_uint128_spec buf nbytes minimum maximum buffer
theorem le_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    byteorder_read128_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
 simpa only [le_read_uint128_body] using ReadVariableProofs.le_read_uint128_invalid buf nbytes bad
end Byte128Support
