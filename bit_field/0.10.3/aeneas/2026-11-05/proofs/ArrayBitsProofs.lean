import BitArrayGenerated.BitField
open Aeneas Aeneas.Std Result Std.WP
open bit_field
namespace ArrayBitsProofs
theorem get_bit_spec {T : Type} (inst : BitField T) (xs : Slice T)
    (width bit : Usize) (hw : inst.BIT_LENGTH = .ok width) (positive : 0 < width.val)
    (indexBound : bit.val / width.val < xs.length) (observe : T → Nat → Bool)
    (callback : ∀ (x : T) (j : Usize), j.val < width.val →
      inst.get_bit x j ⦃ r => r = observe x j.val ⦄) :
    Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit
      ⦃ r => r = observe xs.val[bit.val / width.val] (bit.val % width.val) ⦄ := by
  unfold Slice.Insts.Bit_fieldBitArray.get_bit
  rw [hw]
  step
  step
  step
  step with callback
  simp_all
end ArrayBitsProofs
#print axioms ArrayBitsProofs.get_bit_spec
namespace ArrayBitsProofs
theorem set_bit_spec {T : Type} (inst : BitField T) (xs : Slice T)
    (width bit : Usize) (value : Bool) (hw : inst.BIT_LENGTH = .ok width)
    (positive : 0 < width.val) (indexBound : bit.val / width.val < xs.length)
    (update : T → Nat → Bool → T)
    (callback : ∀ (x : T) (j : Usize) (v : Bool), j.val < width.val →
      inst.set_bit x j v ⦃ r => r.2 r.1 = update x j.val v ⦄) :
    Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value
      ⦃ r => r.val = xs.val.set (bit.val / width.val)
        (update xs.val[bit.val / width.val] (bit.val % width.val) value) ⦄ := by
  unfold Slice.Insts.Bit_fieldBitArray.set_bit
  rw [hw]
  step
  step
  step
  step with callback
  simp_all
