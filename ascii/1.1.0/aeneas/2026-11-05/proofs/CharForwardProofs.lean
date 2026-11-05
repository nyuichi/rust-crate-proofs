import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace CharForwardProofs
theorem from_ascii_exact {C : Type} (inst : ascii.ascii_char.ToAsciiChar C) (ch : C) :
 ascii.ascii_char.AsciiChar.from_ascii inst ch = inst.to_ascii_char ch := by rfl
theorem from_ascii_return {C : Type} (inst : ascii.ascii_char.ToAsciiChar C) (ch : C)
 (r : Result (core.result.Result ascii.ascii_char.AsciiChar ascii.ascii_char.ToAsciiCharError))
 (h : inst.to_ascii_char ch = r) : ascii.ascii_char.AsciiChar.from_ascii inst ch = r := by exact h
#print axioms from_ascii_exact
#print axioms from_ascii_return
end CharForwardProofs
