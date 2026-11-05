import ReadVariableGenerated.ByteorderLite
import Mathlib.Tactic.IntervalCases
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite
namespace ReadVariableProofs
private theorem from_be_bytes_val (a : Aeneas.Std.Array U8 8#usize) :
    (core.num.U64.from_be_bytes a).val = (BitVec.fromBEBytes (a.val.map U8.bv)).toNat := by
  simp only [core.num.U64.from_be_bytes, UScalar.val, BitVec.toNat_cast]

theorem be_read_uint_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (8 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_uint
  step
  step
  step
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  all_goals simp_all [from_be_bytes_val, Array.to_slice, Array.repeat_val, List.slice, List.setSlice!]
  · omega
  · have hlen3 : s3.val.length = nbytes.val := by
      simp_all
    have hsub : 8 - (8 - nbytes.val) = nbytes.val := by omega
    have hsum : 8 - nbytes.val + nbytes.val = 8 := by omega
    have hlist : (back s3).val =
        List.replicate (8 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val := by
      rw [hback]
      simp only [hlen3, hsub, Nat.min_self, hsum]
      simp only [← hlen3, List.take_length]
      rw [s3_post]
      have hz : [0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8] =
          List.replicate 8 (0#u8) := rfl
      interval_cases hn : nbytes.val <;> simp_all
    exact congrArg (fun bytes : List U8 =>
      (BitVec.fromBEBytes (bytes.map U8.bv)).toNat) hlist
private theorem from_be_bytes_val128 (a : Aeneas.Std.Array U8 16#usize) :
    (core.num.U128.from_be_bytes a).val = (BitVec.fromBEBytes (a.val.map U8.bv)).toNat := by
  simp only [core.num.U128.from_be_bytes, UScalar.val, BitVec.toNat_cast]

theorem be_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (16 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_uint128
  step
  step
  step
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  all_goals simp_all [from_be_bytes_val128, Array.to_slice, Array.repeat_val, List.slice, List.setSlice!]
  · omega
  · have hlen3 : s3.val.length = nbytes.val := by
      simp_all
    have hsub : 16 - (16 - nbytes.val) = nbytes.val := by omega
    have hsum : 16 - nbytes.val + nbytes.val = 16 := by omega
    have hlist : (back s3).val =
        List.replicate (16 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val := by
      rw [hback]
      simp only [hlen3, hsub, Nat.min_self, hsum]
      simp only [← hlen3, List.take_length]
      rw [s3_post]
      have hz : [0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8] =
          List.replicate 16 (0#u8) := rfl
      interval_cases hn : nbytes.val <;> simp_all
    exact congrArg (fun bytes : List U8 =>
      (BitVec.fromBEBytes (bytes.map U8.bv)).toNat) hlist

private theorem from_le_bytes_val (a : Aeneas.Std.Array U8 8#usize) :
    (core.num.U64.from_le_bytes a).val = (BitVec.fromLEBytes (a.val.map U8.bv)).toNat := by
  simp only [core.num.U64.from_le_bytes, UScalar.val, BitVec.toNat_cast]
theorem le_read_uint_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (8 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_uint
  step
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  all_goals simp_all [from_le_bytes_val, Array.to_slice, Array.repeat_val, List.slice, List.setSlice!]
  have hlist : (back s2).val = buf.val.take nbytes.val ++
      List.replicate (8 - nbytes.val) (0#u8) := by
    rw [hback, s2_post]
    interval_cases hn : nbytes.val <;> simp_all
  exact congrArg (fun bytes : List U8 =>
    (BitVec.fromLEBytes (bytes.map U8.bv)).toNat) hlist
private theorem from_le_bytes_val128 (a : Aeneas.Std.Array U8 16#usize) :
    (core.num.U128.from_le_bytes a).val = (BitVec.fromLEBytes (a.val.map U8.bv)).toNat := by
  simp only [core.num.U128.from_le_bytes, UScalar.val, BitVec.toNat_cast]
theorem le_read_uint128_spec (buf : Slice U8) (nbytes : Usize)
    (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (16 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128
  step
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  all_goals simp_all [from_le_bytes_val128, Array.to_slice, Array.repeat_val, List.slice, List.setSlice!]
  have hlist : (back s2).val = buf.val.take nbytes.val ++
      List.replicate (16 - nbytes.val) (0#u8) := by
    rw [hback, s2_post]
    interval_cases hn : nbytes.val <;> simp_all
  exact congrArg (fun bytes : List U8 =>
    (BitVec.fromLEBytes (bytes.map U8.bv)).toNat) hlist

theorem be_read_uint_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_uint
  simp only [Std.bind_ok, bind_tc_ok, Array.to_slice, Array.repeat_val,
    Slice.len, Slice.length, massert, lift, Slice.from_val, List.length_replicate]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
theorem le_read_uint_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_uint
  simp only [Std.bind_ok, bind_tc_ok, Array.to_slice, Array.repeat_val,
    Slice.len, Slice.length, massert, lift, Slice.from_val, List.length_replicate]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
theorem be_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_uint128
  simp only [Std.bind_ok, bind_tc_ok, Array.to_slice, Array.repeat_val,
    Slice.len, Slice.length, massert, lift, Slice.from_val, List.length_replicate]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
theorem le_read_uint128_invalid (buf : Slice U8) (nbytes : Usize)
    (bad : ¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128
  simp only [Std.bind_ok, bind_tc_ok, Array.to_slice, Array.repeat_val,
    Slice.len, Slice.length, massert, lift, Slice.from_val, List.length_replicate]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
end ReadVariableProofs
#print axioms ReadVariableProofs.be_read_uint_spec

#print axioms ReadVariableProofs.be_read_uint128_spec

#print axioms ReadVariableProofs.le_read_uint_spec

#print axioms ReadVariableProofs.le_read_uint128_spec

#print axioms ReadVariableProofs.be_read_uint_invalid

#print axioms ReadVariableProofs.le_read_uint_invalid
#print axioms ReadVariableProofs.be_read_uint128_invalid
#print axioms ReadVariableProofs.le_read_uint128_invalid

namespace ReadVariableProofs
theorem be_read_uint_contract (buf : Slice U8) (nbytes : Usize) :
    (∀ (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
       (buffer : nbytes.val ≤ buf.length), BigEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (8 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes = .fail .assertionFailure) := by
  exact ⟨fun h1 h2 h3 => be_read_uint_spec buf nbytes h1 h2 h3,
    fun h => be_read_uint_invalid buf nbytes h⟩
theorem be_read_uint128_contract (buf : Slice U8) (nbytes : Usize) :
    (∀ (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
       (buffer : nbytes.val ≤ buf.length), BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (16 - nbytes.val) (0#u8) ++ buf.val.take nbytes.val).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure) := by
  exact ⟨fun h1 h2 h3 => be_read_uint128_spec buf nbytes h1 h2 h3,
    fun h => be_read_uint128_invalid buf nbytes h⟩
theorem le_read_uint_contract (buf : Slice U8) (nbytes : Usize) :
    (∀ (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
       (buffer : nbytes.val ≤ buf.length), LittleEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (8 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.read_uint buf nbytes = .fail .assertionFailure) := by
  exact ⟨fun h1 h2 h3 => le_read_uint_spec buf nbytes h1 h2 h3,
    fun h => le_read_uint_invalid buf nbytes h⟩
theorem le_read_uint128_contract (buf : Slice U8) (nbytes : Usize) :
    (∀ (minimum : 1 ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
       (buffer : nbytes.val ≤ buf.length), LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take nbytes.val ++ List.replicate (16 - nbytes.val) (0#u8)).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.read_uint128 buf nbytes = .fail .assertionFailure) := by
  exact ⟨fun h1 h2 h3 => le_read_uint128_spec buf nbytes h1 h2 h3,
    fun h => le_read_uint128_invalid buf nbytes h⟩
end ReadVariableProofs
#print axioms ReadVariableProofs.be_read_uint_contract
#print axioms ReadVariableProofs.be_read_uint128_contract
#print axioms ReadVariableProofs.le_read_uint_contract
#print axioms ReadVariableProofs.le_read_uint128_contract
