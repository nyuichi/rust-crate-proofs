import SignedVariableFailure
open Aeneas Aeneas.Std Result Std.WP
open signed_variable_roots.byteorder_lite
namespace SignedWrites
theorem i32_low24 (n : I32) :
    (truncatedWord (IScalar.cast .I64 n) (3#usize)).val = (IScalar.bv n).toNat % 2^24 := by
  have hb : (IScalar.bv (IScalar.cast .I64 n)).setWidth 24 = (IScalar.bv n).setWidth 24 := by
    rw [IScalar.cast_bv_eq]
    apply BitVec.eq_of_getElem_eq
    intro i hi
    have hi32 : i < 32 := by omega
    simp [BitVec.getLsbD_signExtend, hi32]
    omega
  have hn := congrArg BitVec.toNat hb
  have h3 : (3#usize).val = 3 := by decide
  calc
    _ = (IScalar.bv (IScalar.cast .I64 n)).toNat % 2^24 := by
      simpa only [h3, Nat.reduceMul] using truncatedWord_val (IScalar.cast .I64 n) (3#usize) (by decide)
    _ = _ := by simpa only [BitVec.toNat_setWidth] using hn
end SignedWrites
#print axioms SignedWrites.i32_low24

namespace SignedWrites
theorem be_write_i24_spec (buf : Slice U8) (n : I32) (enough : 3 ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_be_bytes (truncatedWord (IScalar.cast .I64 n) (3#usize))).val.drop 5) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_i24
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    be_write_int_spec buf (IScalar.cast .I64 n) (3#usize) (by decide) (by decide) enough

theorem be_write_i24_short (buf : Slice U8) (n : I32) (short : buf.length < 3) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_i24
  simpa only [lift, Std.bind_ok, bind_tc_ok] using
    be_write_int_short buf (IScalar.cast .I64 n) (3#usize) (by decide) (by decide) short
theorem le_write_i24_spec (buf : Slice U8) (n : I32) (enough : 3 ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_le_bytes (truncatedWord (IScalar.cast .I64 n) (3#usize))).val.take 3) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_i24
  have h3 : (3#usize).val = 3 := by decide
  simpa only [lift, Std.bind_ok, bind_tc_ok, h3, Nat.reduceSub] using
    le_write_int_spec buf (IScalar.cast .I64 n) (3#usize) (by decide) (by decide) enough

theorem le_write_i24_short (buf : Slice U8) (n : I32) (short : buf.length < 3) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_i24
  simpa only [lift, Std.bind_ok, bind_tc_ok] using
    le_write_int_short buf (IScalar.cast .I64 n) (3#usize) (by decide) (by decide) short
end SignedWrites
#print axioms SignedWrites.be_write_i24_spec
#print axioms SignedWrites.be_write_i24_short
#print axioms SignedWrites.le_write_i24_spec
#print axioms SignedWrites.le_write_i24_short
