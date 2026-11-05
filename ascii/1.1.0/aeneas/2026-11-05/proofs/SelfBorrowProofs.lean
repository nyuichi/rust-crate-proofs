import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace SelfBorrowProofs
 theorem as_mut_self_exact (s : ascii.ascii_str.AsciiStr) :
 ascii.ascii_str.AsciiStr.Insts.CoreConvertAsMutAsciiStr.as_mut s =
 .ok (s, fun replacement => replacement) := by rfl
 theorem same_length_writeback (s replacement : ascii.ascii_str.AsciiStr)
 (same_length : replacement.slice.length = s.slice.length) :
 ∃ view back,
 ascii.ascii_str.AsciiStr.Insts.CoreConvertAsMutAsciiStr.as_mut s = .ok (view,back) ∧
 view = s ∧ (back replacement).slice.val = replacement.slice.val ∧
 (back replacement).slice.length = s.slice.length := by
   exact ⟨s, (fun replacement => replacement), rfl, rfl, rfl, same_length⟩
#print axioms as_mut_self_exact
#print axioms same_length_writeback
end SelfBorrowProofs
