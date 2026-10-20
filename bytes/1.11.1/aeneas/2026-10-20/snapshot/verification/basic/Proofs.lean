import Generated

open Aeneas Aeneas.Std Result ControlFlow Error

namespace BytesProofs.Basic

/-!
These specifications refer directly to the functions emitted from the adapted
Rust crate.  They quantify over every Lean value of the generated input type;
they do not assume Rust-side well-formedness invariants.
-/

theorem bytes_len_returns_stored_length (self : _root_.bytes.bytes.Bytes) :
    _root_.bytes.bytes.Bytes.impl.len self = ok self.len := by
  rfl

theorem bytes_is_empty_returns_length_test (self : _root_.bytes.bytes.Bytes) :
    _root_.bytes.bytes.Bytes.is_empty self =
      ok (decide (self.len = 0#usize)) := by
  rfl

theorem bytes_is_empty_iff_length_zero (self : _root_.bytes.bytes.Bytes) :
    (_root_.bytes.bytes.Bytes.is_empty self = ok true) ↔ self.len = 0#usize := by
  simp [_root_.bytes.bytes.Bytes.is_empty]

theorem bytes_is_empty_iff_len_returns_zero (self : _root_.bytes.bytes.Bytes) :
    (_root_.bytes.bytes.Bytes.is_empty self = ok true) ↔
      _root_.bytes.bytes.Bytes.impl.len self = ok 0#usize := by
  simp [_root_.bytes.bytes.Bytes.is_empty, _root_.bytes.bytes.Bytes.impl.len]

theorem bytes_mut_len_returns_stored_length (self : _root_.bytes.bytes_mut.BytesMut) :
    _root_.bytes.bytes_mut.BytesMut.impl.len self = ok self.len := by
  rfl

theorem bytes_mut_capacity_returns_stored_capacity
    (self : _root_.bytes.bytes_mut.BytesMut) :
    _root_.bytes.bytes_mut.BytesMut.capacity self = ok self.cap := by
  rfl

theorem bytes_mut_is_empty_returns_length_test
    (self : _root_.bytes.bytes_mut.BytesMut) :
    _root_.bytes.bytes_mut.BytesMut.is_empty self =
      ok (decide (self.len = 0#usize)) := by
  rfl

theorem bytes_mut_is_empty_iff_length_zero
    (self : _root_.bytes.bytes_mut.BytesMut) :
    (_root_.bytes.bytes_mut.BytesMut.is_empty self = ok true) ↔ self.len = 0#usize := by
  simp [_root_.bytes.bytes_mut.BytesMut.is_empty]

theorem bytes_mut_is_empty_iff_len_returns_zero
    (self : _root_.bytes.bytes_mut.BytesMut) :
    (_root_.bytes.bytes_mut.BytesMut.is_empty self = ok true) ↔
      _root_.bytes.bytes_mut.BytesMut.impl.len self = ok 0#usize := by
  simp [_root_.bytes.bytes_mut.BytesMut.is_empty, _root_.bytes.bytes_mut.BytesMut.impl.len]

#print axioms bytes_len_returns_stored_length
#print axioms bytes_is_empty_returns_length_test
#print axioms bytes_is_empty_iff_length_zero
#print axioms bytes_is_empty_iff_len_returns_zero
#print axioms bytes_mut_len_returns_stored_length
#print axioms bytes_mut_capacity_returns_stored_capacity
#print axioms bytes_mut_is_empty_returns_length_test
#print axioms bytes_mut_is_empty_iff_length_zero
#print axioms bytes_mut_is_empty_iff_len_returns_zero

end BytesProofs.Basic
