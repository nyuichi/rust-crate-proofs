import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace ErrorDescriptionProofs
theorem char_description_exact (e : ascii.ascii_char.ToAsciiCharError) :
 ascii.ascii_char.ToAsciiCharError.Insts.CoreErrorError.description e =
 .ok (toStr "not an ASCII character") := by
 unfold ascii.ascii_char.ToAsciiCharError.Insts.CoreErrorError.description ascii.ascii_char.ERRORMSG_CHAR
 rfl
theorem str_description_exact (e : ascii.ascii_str.AsAsciiStrError) :
 ascii.ascii_str.AsAsciiStrError.Insts.CoreErrorError.description e =
 .ok (toStr "one or more bytes are not ASCII") := by
 unfold ascii.ascii_str.AsAsciiStrError.Insts.CoreErrorError.description ascii.ascii_str.ERRORMSG_STR
 rfl
#print axioms char_description_exact
#print axioms str_description_exact
end ErrorDescriptionProofs
