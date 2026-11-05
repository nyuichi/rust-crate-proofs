module
public import Aeneas
public import Serializers.Types
@[expose] public section
open Aeneas Aeneas.Std
namespace XoshiroSerializationModel
/-- Explicit Rust primitive layout evidence for this guarded serializer graph only.
Not Lean memory layout, not an arbitrary-Type Rust layout model. -/
class SerializerRustLayout (T : Type) where
  bytes : Usize
instance layout_u32 : SerializerRustLayout U32 := ⟨4#usize⟩
instance layout_u64 : SerializerRustLayout U64 := ⟨8#usize⟩
def core.mem.size_of (T : Type) [SerializerRustLayout T] : Result Usize :=
  .ok (SerializerRustLayout.bytes (T := T))
end XoshiroSerializationModel
