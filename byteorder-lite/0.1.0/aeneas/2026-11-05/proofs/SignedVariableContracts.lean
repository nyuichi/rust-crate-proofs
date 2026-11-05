import SignedVariableI24
open Aeneas Aeneas.Std Result Std.WP
open signed_variable_roots.byteorder_lite
namespace SignedWrites
theorem be_write_int_contract (buf : Slice U8) (n : I64) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_be_bytes (truncatedWord n count)).val.drop (8-count.val)) ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 8) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .integerOverflow) ∧
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ buf.length < count.val) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .assertionFailure) := by
  exact ⟨fun h => be_write_int_spec buf n count h.1 h.2.1 h.2.2,
    fun h => be_write_int_bad_count buf n count h,
    fun h => be_write_int_short buf n count h.1 h.2.1 h.2.2⟩
theorem be_write_i24_contract (buf : Slice U8) (n : I32) :
    ((3 ≤ buf.length) → BigEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_be_bytes (truncatedWord (IScalar.cast .I64 n) (3#usize))).val.drop 5) ⦄) ∧
    (buf.length < 3 → BigEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n = .fail .assertionFailure) ∧
    ((truncatedWord (IScalar.cast .I64 n) (3#usize)).val = (IScalar.bv n).toNat % 2^24) := by
  exact ⟨fun h => be_write_i24_spec buf n h,
    fun h => be_write_i24_short buf n h, i32_low24 n⟩
theorem le_write_int_contract (buf : Slice U8) (n : I64) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_le_bytes (truncatedWord n count)).val.take count.val) ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 8) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .integerOverflow) ∧
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ buf.length < count.val) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n count = .fail .assertionFailure) := by
  exact ⟨fun h => le_write_int_spec buf n count h.1 h.2.1 h.2.2,
    fun h => le_write_int_bad_count buf n count h,
    fun h => le_write_int_short buf n count h.1 h.2.1 h.2.2⟩
theorem le_write_i24_contract (buf : Slice U8) (n : I32) :
    ((3 ≤ buf.length) → LittleEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_le_bytes (truncatedWord (IScalar.cast .I64 n) (3#usize))).val.take 3) ⦄) ∧
    (buf.length < 3 → LittleEndian.Insts.Byteorder_liteByteOrder.write_i24 buf n = .fail .assertionFailure) ∧
    ((truncatedWord (IScalar.cast .I64 n) (3#usize)).val = (IScalar.bv n).toNat % 2^24) := by
  exact ⟨fun h => le_write_i24_spec buf n h,
    fun h => le_write_i24_short buf n h, i32_low24 n⟩
end SignedWrites
#print axioms SignedWrites.be_write_int_contract
#print axioms SignedWrites.be_write_i24_contract
#print axioms SignedWrites.le_write_int_contract
#print axioms SignedWrites.le_write_i24_contract
