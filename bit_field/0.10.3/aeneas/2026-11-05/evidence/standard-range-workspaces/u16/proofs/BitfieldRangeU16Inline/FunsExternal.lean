module
public import Aeneas
public import BitField.FunsExternal
public import BitfieldRangeU16Inline.Types
@[expose] public section
open Aeneas Aeneas.Std
namespace bitfield_range_u16_inline
/-- Definitional alias of the unchanged previously validated provider; only the frozen U16 call is used. -/
abbrev core.mem.size_of (T : Type) [bit_field.PrimitiveRustLayout T] : Result Usize :=
 bit_field.core.mem.size_of T
end bitfield_range_u16_inline
