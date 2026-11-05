import U24WrapperProofs
import U24Decode24
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite_u24_roots.byteorder_lite
namespace U24Proofs
theorem be_read_u24_exact (buf : Slice U8) (enough : 3 ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 3).map U8.bv)).toNat ∧
        r.val < 2^24 ⦄ := by
  have hlen : (buf.val.take 3).length = 3 := by simp [List.length_take, Nat.min_eq_left enough]
  obtain ⟨a, b, c, hp⟩ := List.length_eq_three.mp hlen
  have hbound : (BitVec.fromBEBytes ([a, b, c].map U8.bv)).toNat < 2^24 := by
    have h := (BitVec.fromBEBytes ([a, b, c].map U8.bv)).isLt
    simpa using h
  step with be_read_u24_spec
  have hpad : List.replicate 5 (0#u8) ++ buf.val.take 3 = [0#u8, 0#u8, 0#u8, 0#u8, 0#u8, a, b, c] := by rw [hp]; rfl
  have hd := congrArg (fun bytes : List U8 => (BitVec.fromBEBytes (bytes.map U8.bv)).toNat) hpad
  have hb := congrArg (fun bytes : List U8 => (BitVec.fromBEBytes (bytes.map U8.bv)).toNat) hp
  have hsmall : (BitVec.fromBEBytes ([a, b, c].map U8.bv)).toNat < 2^32 := by omega
  have heq := hd.trans (be_three_decode a b c)
  change r.val = _ ∧ r.val < 2^24
  rw [r_post, heq, hb, Nat.mod_eq_of_lt hsmall]
  exact ⟨rfl, hbound⟩
theorem le_read_u24_exact (buf : Slice U8) (enough : 3 ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 3).map U8.bv)).toNat ∧
        r.val < 2^24 ⦄ := by
  have hlen : (buf.val.take 3).length = 3 := by simp [List.length_take, Nat.min_eq_left enough]
  obtain ⟨a, b, c, hp⟩ := List.length_eq_three.mp hlen
  have hbound : (BitVec.fromLEBytes ([a, b, c].map U8.bv)).toNat < 2^24 := by
    have h := (BitVec.fromLEBytes ([a, b, c].map U8.bv)).isLt
    simpa using h
  step with le_read_u24_spec
  have hpad : buf.val.take 3 ++ List.replicate 5 (0#u8) = [a, b, c, 0#u8, 0#u8, 0#u8, 0#u8, 0#u8] := by rw [hp]; rfl
  have hd := congrArg (fun bytes : List U8 => (BitVec.fromLEBytes (bytes.map U8.bv)).toNat) hpad
  have hb := congrArg (fun bytes : List U8 => (BitVec.fromLEBytes (bytes.map U8.bv)).toNat) hp
  have hsmall : (BitVec.fromLEBytes ([a, b, c].map U8.bv)).toNat < 2^32 := by omega
  have heq := hd.trans (le_three_decode a b c)
  change r.val = _ ∧ r.val < 2^24
  rw [r_post, heq, hb, Nat.mod_eq_of_lt hsmall]
  exact ⟨rfl, hbound⟩
end U24Proofs

#print axioms U24Proofs.be_read_u24_exact
#print axioms U24Proofs.le_read_u24_exact

namespace U24Proofs
theorem be_read_u24_contract (buf : Slice U8) :
    (3 ≤ buf.length → BigEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 3).map U8.bv)).toNat ∧
        r.val < 2^24 ⦄) ∧
    (buf.length < 3 → BigEndian.Insts.Byteorder_liteByteOrder.read_u24 buf = .fail .assertionFailure) := by
  exact ⟨fun h => be_read_u24_exact buf h, fun h => be_read_u24_short buf h⟩
theorem le_read_u24_contract (buf : Slice U8) :
    (3 ≤ buf.length → LittleEndian.Insts.Byteorder_liteByteOrder.read_u24 buf
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 3).map U8.bv)).toNat ∧
        r.val < 2^24 ⦄) ∧
    (buf.length < 3 → LittleEndian.Insts.Byteorder_liteByteOrder.read_u24 buf = .fail .assertionFailure) := by
  exact ⟨fun h => le_read_u24_exact buf h, fun h => le_read_u24_short buf h⟩
end U24Proofs
#print axioms U24Proofs.be_read_u24_contract
#print axioms U24Proofs.le_read_u24_contract
