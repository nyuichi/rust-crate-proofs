import Generated.Ascii
open Aeneas Aeneas.Std
namespace SliceProofs
open ascii.ascii_str ascii.ascii_char

theorem as_slice_spec (s : AsciiStr) : s.as_slice = .ok s.slice := by rfl

theorem as_mut_slice_spec (s : AsciiStr) : s.as_mut_slice ⦃ (view,back) =>
 view = s.slice ∧ ∀ replacement : Slice AsciiChar,
 replacement.val.length = s.slice.val.length →
 (back replacement).slice = replacement ∧
 (back replacement).slice.val.length = s.slice.val.length ⦄ := by
 simp [AsciiStr.as_mut_slice]

theorem len_spec (s : AsciiStr) : s.len ⦃ n => n.val = s.slice.val.length ⦄ := by
 simp [AsciiStr.len]

theorem is_empty_spec (s : AsciiStr) : s.is_empty = .ok (decide (s.slice.val = [])) := by
 simp [AsciiStr.is_empty, AsciiStr.len, UScalar.eq_equiv]
theorem as_ascii_str_identity_spec (s : AsciiStr) :
 AsciiStr.Insts.AsciiAscii_strAsAsciiStrAsciiChar.as_ascii_str s = .ok (.Ok s) := by rfl

theorem as_ascii_str_unchecked_identity_spec (s : AsciiStr) :
 AsciiStr.Insts.AsciiAscii_strAsAsciiStrAsciiChar.as_ascii_str_unchecked s = .ok s := by rfl

theorem as_mut_ascii_str_unchecked_identity_spec (s : AsciiStr) :
 AsciiStr.Insts.AsciiAscii_strAsMutAsciiStrAsciiChar.as_mut_ascii_str_unchecked s ⦃ (view,back) =>
 view = s ∧ ∀ replacement : AsciiStr,
 replacement.slice.val.length = s.slice.val.length →
 back replacement = replacement ∧ (back replacement).slice.val.length = s.slice.val.length ⦄ := by
 simp [AsciiStr.Insts.AsciiAscii_strAsMutAsciiStrAsciiChar.as_mut_ascii_str_unchecked]
end SliceProofs
#print axioms SliceProofs.as_slice_spec
#print axioms SliceProofs.as_mut_slice_spec
#print axioms SliceProofs.len_spec
#print axioms SliceProofs.is_empty_spec

#print axioms SliceProofs.as_ascii_str_identity_spec
#print axioms SliceProofs.as_ascii_str_unchecked_identity_spec
#print axioms SliceProofs.as_mut_ascii_str_unchecked_identity_spec
