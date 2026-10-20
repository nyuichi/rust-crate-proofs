import extensions.collection.Generated

open Aeneas Aeneas.Std Result ControlFlow Error
open bytes

namespace BytesProofs.Collection

/-- The transparent `Bytes: FromIterator<u8>` body first collects through the
    supplied `IntoIterator` dictionary, then converts that Vec through the
    frozen `From<Vec<u8>> for Bytes` implementation. This theorem makes no
    claim about the iteration or conversion semantics of those callees. -/
theorem bytes_from_iterator_returns_vec_conversion
    {T : Type} {Iter : Type}
    (intoIter : core.iter.traits.collect.IntoIterator T Std.U8 Iter)
    (input : T) (vec : alloc.vec.Vec Std.U8) (out : bytes.Bytes)
    (hcollect : alloc.vec.FromIteratorVec.from_iter intoIter input = ok vec)
    (hconvert : core.convert.IntoFrom.into bytes.Bytes.Insts.CoreConvertFromVecU8 vec = ok out) :
    bytes.Bytes.Insts.CoreIterTraitsCollectFromIteratorU8.from_iter intoIter input = ok out := by
  have hfrom : bytes.Bytes.Insts.CoreConvertFromVecU8.from vec = ok out := by
    simpa [core.convert.IntoFrom.into] using hconvert
  simp [bytes.Bytes.Insts.CoreIterTraitsCollectFromIteratorU8.from_iter,
    hcollect, hfrom]

/-- The private helper returns exactly the result of adding the requested
    offset to the null pointer returned by the translated primitive. No
    provenance or machine-address interpretation is assumed. -/
theorem without_provenance_returns_pointer_operation
    (offset : Std.Usize) (nullPtr out : ConstRawPtr Std.U8)
    (hnull : core.ptr.null Std.U8 = ok nullPtr)
    (hadd : core.ptr.const_ptr.RawPtrConstT.wrapping_add nullPtr offset = ok out) :
    bytes.without_provenance offset = ok out := by
  simp [bytes.without_provenance, hnull, hadd]

#print axioms bytes_from_iterator_returns_vec_conversion
#print axioms without_provenance_returns_pointer_operation

end BytesProofs.Collection

#print axioms BytesProofs.Collection.bytes_from_iterator_returns_vec_conversion
#print axioms BytesProofs.Collection.without_provenance_returns_pointer_operation
