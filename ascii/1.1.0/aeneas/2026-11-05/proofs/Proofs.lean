import Generated.Ascii
set_option linter.internalConstructors false
open Aeneas Aeneas.Std
namespace Proofs
open ascii.ascii_char
theorem blank_spec (c : AsciiChar) : c.is_ascii_blank = .ok (match c with | .Space | .Tab => true | _ => false) := by cases c <;> simp [AsciiChar.is_ascii_blank, UScalar.cast, UScalar.tryMk, lift, AsciiChar.read_discriminant] <;> decide
theorem is_ascii_digit_spec (c : AsciiChar) : c.is_ascii_digit = .ok (match c with | ._0 | ._1 | ._2 | ._3 | ._4 | ._5 | ._6 | ._7 | ._8 | ._9 => true | _ => false) := by cases c <;> simp [AsciiChar.is_ascii_digit, AsciiChar.is_ascii_blank, UScalar.cast, lift, AsciiChar.read_discriminant] <;> decide
theorem is_ascii_whitespace_spec (c : AsciiChar) : c.is_ascii_whitespace = .ok (match c with | .Space | .Tab | .LineFeed | .CarriageReturn | .FF => true | _ => false) := by cases c <;> simp [AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_blank, UScalar.cast, lift, AsciiChar.read_discriminant] <;> decide
theorem is_ascii_control_spec (c : AsciiChar) : c.is_ascii_control = .ok (match c with | .Null | .SOH | .SOX | .ETX | .EOT | .ENQ | .ACK | .Bell | .BackSpace | .Tab | .LineFeed | .VT | .FF | .CarriageReturn | .SI | .SO | .DLE | .DC1 | .DC2 | .DC3 | .DC4 | .NAK | .SYN | .ETB | .CAN | .EM | .SUB | .ESC | .FS | .GS | .RS | .US | .DEL => true | _ => false) := by cases c <;> simp [AsciiChar.is_ascii_control, AsciiChar.is_ascii_blank, UScalar.cast, lift, AsciiChar.read_discriminant] <;> decide
end Proofs
#print axioms Proofs.blank_spec
#print axioms Proofs.is_ascii_digit_spec
#print axioms Proofs.is_ascii_whitespace_spec
#print axioms Proofs.is_ascii_control_spec
