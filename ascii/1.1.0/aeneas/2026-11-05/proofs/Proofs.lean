import Generated.Ascii
open Aeneas Aeneas.Std
set_option linter.internalConstructors false
set_option linter.unusedSimpArgs false
set_option linter.constructorNameAsVariable false
set_option maxHeartbeats 0
namespace Proofs
open ascii.ascii_char
def asciiCode : AsciiChar → Nat
 | .Null => 0
 | .SOH => 1
 | .SOX => 2
 | .ETX => 3
 | .EOT => 4
 | .ENQ => 5
 | .ACK => 6
 | .Bell => 7
 | .BackSpace => 8
 | .Tab => 9
 | .LineFeed => 10
 | .VT => 11
 | .FF => 12
 | .CarriageReturn => 13
 | .SI => 14
 | .SO => 15
 | .DLE => 16
 | .DC1 => 17
 | .DC2 => 18
 | .DC3 => 19
 | .DC4 => 20
 | .NAK => 21
 | .SYN => 22
 | .ETB => 23
 | .CAN => 24
 | .EM => 25
 | .SUB => 26
 | .ESC => 27
 | .FS => 28
 | .GS => 29
 | .RS => 30
 | .US => 31
 | .Space => 32
 | .Exclamation => 33
 | .Quotation => 34
 | .Hash => 35
 | .Dollar => 36
 | .Percent => 37
 | .Ampersand => 38
 | .Apostrophe => 39
 | .ParenOpen => 40
 | .ParenClose => 41
 | .Asterisk => 42
 | .Plus => 43
 | .Comma => 44
 | .Minus => 45
 | .Dot => 46
 | .Slash => 47
 | ._0 => 48
 | ._1 => 49
 | ._2 => 50
 | ._3 => 51
 | ._4 => 52
 | ._5 => 53
 | ._6 => 54
 | ._7 => 55
 | ._8 => 56
 | ._9 => 57
 | .Colon => 58
 | .Semicolon => 59
 | .LessThan => 60
 | .Equal => 61
 | .GreaterThan => 62
 | .Question => 63
 | .At => 64
 | .A => 65
 | .B => 66
 | .C => 67
 | .D => 68
 | .E => 69
 | .F => 70
 | .G => 71
 | .H => 72
 | .I => 73
 | .J => 74
 | .K => 75
 | .L => 76
 | .M => 77
 | .N => 78
 | .O => 79
 | .P => 80
 | .Q => 81
 | .R => 82
 | .S => 83
 | .T => 84
 | .U => 85
 | .V => 86
 | .W => 87
 | .X => 88
 | .Y => 89
 | .Z => 90
 | .BracketOpen => 91
 | .BackSlash => 92
 | .BracketClose => 93
 | .Caret => 94
 | .UnderScore => 95
 | .Grave => 96
 | .a => 97
 | .b => 98
 | .c => 99
 | .d => 100
 | .e => 101
 | .f => 102
 | .g => 103
 | .h => 104
 | .i => 105
 | .j => 106
 | .k => 107
 | .l => 108
 | .m => 109
 | .n => 110
 | .o => 111
 | .p => 112
 | .q => 113
 | .r => 114
 | .s => 115
 | .t => 116
 | .u => 117
 | .v => 118
 | .w => 119
 | .x => 120
 | .y => 121
 | .z => 122
 | .CurlyBraceOpen => 123
 | .VerticalBar => 124
 | .CurlyBraceClose => 125
 | .Tilde => 126
 | .DEL => 127
