import Generated.ByteorderLite
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite
namespace Proofs
theorem be_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  BigEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_be_bytes]

theorem be_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  BigEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_be_bytes]

theorem be_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  BigEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_be_bytes]

theorem be_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  BigEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_u128
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U128.to_be_bytes]

theorem le_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  LittleEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_le_bytes]

theorem le_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  LittleEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_le_bytes]

theorem le_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  LittleEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_le_bytes]

theorem le_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  LittleEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_u128
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U128.to_le_bytes]

#print axioms Proofs.be_write_u16_spec
#print axioms Proofs.be_write_u32_spec
#print axioms Proofs.be_write_u64_spec
#print axioms Proofs.be_write_u128_spec
#print axioms Proofs.le_write_u16_spec
#print axioms Proofs.le_write_u32_spec
#print axioms Proofs.le_write_u64_spec
#print axioms Proofs.le_write_u128_spec
