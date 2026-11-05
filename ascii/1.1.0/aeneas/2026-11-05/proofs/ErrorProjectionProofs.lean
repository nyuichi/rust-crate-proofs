import Generated.Ascii
open Aeneas Aeneas.Std Result
universe u
namespace ErrorProjectionProofs
/-- All owners O, without Clone/Copy constraints or an allocation assumption. -/
theorem ascii_error_exact {O : Type u} (value : ascii.ascii_string.FromAsciiError O) :
    ascii.ascii_string.FromAsciiError.ascii_error value = .ok value.error := by rfl
/-- The original owner is returned unchanged, with no conversion or error. -/
theorem into_source_exact {O : Type u} (value : ascii.ascii_string.FromAsciiError O) :
    ascii.ascii_string.FromAsciiError.into_source value = .ok value.owner := by rfl
end ErrorProjectionProofs
#print axioms ErrorProjectionProofs.ascii_error_exact
#print axioms ErrorProjectionProofs.into_source_exact
