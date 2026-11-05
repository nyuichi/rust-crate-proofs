import U24ReadSupport
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite_u24_roots.byteorder_lite
namespace U24Proofs
theorem be_read_u24_spec (buf : Slice U8) (enough : 3 ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromBEBytes ((List.replicate 5 (0#u8) ++ buf.val.take 3).map U8.bv)).toNat % 2^32 ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_u24
  step with U24ReadSupport.be_read_uint_spec
  simp_all [UScalar.cast_val_eq]
  rfl
theorem le_read_u24_spec (buf : Slice U8) (enough : 3 ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 3 ++ List.replicate 5 (0#u8)).map U8.bv)).toNat % 2^32 ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_u24
  step with U24ReadSupport.le_read_uint_spec
  simp_all [UScalar.cast_val_eq]
  rfl
end U24Proofs
#print axioms U24Proofs.be_read_u24_spec
#print axioms U24Proofs.le_read_u24_spec

namespace U24Proofs
theorem be_read_u24_short (buf : Slice U8) (short : buf.length < 3) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_u24 buf = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_u24
  rw [U24ReadSupport.be_read_uint_invalid buf (3#usize) (by
    intro h
    have h3 : (3#usize).val = 3 := by decide
    simp only [h3] at h
    omega)]
  simp only [Std.bind_fail, bind_tc_fail]
theorem le_read_u24_short (buf : Slice U8) (short : buf.length < 3) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_u24 buf = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_u24
  rw [U24ReadSupport.le_read_uint_invalid buf (3#usize) (by
    intro h
    have h3 : (3#usize).val = 3 := by decide
    simp only [h3] at h
    omega)]
  simp only [Std.bind_fail, bind_tc_fail]
end U24Proofs
#print axioms U24Proofs.be_read_u24_short
#print axioms U24Proofs.le_read_u24_short
