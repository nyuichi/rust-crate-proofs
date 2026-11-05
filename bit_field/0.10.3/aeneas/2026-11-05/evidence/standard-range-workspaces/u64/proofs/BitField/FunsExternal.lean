module
public import Aeneas
public import BitField.Types
@[expose] public section
open Aeneas Aeneas.Std
namespace bit_field
/-- Evidence for only the explicitly selected primitive uses; not a Rust trait or an arbitrary-Type layout model. -/
class PrimitiveRustLayout (T : Type) where
  bytes : Usize
instance layout_u8 : PrimitiveRustLayout U8 := ⟨1#usize⟩
instance layout_u16 : PrimitiveRustLayout U16 := ⟨2#usize⟩
instance layout_u32 : PrimitiveRustLayout U32 := ⟨4#usize⟩
instance layout_u64 : PrimitiveRustLayout U64 := ⟨8#usize⟩
instance layout_u128 : PrimitiveRustLayout U128 := ⟨16#usize⟩
instance layout_usize : PrimitiveRustLayout Usize := ⟨8#usize⟩
instance layout_i8 : PrimitiveRustLayout I8 := ⟨1#usize⟩
instance layout_i16 : PrimitiveRustLayout I16 := ⟨2#usize⟩
instance layout_i32 : PrimitiveRustLayout I32 := ⟨4#usize⟩
instance layout_i64 : PrimitiveRustLayout I64 := ⟨8#usize⟩
instance layout_i128 : PrimitiveRustLayout I128 := ⟨16#usize⟩
instance layout_isize : PrimitiveRustLayout Isize := ⟨8#usize⟩
def core.mem.size_of (T : Type) [PrimitiveRustLayout T] : Result Usize :=
  .ok (PrimitiveRustLayout.bytes (T := T))
theorem core.mem.size_ofU8 : core.mem.size_of U8 = .ok 1#usize := by rfl
theorem core.mem.size_ofU16 : core.mem.size_of U16 = .ok 2#usize := by rfl
theorem core.mem.size_ofU32 : core.mem.size_of U32 = .ok 4#usize := by rfl
theorem core.mem.size_ofU64 : core.mem.size_of U64 = .ok 8#usize := by rfl
theorem core.mem.size_ofU128 : core.mem.size_of U128 = .ok 16#usize := by rfl
theorem core.mem.size_ofUsize : core.mem.size_of Usize = .ok 8#usize := by rfl
theorem core.mem.size_ofI8 : core.mem.size_of I8 = .ok 1#usize := by rfl
theorem core.mem.size_ofI16 : core.mem.size_of I16 = .ok 2#usize := by rfl
theorem core.mem.size_ofI32 : core.mem.size_of I32 = .ok 4#usize := by rfl
theorem core.mem.size_ofI64 : core.mem.size_of I64 = .ok 8#usize := by rfl
theorem core.mem.size_ofI128 : core.mem.size_of I128 = .ok 16#usize := by rfl
theorem core.mem.size_ofIsize : core.mem.size_of Isize = .ok 8#usize := by rfl
end bit_field
