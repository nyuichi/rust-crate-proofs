import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace StringBorrowViewProofs
theorem as_ref_slice_exact (s : ascii.ascii_string.AsciiString) :
 ascii.ascii_string.AsciiString.Insts.CoreConvertAsRefSliceAsciiChar.as_ref s = .ok s.vec.slice := by
 unfold ascii.ascii_string.AsciiString.Insts.CoreConvertAsRefSliceAsciiChar.as_ref
 apply congrArg Result.ok
 apply (Slice.eq_iff _ _).mpr
 simp [alloc.vec.Vec.deref, alloc.vec.Vec.val]

theorem as_mut_slice_contract (s : ascii.ascii_string.AsciiString) :
 ∃ view back,
   ascii.ascii_string.AsciiString.Insts.CoreConvertAsMutSliceAsciiChar.as_mut s = .ok (view,back) ∧
   view.val = s.vec.val ∧
   (∀ replacement : Slice ascii.ascii_char.AsciiChar,
    replacement.length = s.vec.length →
    (back replacement).vec.val = replacement.val ∧
    (back replacement).vec.length = s.vec.length) := by
 refine ⟨s.vec.slice, (fun replacement => {vec := {slice := replacement}}), ?_, rfl, ?_⟩
 · simp [ascii.ascii_string.AsciiString.Insts.CoreConvertAsMutSliceAsciiChar.as_mut,
     alloc.vec.Vec.deref_mut, lift, Std.bind_ok]
 · intro replacement same_length
   exact ⟨rfl, same_length⟩
end StringBorrowViewProofs
#print axioms StringBorrowViewProofs.as_ref_slice_exact
#print axioms StringBorrowViewProofs.as_mut_slice_contract
