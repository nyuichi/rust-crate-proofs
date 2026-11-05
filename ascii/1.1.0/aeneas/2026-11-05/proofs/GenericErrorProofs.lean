import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace GenericErrorProofs
theorem description_exact {O : Type} (e : ascii.ascii_string.FromAsciiError O) :
 ascii.ascii_string.FromAsciiError.Insts.CoreErrorError.description e =
 .ok (toStr "one or more bytes are not ASCII") := by
 unfold ascii.ascii_string.FromAsciiError.Insts.CoreErrorError.description
 unfold ascii.ascii_str.AsAsciiStrError.Insts.CoreErrorError.description ascii.ascii_str.ERRORMSG_STR
 rfl
#print axioms description_exact
end GenericErrorProofs
