import Generated.AsciiFieldSelectors
open Aeneas Aeneas.Std Result
namespace FieldFromProofs
open ascii_field_selectors
theorem shared_original_exact (s : ascii.ascii_str.AsciiStr) :
 SharedASliceAsciiChar.Insts.CoreConvertFromSharedAAsciiStr.from s = .ok s.slice := by rfl
theorem mutable_original_exact (s : ascii.ascii_str.AsciiStr) :
 MutASliceAsciiChar.Insts.CoreConvertFromMutAAsciiStr.from s =
 .ok (s.slice, fun replacement => {slice := replacement}) := by rfl
theorem mutable_same_length_writeback (s : ascii.ascii_str.AsciiStr)
 (replacement : Slice ascii.ascii_char.AsciiChar) (same_length : replacement.length = s.slice.length) :
 ∃ view back, MutASliceAsciiChar.Insts.CoreConvertFromMutAAsciiStr.from s = .ok (view,back) ∧
 view = s.slice ∧ (back replacement).slice.val = replacement.val ∧
 (back replacement).slice.length = s.slice.length := by
 exact ⟨s.slice,(fun replacement => {slice := replacement}),rfl,rfl,rfl,same_length⟩
theorem shared_wrapper_route (s : ascii.ascii_str.AsciiStr) :
 shared s = SharedASliceAsciiChar.Insts.CoreConvertFromSharedAAsciiStr.from s := by rfl
theorem mutable_wrapper_route (s : ascii.ascii_str.AsciiStr) :
 mutable s = MutASliceAsciiChar.Insts.CoreConvertFromMutAAsciiStr.from s := by rfl
#print axioms shared_original_exact
#print axioms mutable_original_exact
#print axioms mutable_same_length_writeback
#print axioms shared_wrapper_route
#print axioms mutable_wrapper_route
end FieldFromProofs
