import BitfieldRangeUsizeInline.Funs
open Aeneas Aeneas.Std Result
namespace LayoutAudit
theorem alias_exact (T : Type) [bit_field.PrimitiveRustLayout T] :
 bitfield_range_usize_inline.core.mem.size_of T = bit_field.core.mem.size_of T := rfl
theorem usize_size : bitfield_range_usize_inline.core.mem.size_of Usize = .ok 8#usize := rfl
example : True := by
 fail_if_success have h : bit_field.PrimitiveRustLayout Bool := inferInstance
 trivial
end LayoutAudit
set_option pp.explicit true in
#print bitfield_range_usize_inline.Usize.Insts.Bit_fieldBitField.BIT_LENGTH
#print axioms LayoutAudit.alias_exact
#print axioms LayoutAudit.usize_size
