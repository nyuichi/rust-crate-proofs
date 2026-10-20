import extensions.borrowed_iter.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes_borrowed_iter_harness

namespace BytesProofs.BorrowedIter

/-- The extracted `IntoIterator for &Bytes` body returns the Aeneas slice
    iterator beginning at index zero, given the frozen `as_slice` result. -/
theorem bytes_borrowed_iterator_starts_at_slice_front
    (self : bytes.bytes.Bytes) (slice : Slice Std.U8)
    (h_slice : bytes.bytes.Bytes.as_slice self = ok slice) :
    SharedABytes.Insts.CoreIterTraitsCollectIntoIteratorSharedAU8IterAU8.into_iter self =
      ok ({ slice := slice, i := 0 } : core.slice.iter.Iter Std.U8) := by
  simp [SharedABytes.Insts.CoreIterTraitsCollectIntoIteratorSharedAU8IterAU8.into_iter,
    core.slice.Slice.iter, h_slice]

/-- The extracted `IntoIterator for &BytesMut` body returns a slice iterator
    at index zero. The only buffer-specific premise exposes its frozen slice
    view; the transparent `AsRef` forwarding body is reduced in the proof. -/
theorem bytes_mut_borrowed_iterator_starts_at_slice_front
    (self : bytes.bytes_mut.BytesMut) (slice : Slice Std.U8)
    (h_slice : bytes.bytes_mut.BytesMut.as_slice self = ok slice) :
    SharedABytesMut.Insts.CoreIterTraitsCollectIntoIteratorSharedAU8IterAU8.into_iter self =
      ok ({ slice := slice, i := 0 } : core.slice.iter.Iter Std.U8) := by
  simp [SharedABytesMut.Insts.CoreIterTraitsCollectIntoIteratorSharedAU8IterAU8.into_iter,
    bytes.bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref,
    core.slice.Slice.iter, h_slice]

#print axioms bytes_borrowed_iterator_starts_at_slice_front
#print axioms bytes_mut_borrowed_iterator_starts_at_slice_front

end BytesProofs.BorrowedIter
