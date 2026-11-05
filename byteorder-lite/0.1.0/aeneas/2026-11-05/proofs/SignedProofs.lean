import ByteorderLiteApiRoots.Funs
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite_api_roots
namespace SignedProofs
private theorem be_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes n).val ⦄ := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_be_bytes]

theorem be_write_i16_spec (buf : Slice U8) (n : I16) (h : 2 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes (⟨n.bv⟩ : U16)).val ⦄ := by
  have hc : IScalar.hcast .U16 n = (⟨n.bv⟩ : U16) := by simp [IScalar.hcast]
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16
  simp only [hc, lift, Std.bind_ok]
  exact be_write_u16_spec buf _ h
private theorem be_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes n).val ⦄ := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_be_bytes]

theorem be_write_i32_spec (buf : Slice U8) (n : I32) (h : 4 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes (⟨n.bv⟩ : U32)).val ⦄ := by
  have hc : IScalar.hcast .U32 n = (⟨n.bv⟩ : U32) := by simp [IScalar.hcast]
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32
  simp only [hc, lift, Std.bind_ok]
  exact be_write_u32_spec buf _ h
private theorem be_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes n).val ⦄ := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_be_bytes]

theorem be_write_i64_spec (buf : Slice U8) (n : I64) (h : 8 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes (⟨n.bv⟩ : U64)).val ⦄ := by
  have hc : IScalar.hcast .U64 n = (⟨n.bv⟩ : U64) := by simp [IScalar.hcast]
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64
  simp only [hc, lift, Std.bind_ok]
  exact be_write_u64_spec buf _ h
private theorem be_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes n).val ⦄ := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u128
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U128.to_be_bytes]

theorem be_write_i128_spec (buf : Slice U8) (n : I128) (h : 16 ≤ buf.length) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes (⟨n.bv⟩ : U128)).val ⦄ := by
  have hc : IScalar.hcast .U128 n = (⟨n.bv⟩ : U128) := by simp [IScalar.hcast]
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128
  simp only [hc, lift, Std.bind_ok]
  exact be_write_u128_spec buf _ h
private theorem le_write_u16_spec (buf : Slice U8) (n : U16) (h : 2 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes n).val ⦄ := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u16
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U16.to_le_bytes]

theorem le_write_i16_spec (buf : Slice U8) (n : I16) (h : 2 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes (⟨n.bv⟩ : U16)).val ⦄ := by
  have hc : IScalar.hcast .U16 n = (⟨n.bv⟩ : U16) := by simp [IScalar.hcast]
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16
  simp only [hc, lift, Std.bind_ok]
  exact le_write_u16_spec buf _ h
private theorem le_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes n).val ⦄ := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_le_bytes]

theorem le_write_i32_spec (buf : Slice U8) (n : I32) (h : 4 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes (⟨n.bv⟩ : U32)).val ⦄ := by
  have hc : IScalar.hcast .U32 n = (⟨n.bv⟩ : U32) := by simp [IScalar.hcast]
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32
  simp only [hc, lift, Std.bind_ok]
  exact le_write_u32_spec buf _ h
private theorem le_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes n).val ⦄ := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_le_bytes]

theorem le_write_i64_spec (buf : Slice U8) (n : I64) (h : 8 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes (⟨n.bv⟩ : U64)).val ⦄ := by
  have hc : IScalar.hcast .U64 n = (⟨n.bv⟩ : U64) := by simp [IScalar.hcast]
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64
  simp only [hc, lift, Std.bind_ok]
  exact le_write_u64_spec buf _ h
private theorem le_write_u128_spec (buf : Slice U8) (n : U128) (h : 16 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes n).val ⦄ := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u128
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U128.to_le_bytes]

theorem le_write_i128_spec (buf : Slice U8) (n : I128) (h : 16 ≤ buf.length) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes (⟨n.bv⟩ : U128)).val ⦄ := by
  have hc : IScalar.hcast .U128 n = (⟨n.bv⟩ : U128) := by simp [IScalar.hcast]
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128
  simp only [hc, lift, Std.bind_ok]
  exact le_write_u128_spec buf _ h
end SignedProofs
#print axioms SignedProofs.be_write_i16_spec
#print axioms SignedProofs.be_write_i32_spec
#print axioms SignedProofs.be_write_i64_spec
#print axioms SignedProofs.be_write_i128_spec
#print axioms SignedProofs.le_write_i16_spec
#print axioms SignedProofs.le_write_i32_spec
#print axioms SignedProofs.le_write_i64_spec
#print axioms SignedProofs.le_write_i128_spec

namespace SignedProofs
private theorem be_write_u16_short (buf : Slice U8) (n : U16) (h : buf.length < 2) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u16
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_i16_short (buf : Slice U8) (n : I16) (h : buf.length < 2) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16
  simp only [lift, Std.bind_ok]
  exact be_write_u16_short buf _ h

