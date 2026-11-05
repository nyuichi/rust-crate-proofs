import ReadVariableProofs
open Aeneas Aeneas.Std
namespace U24Proofs
private theorem be_padding (a b c : BitVec 8) :
    BitVec.fromBEBytes [0, 0, 0, 0, 0, a, b, c] =
      (BitVec.fromBEBytes [a, b, c]).setWidth 64 := by
  change BitVec.fromLEBytes [c, b, a, 0, 0, 0, 0, 0] =
    (BitVec.fromLEBytes [c, b, a]).setWidth 64
  simp only [BitVec.fromLEBytes, List.length_cons, List.length_nil]
  apply BitVec.eq_of_getElem_eq
  intro i hi
  simp [BitVec.getLsbD_shiftLeft, BitVec.getLsbD_setWidth]
  all_goals by_cases h8 : i < 8
  all_goals by_cases h16 : i < 16
  all_goals by_cases h24 : i < 24
  all_goals simp (disch := omega) [*, BitVec.getLsbD_of_ge]
  all_goals have h1 : i - 8 < 56 := by omega
  all_goals have h2 : i - 8 < 24 := by omega
  all_goals have h3 : i - 8 < 16 := by omega
  all_goals have h4 : i - 8 - 8 < 56 := by omega
  all_goals have h5 : i - 8 - 8 < 16 := by omega
  all_goals simp_all
#print axioms be_padding
private theorem le_padding (a b c : BitVec 8) :
    BitVec.fromLEBytes [a, b, c, 0, 0, 0, 0, 0] =
      (BitVec.fromLEBytes [a, b, c]).setWidth 64 := by
  exact be_padding c b a

theorem be_three_decode (a b c : U8) :
    (BitVec.fromBEBytes ([0#u8, 0#u8, 0#u8, 0#u8, 0#u8, a, b, c].map U8.bv)).toNat =
      (BitVec.fromBEBytes ([a, b, c].map U8.bv)).toNat := by
  rw [show ([0#u8, 0#u8, 0#u8, 0#u8, 0#u8, a, b, c].map U8.bv) =
    [0, 0, 0, 0, 0, a.bv, b.bv, c.bv] from rfl]
  rw [be_padding]
  exact BitVec.toNat_setWidth_of_le (by simp)

#print axioms be_three_decode
theorem le_three_decode (a b c : U8) :
    (BitVec.fromLEBytes ([a, b, c, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8].map U8.bv)).toNat =
      (BitVec.fromLEBytes ([a, b, c].map U8.bv)).toNat := by
  rw [show ([a, b, c, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8].map U8.bv) =
    [a.bv, b.bv, c.bv, 0, 0, 0, 0, 0] from rfl]
  rw [le_padding]
  exact BitVec.toNat_setWidth_of_le (by simp)
#print axioms le_three_decode
end U24Proofs
