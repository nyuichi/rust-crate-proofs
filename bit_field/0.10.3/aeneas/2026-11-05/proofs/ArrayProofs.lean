import BitArrayGenerated.BitField
open Aeneas Aeneas.Std Result Std.WP
open bit_field
namespace ArrayProofs
theorem bit_length_spec {T : Type} (inst : BitField T) (xs : Slice T)
    (width : Usize) (hw : inst.BIT_LENGTH = .ok width)
    (bound : xs.length * width.val ≤ Usize.max) :
    Slice.Insts.Bit_fieldBitArray.bit_length inst xs
      ⦃ r => r.val = xs.length * width.val ⦄ := by
  unfold Slice.Insts.Bit_fieldBitArray.bit_length
  rw [hw]
  step
  scalar_tac
end ArrayProofs
#print axioms ArrayProofs.bit_length_spec
namespace ArrayProofs
theorem bit_length_overflow {T : Type} (inst : BitField T) (xs : Slice T)
    (width : Usize) (hw : inst.BIT_LENGTH = .ok width)
    (overflow : Usize.max < xs.length * width.val) :
    Slice.Insts.Bit_fieldBitArray.bit_length inst xs = .fail .integerOverflow := by
  unfold Slice.Insts.Bit_fieldBitArray.bit_length
  rw [hw]
  simp only [Std.bind_ok, bind_tc_ok]
  change UScalar.mul (Slice.len xs) width = _
  have bad : ¬ UScalar.check_bounds .Usize ((Slice.len xs).val * width.val) := by
    intro hb
    have := UScalar.check_bounds_imp_inBounds hb
    simp only [Slice.len_val] at this
    scalar_tac
  simp only [UScalar.mul, UScalar.tryMk, UScalar.tryMkOpt, bad, ↓reduceDIte, ofOption]
  rfl

theorem bit_length_callback_failure {T : Type} (inst : BitField T) (xs : Slice T)
    (e : Error) (hw : inst.BIT_LENGTH = .fail e) :
    Slice.Insts.Bit_fieldBitArray.bit_length inst xs = .fail e := by
  simp only [Slice.Insts.Bit_fieldBitArray.bit_length, hw, Std.bind_fail]
end ArrayProofs
#print axioms ArrayProofs.bit_length_overflow
#print axioms ArrayProofs.bit_length_callback_failure
namespace ArrayProofs
theorem bit_length_contract {T : Type} (inst : BitField T) (xs : Slice T)
    (width : Usize) (hw : inst.BIT_LENGTH = .ok width) :
    (xs.length * width.val ≤ Usize.max →
      Slice.Insts.Bit_fieldBitArray.bit_length inst xs
        ⦃ r => r.val = xs.length * width.val ⦄) ∧
    (Usize.max < xs.length * width.val →
      Slice.Insts.Bit_fieldBitArray.bit_length inst xs = .fail .integerOverflow) :=
  ⟨fun h => bit_length_spec inst xs width hw h,
   fun h => bit_length_overflow inst xs width hw h⟩
end ArrayProofs
#print axioms ArrayProofs.bit_length_contract
