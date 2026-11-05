import LiteSigned128Roots.Funs
import ReadVariableProofs
import VariableProofs
open Aeneas Aeneas.Std Result Std.WP
namespace Signed128Support
abbrev requiredBytes128 := VariableProofs.requiredBytes128
theorem be_read_uint128_body (buf : Slice U8) (nbytes : Usize) : lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes := by rfl
theorem be_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (16 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄ := by
 simpa only [be_read_uint128_body] using ReadVariableProofs.be_read_uint128_spec buf nbytes minimum maximum buffer
theorem be_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
 simpa only [be_read_uint128_body] using ReadVariableProofs.be_read_uint128_invalid buf nbytes bad
theorem be_write_uint128_body (buf : Slice U8) (n : U128) (nbytes : Usize) : lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes := by rfl
theorem be_write_uint128_spec (buf : Slice U8) (n : U128) (nbytes : Usize)
    (fit : VariableProofs.requiredBytes128 n ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_be_bytes n).val.drop (16 - nbytes.val)) ⦄ := by
 simpa only [be_write_uint128_body] using VariableProofs.be_write_uint128_spec buf n nbytes fit maximum buffer
theorem be_write_uint128_invalid (buf : Slice U8) (n : U128) (nbytes : Usize)
    (bad : ¬ (VariableProofs.requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    lite_signed128_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure := by
 simpa only [be_write_uint128_body] using VariableProofs.be_write_uint128_invalid buf n nbytes bad
theorem le_read_uint128_body (buf : Slice U8) (nbytes : Usize) : lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes := by rfl
theorem le_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (16 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄ := by
 simpa only [le_read_uint128_body] using ReadVariableProofs.le_read_uint128_spec buf nbytes minimum maximum buffer
theorem le_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
 simpa only [le_read_uint128_body] using ReadVariableProofs.le_read_uint128_invalid buf nbytes bad
theorem le_write_uint128_body (buf : Slice U8) (n : U128) (nbytes : Usize) : lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes := by rfl
theorem le_write_uint128_spec (buf : Slice U8) (n : U128) (nbytes : Usize)
    (fit : VariableProofs.requiredBytes128 n ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_le_bytes n).val.take nbytes.val) ⦄ := by
 simpa only [le_write_uint128_body] using VariableProofs.le_write_uint128_spec buf n nbytes fit maximum buffer
theorem le_write_uint128_invalid (buf : Slice U8) (n : U128) (nbytes : Usize)
    (bad : ¬ (VariableProofs.requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    lite_signed128_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure := by
 simpa only [le_write_uint128_body] using VariableProofs.le_write_uint128_invalid buf n nbytes bad
end Signed128Support