end ArrayBitsProofs
#print axioms ArrayBitsProofs.set_bit_spec
namespace ArrayBitsProofs
theorem get_bit_zero_width {T : Type} (inst : BitField T) (xs : Slice T)
    (bit : Usize) (hw : inst.BIT_LENGTH = .ok 0#usize) :
    Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit = .fail .divisionByZero := by
  unfold Slice.Insts.Bit_fieldBitArray.get_bit
  rw [hw]
  simp only [Std.bind_ok]
  change (do
    let i ← UScalar.div bit 0#usize
    let j ← bit % 0#usize
    let x ← Slice.index_usize xs i
    inst.get_bit x j) = _
  simp [UScalar.div]

theorem set_bit_zero_width {T : Type} (inst : BitField T) (xs : Slice T)
    (bit : Usize) (value : Bool) (hw : inst.BIT_LENGTH = .ok 0#usize) :
    Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value = .fail .divisionByZero := by
  unfold Slice.Insts.Bit_fieldBitArray.set_bit
  rw [hw]
  simp only [Std.bind_ok]
  simp only [HDiv.hDiv, UScalar.div]
  simp
end ArrayBitsProofs
#print axioms ArrayBitsProofs.get_bit_zero_width
#print axioms ArrayBitsProofs.set_bit_zero_width
namespace ArrayBitsProofs
private theorem quotient_index_fails {T : Type} (xs : Slice T) (bit width : Usize)
    (outside : xs.length ≤ bit.val / width.val) :
    Slice.index_usize xs (⟨(UScalar.bv bit).udiv (UScalar.bv width)⟩ : Usize) =
      .fail .arrayOutOfBounds := by
  unfold Slice.index_usize
  simp only [Slice.getElem?_Usize_eq]
  rw [List.getElem?_eq_none (by simpa only [Slice.length, UScalar.val, BitVec.udiv_eq, BitVec.toNat_udiv] using outside)]

private theorem nonzero_width (width : Usize) (positive : 0 < width.val) :
    UScalar.bv width ≠ 0 := by
  intro hz
  have hval := congrArg BitVec.toNat hz
  change width.val = (BitVec.ofNat UScalarTy.Usize.numBits 0).toNat at hval
  simp only [BitVec.toNat_ofNat, Nat.zero_mod] at hval
  omega

theorem get_bit_outside {T : Type} (inst : BitField T) (xs : Slice T)
    (bit width : Usize) (hw : inst.BIT_LENGTH = .ok width) (positive : 0 < width.val)
    (outside : xs.length ≤ bit.val / width.val) :
    Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit = .fail .arrayOutOfBounds := by
  have hn := nonzero_width width positive
  have hnv : width.val ≠ 0 := by omega
  unfold Slice.Insts.Bit_fieldBitArray.get_bit
  rw [hw]
  simp only [Std.bind_ok, HDiv.hDiv, HMod.hMod, UScalar.div, UScalar.rem]
  simp only [bne_iff_ne]
  split
  · simp only [Std.bind_ok, quotient_index_fails xs bit width outside, Std.bind_fail]
  · contradiction

theorem set_bit_outside {T : Type} (inst : BitField T) (xs : Slice T)
    (bit width : Usize) (value : Bool) (hw : inst.BIT_LENGTH = .ok width)
    (positive : 0 < width.val) (outside : xs.length ≤ bit.val / width.val) :
    Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value = .fail .arrayOutOfBounds := by
  have hn := nonzero_width width positive
  have hnv : width.val ≠ 0 := by omega
  unfold Slice.Insts.Bit_fieldBitArray.set_bit
  rw [hw]
  simp only [Std.bind_ok, HDiv.hDiv, HMod.hMod, UScalar.div, UScalar.rem]
  simp only [bne_iff_ne]
  split
  · simp only [Std.bind_ok, Slice.index_mut_usize, quotient_index_fails xs bit width outside, Std.bind_fail, bind_tc_fail]
  · contradiction
end ArrayBitsProofs
#print axioms ArrayBitsProofs.get_bit_outside
#print axioms ArrayBitsProofs.set_bit_outside
namespace ArrayBitsProofs
theorem get_bit_contract {T : Type} (inst : BitField T) (xs : Slice T)
    (width bit : Usize) (hw : inst.BIT_LENGTH = .ok width) (observe : T → Nat → Bool)
    (callback : ∀ (x : T) (j : Usize), j.val < width.val →
      inst.get_bit x j ⦃ r => r = observe x j.val ⦄) :
    (width.val = 0 → Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit = .fail .divisionByZero) ∧
    (0 < width.val →
      (xs.length ≤ bit.val / width.val →
        Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit = .fail .arrayOutOfBounds) ∧
      (∀ indexBound : bit.val / width.val < xs.length,
        Slice.Insts.Bit_fieldBitArray.get_bit inst xs bit
          ⦃ r => r = observe xs.val[bit.val / width.val] (bit.val % width.val) ⦄)) := by
  refine ⟨?_, fun positive => ⟨?_, ?_⟩⟩
  · intro zero
    have hw0 : inst.BIT_LENGTH = .ok 0#usize := by
      rw [hw]
      congr 1
      scalar_tac
    exact get_bit_zero_width inst xs bit hw0
  · intro outside
    exact get_bit_outside inst xs bit width hw positive outside
  · intro inside
    exact get_bit_spec inst xs width bit hw positive inside observe callback

theorem set_bit_contract {T : Type} (inst : BitField T) (xs : Slice T)
    (width bit : Usize) (value : Bool) (hw : inst.BIT_LENGTH = .ok width)
    (update : T → Nat → Bool → T)
    (callback : ∀ (x : T) (j : Usize) (v : Bool), j.val < width.val →
      inst.set_bit x j v ⦃ r => r.2 r.1 = update x j.val v ⦄) :
    (width.val = 0 → Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value = .fail .divisionByZero) ∧
    (0 < width.val →
      (xs.length ≤ bit.val / width.val →
        Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value = .fail .arrayOutOfBounds) ∧
      (∀ indexBound : bit.val / width.val < xs.length,
        Slice.Insts.Bit_fieldBitArray.set_bit inst xs bit value
          ⦃ r => r.val = xs.val.set (bit.val / width.val)
            (update xs.val[bit.val / width.val] (bit.val % width.val) value) ⦄)) := by
  refine ⟨?_, fun positive => ⟨?_, ?_⟩⟩
  · intro zero
    have hw0 : inst.BIT_LENGTH = .ok 0#usize := by
      rw [hw]
      congr 1
      scalar_tac
    exact set_bit_zero_width inst xs bit value hw0
  · intro outside
    exact set_bit_outside inst xs bit width value hw positive outside
  · intro inside
    exact set_bit_spec inst xs width bit value hw positive inside update callback
end ArrayBitsProofs
#print axioms ArrayBitsProofs.get_bit_contract
#print axioms ArrayBitsProofs.set_bit_contract
