import SignedProofs
import Generated.Byteorder
open Aeneas Aeneas.Std Result Std.WP
open byteorder
namespace Proofs
theorem be_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  BigEndian.Insts.ByteorderByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.ByteorderByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_be_bytes]

theorem be_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  BigEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.ByteorderByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_be_bytes]

theorem be_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  BigEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.ByteorderByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_be_bytes]

theorem be_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  BigEndian.Insts.ByteorderByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes n).val ⦄ := by
  unfold BigEndian.Insts.ByteorderByteOrder.write_u128
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U128.to_be_bytes]

theorem le_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  LittleEndian.Insts.ByteorderByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.ByteorderByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_le_bytes]

theorem le_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.ByteorderByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_le_bytes]

theorem le_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.ByteorderByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_le_bytes]

theorem le_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  LittleEndian.Insts.ByteorderByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes n).val ⦄ := by
  unfold LittleEndian.Insts.ByteorderByteOrder.write_u128
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

theorem BigEndian_default_panics : byteorder.BigEndian.Insts.CoreDefaultDefault.default = .fail .panic := by rfl
#print axioms Proofs.BigEndian_default_panics
theorem LittleEndian_default_panics : byteorder.LittleEndian.Insts.CoreDefaultDefault.default = .fail .panic := by rfl
#print axioms Proofs.LittleEndian_default_panics
end Proofs
namespace Proofs
theorem be_write_u16_short (buf : Slice U8) (n : U16) (h : buf.length < 2) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u16 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u16
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_u16_contract (buf : Slice U8) (n : U16) :
  (2 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes n).val ⦄) ∧
  (buf.length < 2 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u16 buf n = .fail .panic) :=
  ⟨fun h => be_write_u16_spec buf n h, fun h => be_write_u16_short buf n h⟩
theorem be_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_u32_contract (buf : Slice U8) (n : U32) :
  (4 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes n).val ⦄) ∧
  (buf.length < 4 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic) :=
  ⟨fun h => be_write_u32_spec buf n h, fun h => be_write_u32_short buf n h⟩
theorem be_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_u64_contract (buf : Slice U8) (n : U64) :
  (8 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes n).val ⦄) ∧
  (buf.length < 8 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic) :=
  ⟨fun h => be_write_u64_spec buf n h, fun h => be_write_u64_short buf n h⟩
theorem be_write_u128_short (buf : Slice U8) (n : U128) (h : buf.length < 16) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u128 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u128
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_u128_contract (buf : Slice U8) (n : U128) :
  (16 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes n).val ⦄) ∧
  (buf.length < 16 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_u128 buf n = .fail .panic) :=
  ⟨fun h => be_write_u128_spec buf n h, fun h => be_write_u128_short buf n h⟩
theorem le_write_u16_short (buf : Slice U8) (n : U16) (h : buf.length < 2) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u16 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u16
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_u16_contract (buf : Slice U8) (n : U16) :
  (2 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes n).val ⦄) ∧
  (buf.length < 2 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u16 buf n = .fail .panic) :=
  ⟨fun h => le_write_u16_spec buf n h, fun h => le_write_u16_short buf n h⟩
theorem le_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_u32_contract (buf : Slice U8) (n : U32) :
  (4 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes n).val ⦄) ∧
  (buf.length < 4 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic) :=
  ⟨fun h => le_write_u32_spec buf n h, fun h => le_write_u32_short buf n h⟩
theorem le_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_u64_contract (buf : Slice U8) (n : U64) :
  (8 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes n).val ⦄) ∧
  (buf.length < 8 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic) :=
  ⟨fun h => le_write_u64_spec buf n h, fun h => le_write_u64_short buf n h⟩
theorem le_write_u128_short (buf : Slice U8) (n : U128) (h : buf.length < 16) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u128 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u128
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_u128_contract (buf : Slice U8) (n : U128) :
  (16 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes n).val ⦄) ∧
  (buf.length < 16 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u128 buf n = .fail .panic) :=
  ⟨fun h => le_write_u128_spec buf n h, fun h => le_write_u128_short buf n h⟩
end Proofs
#print axioms Proofs.be_write_u16_contract
#print axioms Proofs.be_write_u32_contract
#print axioms Proofs.be_write_u64_contract
#print axioms Proofs.be_write_u128_contract
#print axioms Proofs.le_write_u16_contract
#print axioms Proofs.le_write_u32_contract
#print axioms Proofs.le_write_u64_contract
#print axioms Proofs.le_write_u128_contract
