import ReadVariableProofs
import ByteorderReadPureRoots.Funs
open Aeneas Aeneas.Std Result Std.WP
namespace ByteReadSupport
theorem be_read_uint_body (buf : Slice U8) (nbytes : Usize) : byteorder_read_pure_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf nbytes = byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf nbytes := by rfl
theorem be_read_uint_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    byteorder_read_pure_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (8 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄ := by
 simpa only [be_read_uint_body] using ReadVariableProofs.be_read_uint_spec buf nbytes minimum maximum buffer
theorem be_read_uint_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    byteorder_read_pure_roots.byteorder.BigEndian.Insts.ByteorderByteOrder.read_uint buf nbytes = .fail .assertionFailure := by
 simpa only [be_read_uint_body] using ReadVariableProofs.be_read_uint_invalid buf nbytes bad
theorem le_read_uint_body (buf : Slice U8) (nbytes : Usize) : byteorder_read_pure_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf nbytes = byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf nbytes := by rfl
theorem le_read_uint_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    byteorder_read_pure_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (8 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄ := by
 simpa only [le_read_uint_body] using ReadVariableProofs.le_read_uint_spec buf nbytes minimum maximum buffer
theorem le_read_uint_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    byteorder_read_pure_roots.byteorder.LittleEndian.Insts.ByteorderByteOrder.read_uint buf nbytes = .fail .assertionFailure := by
 simpa only [le_read_uint_body] using ReadVariableProofs.le_read_uint_invalid buf nbytes bad
end ByteReadSupport
