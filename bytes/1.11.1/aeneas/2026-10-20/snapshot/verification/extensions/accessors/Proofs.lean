import extensions.accessors.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Accessors

/-- Deref for `Bytes` returns the successful slice view of the handle. -/
theorem bytes_deref_returns_as_slice
    (self : bytes.Bytes) (slice : Slice Std.U8)
    (hasSlice : bytes.Bytes.as_slice self = ok slice) :
    bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref self = ok slice := by
  simp [bytes.Bytes.Insts.CoreOpsDerefDerefSliceU8.deref, hasSlice]

/-- `AsRef<[u8]>` for `Bytes` returns the same successful slice view. -/
theorem bytes_as_ref_returns_as_slice
    (self : bytes.Bytes) (slice : Slice Std.U8)
    (hasSlice : bytes.Bytes.as_slice self = ok slice) :
    bytes.Bytes.Insts.CoreConvertAsRefSliceU8.as_ref self = ok slice := by
  simp [bytes.Bytes.Insts.CoreConvertAsRefSliceU8.as_ref, hasSlice]

/-- `Borrow<[u8]>` for `Bytes` returns the same successful slice view. -/
theorem bytes_borrow_returns_as_slice
    (self : bytes.Bytes) (slice : Slice Std.U8)
    (hasSlice : bytes.Bytes.as_slice self = ok slice) :
    bytes.Bytes.Insts.CoreBorrowBorrowSliceU8.borrow self = ok slice := by
  simp [bytes.Bytes.Insts.CoreBorrowBorrowSliceU8.borrow, hasSlice]

/-- `AsRef<[u8]>` for `BytesMut` returns its successful slice view. -/
theorem bytes_mut_as_ref_returns_as_slice
    (self : bytes_mut.BytesMut) (slice : Slice Std.U8)
    (hasSlice : bytes_mut.BytesMut.as_slice self = ok slice) :
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref self = ok slice := by
  simp [bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref, hasSlice]

/-- `AsMut<[u8]>` for `BytesMut` forwards both the borrowed slice and its
    state-reconstruction closure from `as_slice_mut`. -/
theorem bytes_mut_as_mut_forwards_slice_and_rebuilder
    (self : bytes_mut.BytesMut) (slice : Slice Std.U8)
    (back : Slice Std.U8 → bytes_mut.BytesMut)
    (hasSliceMut : bytes_mut.BytesMut.as_slice_mut self = ok (slice, back)) :
    bytes_mut.BytesMut.Insts.CoreConvertAsMutSliceU8.as_mut self =
      ok (slice, back) := by
  simp [bytes_mut.BytesMut.Insts.CoreConvertAsMutSliceU8.as_mut, hasSliceMut]

/-- `Borrow<[u8]>` for `BytesMut` delegates through its `AsRef` implementation. -/
theorem bytes_mut_borrow_returns_as_slice
    (self : bytes_mut.BytesMut) (slice : Slice Std.U8)
    (hasSlice : bytes_mut.BytesMut.as_slice self = ok slice) :
    bytes_mut.BytesMut.Insts.CoreBorrowBorrowSliceU8.borrow self = ok slice := by
  simp [bytes_mut.BytesMut.Insts.CoreBorrowBorrowSliceU8.borrow,
    bytes_mut.BytesMut.Insts.CoreConvertAsRefSliceU8.as_ref, hasSlice]

/-- `BorrowMut<[u8]>` for `BytesMut` forwards the same mutable slice and
    reconstruction closure as `AsMut<[u8]>`. -/
theorem bytes_mut_borrow_mut_forwards_slice_and_rebuilder
    (self : bytes_mut.BytesMut) (slice : Slice Std.U8)
    (back : Slice Std.U8 → bytes_mut.BytesMut)
    (hasSliceMut : bytes_mut.BytesMut.as_slice_mut self = ok (slice, back)) :
    bytes_mut.BytesMut.Insts.CoreBorrowBorrowMutSliceU8.borrow_mut self =
      ok (slice, back) := by
  simp [bytes_mut.BytesMut.Insts.CoreBorrowBorrowMutSliceU8.borrow_mut,
    bytes_mut.BytesMut.Insts.CoreConvertAsMutSliceU8.as_mut, hasSliceMut]

/-- `DerefMut` for `BytesMut` forwards the mutable slice and reconstruction
    closure from `as_slice_mut`. -/
theorem bytes_mut_deref_mut_forwards_slice_and_rebuilder
    (self : bytes_mut.BytesMut) (slice : Slice Std.U8)
    (back : Slice Std.U8 → bytes_mut.BytesMut)
    (hasSliceMut : bytes_mut.BytesMut.as_slice_mut self = ok (slice, back)) :
    bytes_mut.BytesMut.Insts.CoreOpsDerefDerefMutSliceU8.deref_mut self =
      ok (slice, back) := by
  simp [bytes_mut.BytesMut.Insts.CoreOpsDerefDerefMutSliceU8.deref_mut,
    bytes_mut.BytesMut.Insts.CoreConvertAsMutSliceU8.as_mut, hasSliceMut]

#print axioms bytes_deref_returns_as_slice
#print axioms bytes_as_ref_returns_as_slice
#print axioms bytes_borrow_returns_as_slice
#print axioms bytes_mut_as_ref_returns_as_slice
#print axioms bytes_mut_as_mut_forwards_slice_and_rebuilder
#print axioms bytes_mut_borrow_returns_as_slice
#print axioms bytes_mut_borrow_mut_forwards_slice_and_rebuilder
#print axioms bytes_mut_deref_mut_forwards_slice_and_rebuilder

end BytesProofs.Accessors
