import extensions.hash.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Hash

/-- `Bytes::hash` hashes its successful slice view using the supplied hasher
    and returns that slice-hash result. -/
theorem bytes_hash_delegates_to_slice_hash
    {H : Type _} (hasher : core.hash.Hasher H) (self : bytes.Bytes)
    (state out : H) (slice : Slice Std.U8)
    (hasSlice : bytes.Bytes.as_slice self = ok slice)
    (hasHash : Slice.Insts.CoreHashHash.hash U8.Insts.CoreHashHash hasher
      slice state = ok out) :
    bytes.Bytes.Insts.CoreHashHash.hash hasher self state = ok out := by
  simp [bytes.Bytes.Insts.CoreHashHash.hash, hasSlice, hasHash]

/-- `BytesMut::hash` converts to its slice view and delegates to slice hashing. -/
theorem bytes_mut_hash_delegates_to_slice_hash
    {H : Type _} (hasher : core.hash.Hasher H) (self : bytes_mut.BytesMut)
    (state out : H) (slice : Slice Std.U8)
    (hasSlice : bytes_mut.BytesMut.as_slice self = ok slice)
    (hasHash : Slice.Insts.CoreHashHash.hash U8.Insts.CoreHashHash hasher
      slice state = ok out) :
    bytes_mut.BytesMut.Insts.CoreHashHash.hash hasher self state = ok out := by
  simp [bytes_mut.BytesMut.Insts.CoreHashHash.hash,
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref,
    hasSlice, hasHash]

#print axioms bytes_hash_delegates_to_slice_hash
#print axioms bytes_mut_hash_delegates_to_slice_hash

end BytesProofs.Hash

#print axioms BytesProofs.Hash.bytes_hash_delegates_to_slice_hash
#print axioms BytesProofs.Hash.bytes_mut_hash_delegates_to_slice_hash
