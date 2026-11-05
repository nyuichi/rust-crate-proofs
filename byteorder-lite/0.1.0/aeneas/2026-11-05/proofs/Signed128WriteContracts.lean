import Signed128WriteFailure
open Aeneas Aeneas.Std Result Std.WP
open lite_signed128_roots.byteorder_lite
namespace Signed128Writes
theorem be_write_int128_contract (buf : Slice U8) (n : I128) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U128.to_be_bytes (truncatedWord n count)).val.drop (16-count.val)) ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 16) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .integerOverflow) ∧
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ buf.length < count.val) →
      BigEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .assertionFailure) := by
  exact ⟨fun h => be_write_int128_spec buf n count h.1 h.2.1 h.2.2,
    fun h => be_write_int128_bad_count buf n count h,
    fun h => be_write_int128_short buf n count h.1 h.2.1 h.2.2⟩
theorem le_write_int128_contract (buf : Slice U8) (n : I128) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U128.to_le_bytes (truncatedWord n count)).val.take count.val) ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 16) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .integerOverflow) ∧
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ buf.length < count.val) →
      LittleEndian.Insts.Byteorder_liteByteOrder.write_int128 buf n count = .fail .assertionFailure) := by
  exact ⟨fun h => le_write_int128_spec buf n count h.1 h.2.1 h.2.2,
    fun h => le_write_int128_bad_count buf n count h,
    fun h => le_write_int128_short buf n count h.1 h.2.1 h.2.2⟩
end Signed128Writes
#print axioms Signed128Writes.be_write_int128_contract
#print axioms Signed128Writes.le_write_int128_contract
