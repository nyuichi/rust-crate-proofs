import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace BorrowViewProofs
theorem as_ref_slice_exact (s : ascii.ascii_str.AsciiStr) :
 ascii.ascii_str.AsciiStr.Insts.CoreConvertAsRefSliceAsciiChar.as_ref s = .ok s.slice := by rfl

theorem as_ref_self_exact (s : ascii.ascii_str.AsciiStr) :
 ascii.ascii_str.AsciiStr.Insts.CoreConvertAsRefAsciiStr.as_ref s = .ok s := by rfl

/-- Normal return, unchanged initial view, and exact valid same-length writeback. -/
theorem as_mut_slice_contract (s : ascii.ascii_str.AsciiStr) :
 ∃ view back,
   ascii.ascii_str.AsciiStr.Insts.CoreConvertAsMutSliceAsciiChar.as_mut s = .ok (view,back) ∧
   view = s.slice ∧
   (∀ replacement : Slice ascii.ascii_char.AsciiChar,
    replacement.length = s.slice.length →
    (back replacement).slice.val = replacement.val ∧
    (back replacement).slice.length = s.slice.length) := by
 refine ⟨s.slice, (fun replacement => {slice := replacement}), rfl, rfl, ?_⟩
 intro replacement same_length
 exact ⟨rfl, same_length⟩
end BorrowViewProofs
#print axioms BorrowViewProofs.as_ref_slice_exact
#print axioms BorrowViewProofs.as_ref_self_exact
#print axioms BorrowViewProofs.as_mut_slice_contract