theorem be_write_i16_contract (buf : Slice U8) (n : I16) :
  (2 ≤ buf.length → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_be_bytes (⟨n.bv⟩ : U16)).val ⦄) ∧
  (buf.length < 2 → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n = .fail .panic) :=
  ⟨fun h => be_write_i16_spec buf n h, fun h => be_write_i16_short buf n h⟩
private theorem be_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_i32_short (buf : Slice U8) (n : I32) (h : buf.length < 4) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32
  simp only [lift, Std.bind_ok]
  exact be_write_u32_short buf _ h

theorem be_write_i32_contract (buf : Slice U8) (n : I32) :
  (4 ≤ buf.length → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes (⟨n.bv⟩ : U32)).val ⦄) ∧
  (buf.length < 4 → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n = .fail .panic) :=
  ⟨fun h => be_write_i32_spec buf n h, fun h => be_write_i32_short buf n h⟩
private theorem be_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_i64_short (buf : Slice U8) (n : I64) (h : buf.length < 8) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64
  simp only [lift, Std.bind_ok]
  exact be_write_u64_short buf _ h

theorem be_write_i64_contract (buf : Slice U8) (n : I64) :
  (8 ≤ buf.length → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes (⟨n.bv⟩ : U64)).val ⦄) ∧
  (buf.length < 8 → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n = .fail .panic) :=
  ⟨fun h => be_write_i64_spec buf n h, fun h => be_write_i64_short buf n h⟩
private theorem be_write_u128_short (buf : Slice U8) (n : U128) (h : buf.length < 16) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u128
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_i128_short (buf : Slice U8) (n : I128) (h : buf.length < 16) :
  byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n = .fail .panic := by
  unfold byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128
  simp only [lift, Std.bind_ok]
  exact be_write_u128_short buf _ h

theorem be_write_i128_contract (buf : Slice U8) (n : I128) :
  (16 ≤ buf.length → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_be_bytes (⟨n.bv⟩ : U128)).val ⦄) ∧
  (buf.length < 16 → byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n = .fail .panic) :=
  ⟨fun h => be_write_i128_spec buf n h, fun h => be_write_i128_short buf n h⟩
private theorem le_write_u16_short (buf : Slice U8) (n : U16) (h : buf.length < 2) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u16 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u16
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_i16_short (buf : Slice U8) (n : I16) (h : buf.length < 2) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16
  simp only [lift, Std.bind_ok]
  exact le_write_u16_short buf _ h

theorem le_write_i16_contract (buf : Slice U8) (n : I16) :
  (2 ≤ buf.length → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U16.to_le_bytes (⟨n.bv⟩ : U16)).val ⦄) ∧
  (buf.length < 2 → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i16 buf n = .fail .panic) :=
  ⟨fun h => le_write_i16_spec buf n h, fun h => le_write_i16_short buf n h⟩
private theorem le_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_i32_short (buf : Slice U8) (n : I32) (h : buf.length < 4) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32
  simp only [lift, Std.bind_ok]
  exact le_write_u32_short buf _ h

theorem le_write_i32_contract (buf : Slice U8) (n : I32) :
  (4 ≤ buf.length → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes (⟨n.bv⟩ : U32)).val ⦄) ∧
  (buf.length < 4 → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i32 buf n = .fail .panic) :=
  ⟨fun h => le_write_i32_spec buf n h, fun h => le_write_i32_short buf n h⟩
private theorem le_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_i64_short (buf : Slice U8) (n : I64) (h : buf.length < 8) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64
  simp only [lift, Std.bind_ok]
  exact le_write_u64_short buf _ h

theorem le_write_i64_contract (buf : Slice U8) (n : I64) :
  (8 ≤ buf.length → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes (⟨n.bv⟩ : U64)).val ⦄) ∧
  (buf.length < 8 → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i64 buf n = .fail .panic) :=
  ⟨fun h => le_write_i64_spec buf n h, fun h => le_write_i64_short buf n h⟩
private theorem le_write_u128_short (buf : Slice U8) (n : U128) (h : buf.length < 16) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u128 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u128
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_i128_short (buf : Slice U8) (n : I128) (h : buf.length < 16) :
  byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n = .fail .panic := by
  unfold byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128
  simp only [lift, Std.bind_ok]
  exact le_write_u128_short buf _ h

theorem le_write_i128_contract (buf : Slice U8) (n : I128) :
  (16 ≤ buf.length → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U128.to_le_bytes (⟨n.bv⟩ : U128)).val ⦄) ∧
  (buf.length < 16 → byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i128 buf n = .fail .panic) :=
  ⟨fun h => le_write_i128_spec buf n h, fun h => le_write_i128_short buf n h⟩
end SignedProofs
#print axioms SignedProofs.be_write_i16_contract
#print axioms SignedProofs.be_write_i32_contract
#print axioms SignedProofs.be_write_i64_contract
#print axioms SignedProofs.be_write_i128_contract
#print axioms SignedProofs.le_write_i16_contract
#print axioms SignedProofs.le_write_i32_contract
#print axioms SignedProofs.le_write_i64_contract
#print axioms SignedProofs.le_write_i128_contract
