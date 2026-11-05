import Generated.Ascii
open Aeneas Aeneas.Std
open ascii.ascii_char
set_option linter.internalConstructors false
set_option linter.unusedSimpArgs false
set_option maxHeartbeats 0
set_option maxRecDepth 4096
namespace DigitProofs
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
-- Canonical scalar boundary: bare generated BitVec constructor literals
-- retain the exact unsigned value; this is independent of the crate body.
@[simp, scalar_tac_simps]
theorem literalU8_val (n : Nat) : (⟨BitVec.ofNat 8 n⟩ : U8).val = n % 256 := by rfl
@[simp, scalar_tac_simps]
theorem literalU32_val (n : Nat) : (⟨BitVec.ofNat 32 n⟩ : U32).val = n % 4294967296 := by rfl

def digitValue (c : AsciiChar) : Option Nat :=
 let n := asciiCode c
 if 48 ≤ n ∧ n ≤ 57 then some (n - 48)
 else if 65 ≤ n ∧ n ≤ 90 then some (n - 55)
 else if 97 ≤ n ∧ n ≤ 122 then some (n - 87)
 else none

def digitMatches (c : AsciiChar) (radix : Nat) : Bool :=
 match digitValue c with
 | some value => decide (value < radix)
 | none => false

theorem is_digit_success_spec (c : AsciiChar) (r : U32) (h : r.val ≤ 36) :
 c.is_digit r ⦃ b => b = digitMatches c r.val ⦄ := by
 have h0 : 0#u32 ≤ r := by scalar_tac
 have h36 : r ≤ 36#u32 := by scalar_tac
 by_cases h11 : 11#u32 ≤ r
 <;> cases c
 <;> simp [AsciiChar.is_digit, asciiCode, digitValue, digitMatches,
   UScalar.cast, AsciiChar.read_discriminant, lift, massert, h0, h36, h11]
 all_goals first | scalar_tac | (step* <;> scalar_tac)

theorem is_digit_panic_spec (c : AsciiChar) (r : U32) (h : 36 < r.val) :
 c.is_digit r = .fail .assertionFailure := by
 have h0 : 0#u32 ≤ r := by scalar_tac
 have h36 : ¬ r ≤ 36#u32 := by scalar_tac
 have h11 : 11#u32 ≤ r := by scalar_tac
 cases c <;> simp [AsciiChar.is_digit, UScalar.cast,
   AsciiChar.read_discriminant, lift, massert, h0, h36, h11]
end DigitProofs
#print axioms DigitProofs.is_digit_success_spec
#print axioms DigitProofs.is_digit_panic_spec
