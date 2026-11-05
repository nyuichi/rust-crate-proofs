import ByteorderFloatRoots.Funs
open Aeneas Aeneas.Std Result Std.WP
open byteorder_float_roots
namespace FloatProofs
private theorem be_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes n).val ⦄ := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_be_bytes]

private theorem be_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_f32_spec (buf : Slice U8) (n : F32) (h : 4 ≤ buf.length) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes (F32.toWord n)).val ⦄ := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 core.f32.F32.to_bits
  simp only [Std.bind_ok]
  exact be_write_u32_spec buf _ h

theorem be_write_f32_short (buf : Slice U8) (n : F32) (h : buf.length < 4) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 core.f32.F32.to_bits
  simp only [Std.bind_ok]
  exact be_write_u32_short buf _ h

theorem be_write_f32_contract (buf : Slice U8) (n : F32) :
  (4 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_be_bytes (F32.toWord n)).val ⦄) ∧
  (buf.length < 4 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_f32 buf n = .fail .panic) :=
  ⟨fun h => be_write_f32_spec buf n h, fun h => be_write_f32_short buf n h⟩
private theorem be_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes n).val ⦄ := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_be_bytes]

private theorem be_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem be_write_f64_spec (buf : Slice U8) (n : F64) (h : 8 ≤ buf.length) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes (F64.toWord n)).val ⦄ := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 core.f64.F64.to_bits
  simp only [Std.bind_ok]
  exact be_write_u64_spec buf _ h

theorem be_write_f64_short (buf : Slice U8) (n : F64) (h : buf.length < 8) :
  byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 buf n = .fail .panic := by
  unfold byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 core.f64.F64.to_bits
  simp only [Std.bind_ok]
  exact be_write_u64_short buf _ h

theorem be_write_f64_contract (buf : Slice U8) (n : F64) :
  (8 ≤ buf.length → byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_be_bytes (F64.toWord n)).val ⦄) ∧
  (buf.length < 8 → byteorder.BigEndian.Insts.ByteorderByteOrder.write_f64 buf n = .fail .panic) :=
  ⟨fun h => be_write_f64_spec buf n h, fun h => be_write_f64_short buf n h⟩
private theorem le_write_u32_spec (buf : Slice U8) (n : U32) (h : 4 ≤ buf.length) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes n).val ⦄ := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U32.to_le_bytes]

private theorem le_write_u32_short (buf : Slice U8) (n : U32) (h : buf.length < 4) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u32
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_f32_spec (buf : Slice U8) (n : F32) (h : 4 ≤ buf.length) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes (F32.toWord n)).val ⦄ := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 core.f32.F32.to_bits
  simp only [Std.bind_ok]
  exact le_write_u32_spec buf _ h

theorem le_write_f32_short (buf : Slice U8) (n : F32) (h : buf.length < 4) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 core.f32.F32.to_bits
  simp only [Std.bind_ok]
  exact le_write_u32_short buf _ h

theorem le_write_f32_contract (buf : Slice U8) (n : F32) :
  (4 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U32.to_le_bytes (F32.toWord n)).val ⦄) ∧
  (buf.length < 4 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f32 buf n = .fail .panic) :=
  ⟨fun h => le_write_f32_spec buf n h, fun h => le_write_f32_short buf n h⟩
private theorem le_write_u64_spec (buf : Slice U8) (n : U64) (h : 8 ≤ buf.length) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes n).val ⦄ := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  simp_all [Array.to_slice, core.num.U64.to_le_bytes]

private theorem le_write_u64_short (buf : Slice U8) (n : U64) (h : buf.length < 8) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_u64
  simp only [core.slice.index.Slice.index_mut, core.slice.index.SliceIndexRangeToUsizeSlice,
    core.slice.index.SliceIndexRangeToUsizeSlice.index_mut]
  split
  · scalar_tac
  · simp only [Std.bind_fail]

theorem le_write_f64_spec (buf : Slice U8) (n : F64) (h : 8 ≤ buf.length) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes (F64.toWord n)).val ⦄ := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 core.f64.F64.to_bits
  simp only [Std.bind_ok]
  exact le_write_u64_spec buf _ h

theorem le_write_f64_short (buf : Slice U8) (n : F64) (h : buf.length < 8) :
  byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 buf n = .fail .panic := by
  unfold byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 core.f64.F64.to_bits
  simp only [Std.bind_ok]
  exact le_write_u64_short buf _ h

theorem le_write_f64_contract (buf : Slice U8) (n : F64) :
  (8 ≤ buf.length → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 buf n
    ⦃ r => r.val = buf.val.setSlice! 0 (core.num.U64.to_le_bytes (F64.toWord n)).val ⦄) ∧
  (buf.length < 8 → byteorder.LittleEndian.Insts.ByteorderByteOrder.write_f64 buf n = .fail .panic) :=
  ⟨fun h => le_write_f64_spec buf n h, fun h => le_write_f64_short buf n h⟩
end FloatProofs
#print axioms FloatProofs.be_write_f32_contract
#print axioms FloatProofs.be_write_f64_contract
#print axioms FloatProofs.le_write_f32_contract
#print axioms FloatProofs.le_write_f64_contract
