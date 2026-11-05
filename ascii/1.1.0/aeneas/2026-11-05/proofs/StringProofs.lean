import Generated.Ascii
open Aeneas Aeneas.Std
set_option maxHeartbeats 0
namespace StringProofs
open ascii.ascii_string ascii.ascii_char

theorem new_spec : AsciiString.new ⦃ s => s.vec.val = [] ⦄ := by
 simp [AsciiString.new, alloc.vec.Vec.new]

theorem len_spec (s : AsciiString) : s.len ⦃ n => n.val = s.vec.val.length ⦄ := by
 simp [AsciiString.len]

theorem is_empty_spec (s : AsciiString) : s.is_empty = .ok (decide (s.vec.val = [])) := by
 simp [AsciiString.is_empty, AsciiString.len, UScalar.eq_equiv]

theorem push_spec (s : AsciiString) (c : AsciiChar) (h : s.vec.val.length < Usize.max) :
 s.push c ⦃ s' => s'.vec.val = s.vec.val ++ [c] ⦄ := by
 unfold AsciiString.push
 step
 simp_all

theorem push_maximum_spec (s : AsciiString) (c : AsciiChar) (h : s.vec.val.length = Usize.max) :
 s.push c = .fail .maximumSizeExceeded := by
 unfold AsciiString.push alloc.vec.Vec.push
 have hu : U32.max ≤ Usize.max := by scalar_tac
 simp only [h]
 split <;> simp_all <;> scalar_tac
theorem push_full_spec (s : AsciiString) (c : AsciiChar) :
 (if s.vec.val.length < Usize.max then s.push c ⦃ s' => s'.vec.val = s.vec.val ++ [c] ⦄
 else s.push c = .fail .maximumSizeExceeded) := by
 split
 · exact push_spec s c (by assumption)
 · apply push_maximum_spec s c
   have := s.vec.property
   omega
end StringProofs
#print axioms StringProofs.new_spec
#print axioms StringProofs.len_spec
#print axioms StringProofs.is_empty_spec
#print axioms StringProofs.push_spec
#print axioms StringProofs.push_maximum_spec

#print axioms StringProofs.push_full_spec