theorem as_byte_spec (c : AsciiChar) : c.as_byte ⦃ n => n.val = asciiCode c ⦄ := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_blank_spec (c : AsciiChar) : c.is_ascii_blank = .ok (decide (asciiCode c = 32 ∨ asciiCode c = 9)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_digit_spec (c : AsciiChar) : c.is_ascii_digit = .ok (decide (48 ≤ asciiCode c ∧ asciiCode c ≤ 57)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_whitespace_spec (c : AsciiChar) : c.is_ascii_whitespace = .ok (decide (asciiCode c = 32 ∨ asciiCode c = 9 ∨ asciiCode c = 10 ∨ asciiCode c = 13 ∨ asciiCode c = 12)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_control_spec (c : AsciiChar) : c.is_ascii_control = .ok (decide (asciiCode c < 32 ∨ asciiCode c = 127)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_alphabetic_spec (c : AsciiChar) : c.is_ascii_alphabetic = .ok (decide ((65 ≤ asciiCode c ∧ asciiCode c ≤ 90) ∨ (97 ≤ asciiCode c ∧ asciiCode c ≤ 122))) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_alphanumeric_spec (c : AsciiChar) : c.is_ascii_alphanumeric = .ok (decide ((48 ≤ asciiCode c ∧ asciiCode c ≤ 57) ∨ (65 ≤ asciiCode c ∧ asciiCode c ≤ 90) ∨ (97 ≤ asciiCode c ∧ asciiCode c ≤ 122))) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_graphic_spec (c : AsciiChar) : c.is_ascii_graphic ⦃ b => b = decide (33 ≤ asciiCode c ∧ asciiCode c ≤ 126) ⦄ := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits]
 all_goals
  step
  simp_all [U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_printable_spec (c : AsciiChar) : c.is_ascii_printable = .ok (decide (32 ≤ asciiCode c ∧ asciiCode c ≤ 126)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_lowercase_spec (c : AsciiChar) : c.is_ascii_lowercase = .ok (decide (97 ≤ asciiCode c ∧ asciiCode c ≤ 122)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_uppercase_spec (c : AsciiChar) : c.is_ascii_uppercase = .ok (decide (65 ≤ asciiCode c ∧ asciiCode c ≤ 90)) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
theorem is_ascii_hexdigit_spec (c : AsciiChar) : c.is_ascii_hexdigit = .ok (decide ((48 ≤ asciiCode c ∧ asciiCode c ≤ 57) ∨ (65 ≤ asciiCode c ∧ asciiCode c ≤ 70) ∨ (97 ≤ asciiCode c ∧ asciiCode c ≤ 102))) := by
 cases c <;> simp [asciiCode, AsciiChar.as_byte, AsciiChar.to_not_upper, AsciiChar.is_alphabetic, AsciiChar.is_ascii_alphabetic, AsciiChar.is_ascii_digit, AsciiChar.is_alphanumeric, AsciiChar.is_ascii_alphanumeric, AsciiChar.is_ascii_blank, AsciiChar.is_ascii_whitespace, AsciiChar.is_ascii_control, AsciiChar.is_ascii_graphic, AsciiChar.is_ascii_printable, AsciiChar.is_lowercase, AsciiChar.is_ascii_lowercase, AsciiChar.is_uppercase, AsciiChar.is_ascii_uppercase, AsciiChar.is_ascii_hexdigit, UScalar.cast, lift, AsciiChar.read_discriminant, UScalar.eq_equiv, UScalar.add, UScalar.tryMk, UScalar.inBounds, U8.size, U8.numBits] <;> (simp only [UScalar.val, UScalarTy.U8_numBits_eq] <;> decide)
end Proofs
#print axioms Proofs.as_byte_spec
#print axioms Proofs.is_ascii_blank_spec
#print axioms Proofs.is_ascii_digit_spec
#print axioms Proofs.is_ascii_whitespace_spec
#print axioms Proofs.is_ascii_control_spec
#print axioms Proofs.is_ascii_alphabetic_spec
#print axioms Proofs.is_ascii_alphanumeric_spec
#print axioms Proofs.is_ascii_graphic_spec
#print axioms Proofs.is_ascii_printable_spec
#print axioms Proofs.is_ascii_lowercase_spec
#print axioms Proofs.is_ascii_uppercase_spec
#print axioms Proofs.is_ascii_hexdigit_spec
