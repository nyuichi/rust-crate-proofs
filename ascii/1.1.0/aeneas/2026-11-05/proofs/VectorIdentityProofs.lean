import Generated.Ascii
open Aeneas Aeneas.Std Result
namespace VectorIdentityProofs
theorem from_vector_exact (v : alloc.vec.Vec ascii.ascii_char.AsciiChar) :
 ascii.ascii_string.AsciiString.Insts.CoreConvertFromVecAsciiChar.from v = .ok ⟨v⟩ := by rfl
theorem checked_exact (v : alloc.vec.Vec ascii.ascii_char.AsciiChar) :
 ascii.alloc.vec.VecAsciiChar.Insts.AsciiAscii_stringIntoAsciiString.into_ascii_string v = .ok (.Ok ⟨v⟩) := by
 simp [ascii.alloc.vec.VecAsciiChar.Insts.AsciiAscii_stringIntoAsciiString.into_ascii_string, ascii.ascii_string.AsciiString.Insts.CoreConvertFromVecAsciiChar.from, Std.bind_ok, bind_tc_ok]
theorem unchecked_exact (v : alloc.vec.Vec ascii.ascii_char.AsciiChar) :
 ascii.alloc.vec.VecAsciiChar.Insts.AsciiAscii_stringIntoAsciiString.into_ascii_string_unchecked v = .ok ⟨v⟩ := by rfl
theorem into_vector_exact (s : ascii.ascii_string.AsciiString) : ascii.alloc.vec.VecAsciiChar.Insts.CoreConvertFromAsciiString.from s = .ok s.vec := by rfl
end VectorIdentityProofs
#print axioms VectorIdentityProofs.from_vector_exact
#print axioms VectorIdentityProofs.checked_exact
#print axioms VectorIdentityProofs.unchecked_exact

#print axioms VectorIdentityProofs.into_vector_exact
