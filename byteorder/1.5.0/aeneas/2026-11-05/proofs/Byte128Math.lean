import Aeneas
open Aeneas Aeneas.Std
namespace Byte128Math
theorem signed_low_bits (x : BitVec 128) (s : Nat) (hs : s < 128) :
    (x <<< s).sshiftRight s = (x.setWidth (128-s)).signExtend 128 := by
  apply BitVec.eq_of_getElem_eq
  intro i hi
  rw [← BitVec.getLsbD_eq_getElem, ← BitVec.getLsbD_eq_getElem]
  simp [BitVec.getLsbD_sshiftRight, BitVec.getLsbD_signExtend,
    BitVec.msb_eq_getLsbD_last, BitVec.getLsbD_shiftLeft]
  have hni : ¬ 128 ≤ i := by omega
  have hns : ¬ 127 < s := by omega
  have heq : 127-s = 128-s-1 := by omega
  have hlast : 128-s-1 < 128 := by omega
  by_cases h : s+i < 128
  · have hw : i < 128-s := by omega
    simp [hni, hns, heq, hi, h, hw, hs, hlast]
  · have hw : ¬ i < 128-s := by omega
    simp [hni, hns, heq, hi, h, hw, hs, hlast]
theorem zero_padding_le (bytes : List (BitVec 8)) (count : Nat) :
    BitVec.fromLEBytes (bytes ++ List.replicate count 0) =
      (BitVec.fromLEBytes bytes).setWidth (8*(bytes ++ List.replicate count 0).length) := by
  apply BitVec.eq_of_getElem_eq
  intro i hi
  rw [BitVec.fromLEBytes_getElem, BitVec.getElem_setWidth]
  by_cases h : i < 8*bytes.length
  · rw [BitVec.getLsbD_eq_getElem h, BitVec.fromLEBytes_getElem]
    rw [List.getElem_append_left]
  · rw [BitVec.getLsbD_of_ge (BitVec.fromLEBytes bytes) i (by omega)]
    rw [List.getElem_append_right]
    · simp [Byte.testBit]
    · omega
theorem zero_padding_le_val (bytes : List (BitVec 8)) (count : Nat) :
    (BitVec.fromLEBytes (bytes ++ List.replicate count 0)).toNat =
      (BitVec.fromLEBytes bytes).toNat := by
  have h := congrArg BitVec.toNat (zero_padding_le bytes count)
  exact h.trans (BitVec.toNat_setWidth_of_le (by simp))

theorem zero_padding_be_val (bytes : List (BitVec 8)) (count : Nat) :
    (BitVec.fromBEBytes (List.replicate count 0 ++ bytes)).toNat =
      (BitVec.fromBEBytes bytes).toNat := by
  have hl : (List.replicate count (0 : BitVec 8) ++ bytes).reverse =
      bytes.reverse ++ List.replicate count 0 := by simp
  have hd := congrArg (fun bs : List (BitVec 8) => (BitVec.fromLEBytes bs).toNat) hl
  calc
    _ = (BitVec.fromLEBytes (List.replicate count 0 ++ bytes).reverse).toNat := by
      simp only [BitVec.fromBEBytes, BitVec.toNat_cast]
    _ = (BitVec.fromLEBytes (bytes.reverse ++ List.replicate count 0)).toNat := hd
    _ = (BitVec.fromLEBytes bytes.reverse).toNat := zero_padding_le_val bytes.reverse count
    _ = _ := by simp only [BitVec.fromBEBytes, BitVec.toNat_cast]
end Byte128Math
#print axioms Byte128Math.signed_low_bits

#print axioms Byte128Math.zero_padding_le

#print axioms Byte128Math.zero_padding_be_val
