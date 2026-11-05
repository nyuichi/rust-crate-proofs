import BitField.Funs
open Aeneas Aeneas.Std Result Std.WP
open bit_field
namespace Proofs
set_option linter.unusedSimpArgs false
theorem get_bit_spec (x : U8) (bit : Usize) (h : bit.val < 8) :
  U8.Insts.Bit_fieldBitField.get_bit x bit
    ⦃ b => b = x.bv.getLsbD bit.val ⦄ := by
  unfold U8.Insts.Bit_fieldBitField.get_bit
  simp only [U8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_of, ↓reduceIte, Std.bind_ok, bind_tc_ok]
  step
  step
  step
  simp [lift, UScalar.bv_and, UScalar.eq_equiv, i1_post1]
  change (!(decide ((x.bv &&& i1.bv) = (0 : BitVec 8)))) = x.bv.getLsbD bit.val
  change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
  rw [i1_post1]
  have hm : ((1 : BitVec 8) <<< bit.val) = BitVec.twoPow 8 bit.val := (BitVec.twoPow_eq 8 bit.val).symm
  rw [hm, BitVec.and_twoPow]
  have hn : BitVec.twoPow 8 bit.val ≠ 0 := by
    intro heq
    have hz := congrArg (fun (v : BitVec 8) => v.getLsbD bit.val) heq
    simp only [BitVec.getLsbD_twoPow, BitVec.getLsbD_zero, h, decide_true, beq_self_eq_true, Bool.and_self, decide_eq_true_eq, and_self] at hz
    have hz0 : (0 : BitVec 8).getLsbD bit.val = false := by simp
    rw [hz0] at hz
    contradiction
  cases he : x.bv.getLsbD bit.val
  · simp only [he, Bool.false_eq_true, ↓reduceIte, decide_true, Bool.not_true]
    rfl
  · simp only [he, ↓reduceIte, Bool.not_eq_true', decide_eq_false_iff_not]
    exact hn
def setWord (x : BitVec 8) (bit : Nat) (value : Bool) : BitVec 8 :=
  if value then x ||| ((1 : BitVec 8) <<< bit)
  else x &&& ~~~((1 : BitVec 8) <<< bit)

theorem set_bit_spec (x : U8) (bit : Usize) (value : Bool) (h : bit.val < 8) :
  U8.Insts.Bit_fieldBitField.set_bit x bit value
    ⦃ r => r.1.bv = setWord x.bv bit.val value ∧ ∀ y, r.2 y = y ⦄ := by
  unfold U8.Insts.Bit_fieldBitField.set_bit
  simp only [U8.Insts.Bit_fieldBitField.BIT_LENGTH, core.mem.size_of,
    Std.bind_ok, bind_tc_ok]
  step
  step
  cases value
  · simp only [Bool.false_eq_true, ↓reduceIte]
    step
    simp [lift, setWord]
    change x.bv &&& ~~~i1.bv = x.bv &&& ~~~((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]
  · simp only [↓reduceIte]
    step
    simp [lift, setWord]
    change x.bv ||| i1.bv = x.bv ||| ((1 : BitVec 8) <<< bit.val)
    change i1.bv = ((1 : BitVec 8) <<< bit.val) at i1_post1
    rw [i1_post1]
end Proofs
#print axioms Proofs.get_bit_spec
#print axioms Proofs.set_bit_spec
