import BitfieldRangeU128Inline.Funs
open Aeneas Aeneas.Std Result
namespace LayoutAudit
theorem alias_exact (T : Type) [bit_field.PrimitiveRustLayout T] :
 bitfield_range_u128_inline.core.mem.size_of T = bit_field.core.mem.size_of T := rfl
theorem u128_size : bitfield_range_u128_inline.core.mem.size_of U128 = .ok 16#usize := rfl
example : True := by
 fail_if_success have h : bit_field.PrimitiveRustLayout Bool := inferInstance
 trivial
end LayoutAudit
set_option pp.explicit true in
#print bitfield_range_u128_inline.U128.Insts.Bit_fieldBitField.BIT_LENGTH
#print axioms LayoutAudit.alias_exact
#print axioms LayoutAudit.u128_size
