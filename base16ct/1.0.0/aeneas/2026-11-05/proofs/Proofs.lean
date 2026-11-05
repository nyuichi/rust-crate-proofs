import Generated.Base16ct
open Aeneas Aeneas.Std
namespace Proofs
theorem encoded_len_spec (s : Slice U8) (h : (Slice.len s).val * 2 ≤ Usize.max) :
 base16ct.encoded_len s ⦃ n => n.val = (Slice.len s).val * 2 ⦄ := by
 unfold base16ct.encoded_len
 step
 scalar_tac

theorem decoded_len_spec (s : Slice U8) :
 base16ct.decoded_len s ⦃ r => match r with | .Ok n => (Slice.len s).val % 2 = 0 ∧ n.val = (Slice.len s).val / 2 | .Err e => e = .InvalidLength ∧ (Slice.len s).val % 2 ≠ 0 ⦄ := by
 unfold base16ct.decoded_len
 simp [lift]
 have hbit : (Slice.len s &&& 1#usize = 0#usize) ↔ (Slice.len s).val % 2 = 0 := by
  rw [UScalar.eq_equiv]
  simp
 split
 · rename_i h
   have hp := hbit.mp h
   step
   simp_all
 · rename_i h
   simp_all
end Proofs
#print axioms Proofs.encoded_len_spec
#print axioms Proofs.decoded_len_spec
