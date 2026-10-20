import extensions.constructors.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Constructors

/-- On successful primitive pointer/atomic/vtable calls, `from_static`
    preserves the slice pointer and logical length in the resulting handle.
    The premises expose only outcomes of frozen library/model operations. -/
theorem bytes_from_static_records_slice_metadata
    (src : Slice Std.U8) (ptr : ConstRawPtr Std.U8)
    (nullPtr : MutRawPtr Unit)
    (atomic : core.sync.atomic.Atomic (MutRawPtr Unit)
      (core.sync.atomic.private.Align8 (MutRawPtr Unit)))
    (vtable : bytes.Vtable)
    (hasPtr : core.slice.Slice.as_ptr src = ok ptr)
    (hasNull : core.ptr.null_mut Unit = ok nullPtr)
    (hasAtomic : core.sync.atomic.AtomicRawPtrMutTAlign8RawPtrMutT.new
      nullPtr = ok atomic)
    (hasVtable : bytes.STATIC_VTABLE = ok vtable) :
    bytes.Bytes.from_static src =
      ok { ptr := ptr, len := Slice.len src, data := atomic,
           vtable := vtable } := by
  simp [bytes.Bytes.from_static, hasPtr, hasNull, hasAtomic, hasVtable]

private theorem empty_static_slice_has_zero_length :
    Slice.len bytes.Bytes.new.EMPTY = 0#usize := by
  simp [bytes.Bytes.new.EMPTY, Slice.len, Array.to_slice, Std.Array.empty]
  scalar_tac

/-- When the opaque static-handle primitives succeed on `new.EMPTY`, `Bytes::new`
    returns a handle whose logical length is zero. -/
theorem bytes_new_returns_empty_handle
    (ptr : ConstRawPtr Std.U8) (nullPtr : MutRawPtr Unit)
    (atomic : core.sync.atomic.Atomic (MutRawPtr Unit)
      (core.sync.atomic.private.Align8 (MutRawPtr Unit)))
    (vtable : bytes.Vtable)
    (hasPtr : core.slice.Slice.as_ptr bytes.Bytes.new.EMPTY = ok ptr)
    (hasNull : core.ptr.null_mut Unit = ok nullPtr)
    (hasAtomic : core.sync.atomic.AtomicRawPtrMutTAlign8RawPtrMutT.new
      nullPtr = ok atomic)
    (hasVtable : bytes.STATIC_VTABLE = ok vtable) :
    bytes.Bytes.new =
      ok { ptr := ptr, len := 0#usize, data := atomic, vtable := vtable } := by
  have hfrom := bytes_from_static_records_slice_metadata
    bytes.Bytes.new.EMPTY ptr nullPtr atomic vtable hasPtr hasNull hasAtomic
    hasVtable
  simpa [bytes.Bytes.new, empty_static_slice_has_zero_length] using hfrom

/-- `Bytes::default` delegates to `Bytes::new`; under the same explicit
    successful primitive-call outcomes, it yields an empty handle. -/
theorem bytes_default_returns_empty_handle
    (ptr : ConstRawPtr Std.U8) (nullPtr : MutRawPtr Unit)
    (atomic : core.sync.atomic.Atomic (MutRawPtr Unit)
      (core.sync.atomic.private.Align8 (MutRawPtr Unit)))
    (vtable : bytes.Vtable)
    (hasPtr : core.slice.Slice.as_ptr bytes.Bytes.new.EMPTY = ok ptr)
    (hasNull : core.ptr.null_mut Unit = ok nullPtr)
    (hasAtomic : core.sync.atomic.AtomicRawPtrMutTAlign8RawPtrMutT.new
      nullPtr = ok atomic)
    (hasVtable : bytes.STATIC_VTABLE = ok vtable) :
    bytes.Bytes.Insts.CoreDefaultDefault.default =
      ok { ptr := ptr, len := 0#usize, data := atomic, vtable := vtable } := by
  simpa [bytes.Bytes.Insts.CoreDefaultDefault.default] using
    bytes_new_returns_empty_handle ptr nullPtr atomic vtable
      hasPtr hasNull hasAtomic hasVtable

#print axioms bytes_from_static_records_slice_metadata
#print axioms bytes_new_returns_empty_handle
#print axioms bytes_default_returns_empty_handle

end BytesProofs.Constructors
