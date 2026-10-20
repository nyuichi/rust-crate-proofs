import extensions.writers.Generated

open Aeneas Aeneas.Std Result

namespace BytesProofs.Extensions.Writers

/-!
This module proves adapter behavior under explicit interface laws for the
opaque `BufMut` implementation methods. The laws below are quantified over all
successful writes; they do not assume a particular buffer representation.
-/

theorem put_array_encoding_behavior
    {Self Value : Type _} {N : Std.Usize}
    (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (write : Self → Value → Result Self)
    (encode : Value → Array Std.U8 N)
    (self : Self) (value : Value) (after : Self)
    (method_body :
      write self value = bufInst.put_slice self (Array.to_slice (encode value)))
    (hwrite : bufInst.put_slice self (Array.to_slice (encode value)) = ok after) :
    write self value = ok after ∧
      contents after = contents self ++ (encode value).val := by
  constructor
  · rw [method_body]
    exact hwrite
  · calc
      contents after = contents self ++ (Array.to_slice (encode value)).val :=
        put_slice_appends self (Array.to_slice (encode value)) after hwrite
      _ = contents self ++ (encode value).val := by simp [Array.to_slice]

theorem put_result_array_encoding_behavior
    {Self Value : Type _} {N : Std.Usize}
    (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (write : Self → Value → Result Self)
    (encode : Value → Result (Array Std.U8 N))
    (self : Self) (value : Value) (bytes : Array Std.U8 N) (after : Self)
    (method_body :
      write self value = do
        let bytes ← encode value
        let slice ← lift (Array.to_slice bytes)
        bufInst.put_slice self slice)
    (hencode : encode value = ok bytes)
    (hwrite : bufInst.put_slice self (Array.to_slice bytes) = ok after) :
    write self value = ok after ∧
      contents after = contents self ++ bytes.val := by
  constructor
  · rw [method_body]
    simp [hencode, hwrite, Aeneas.Std.lift]
  · calc
      contents after = contents self ++ (Array.to_slice bytes).val :=
        put_slice_appends self (Array.to_slice bytes) after hwrite
      _ = contents self ++ bytes.val := by simp [Array.to_slice]

theorem put_bits_via_converter_behavior
    {Self Value Word : Type _}
    (contents : Self → List Std.U8)
    (put_word : Self → Word → Result Self)
    (wordBytes : Word → List Std.U8)
    (put_word_appends : ∀ (self : Self) (word : Word) (after : Self),
      put_word self word = ok after → contents after = contents self ++ wordBytes word)
    (write : Self → Value → Result Self)
    (encode : Value → Result Word)
    (self : Self) (value : Value) (word : Word) (after : Self)
    (method_body : write self value = do
      let word ← encode value
      put_word self word)
    (hencode : encode value = ok word)
    (hput : put_word self word = ok after) :
    write self value = ok after ∧
      contents after = contents self ++ wordBytes word := by
  constructor
  · rw [method_body]
    simp [hencode, hput]
  · exact put_word_appends self word after hput

theorem has_remaining_mut_iff_nonzero
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (self : Self) (remaining : Std.Usize)
    (hremaining : bufInst.remaining_mut self = ok remaining) :
    bytes.buf.buf_mut.BufMut.has_remaining_mut.default bufInst self =
      ok (decide (remaining.val ≠ 0)) := by
  simp [bytes.buf.buf_mut.BufMut.has_remaining_mut.default, hremaining]; omega

theorem limit_default_preserves_inner_and_sets_limit
    {Self : Type _} (self : Self) (limit : Std.Usize) :
    bytes.buf.buf_mut.BufMut.limit.default self limit =
      ok ({ inner := self, limit := limit } : bytes.buf.limit.Limit Self) := by
  rfl

theorem chain_mut_default_places_buffers_in_order
    {Self Next : Type _} (self : Self) (next : Next) :
    bytes.buf.buf_mut.BufMut.chain_mut.default self next =
      ok ({ a := self, b := next } : bytes.buf.chain.Chain Self Next) := by
  rfl

theorem writer_default_returns_factory_result
    {Self : Type _} (self : Self) (writer : bytes.buf.writer.Writer Self)
    (hnew : bytes.buf.writer.new self = ok writer) :
    bytes.buf.buf_mut.BufMut.writer.default self = ok writer := by
  simp [bytes.buf.buf_mut.BufMut.writer.default, hnew]

theorem uninit_slice_len_is_logical_slice_length
    (self : bytes.buf.uninit_slice.UninitSlice) :
    ∃ n : Std.Usize,
      bytes.buf.uninit_slice.UninitSlice.len self = ok n ∧ n.val = self.length := by
  refine ⟨Slice.len self, rfl, ?_⟩
  exact Slice.len_val self

theorem put_u16_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U16) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U16.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u16.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u16.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U16.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u16.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u16.default bufInst s n)
    (fun n => core.num.U16.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.U16.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U16.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u16_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U16) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U16.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u16_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u16_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U16.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u16_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u16_le.default bufInst s n)
    (fun n => core.num.U16.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.U16.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U16.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i16_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I16) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I16.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i16.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i16.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I16.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i16.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i16.default bufInst s n)
    (fun n => core.num.I16.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.I16.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I16.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i16_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I16) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I16.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i16_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i16_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I16.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i16_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i16_le.default bufInst s n)
    (fun n => core.num.I16.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.I16.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I16.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u32_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U32) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U32.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u32.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u32.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U32.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u32.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u32.default bufInst s n)
    (fun n => core.num.U32.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.U32.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U32.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u32_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U32) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U32.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u32_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u32_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U32.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u32_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u32_le.default bufInst s n)
    (fun n => core.num.U32.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.U32.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U32.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i32_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I32) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I32.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i32.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i32.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I32.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i32.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i32.default bufInst s n)
    (fun n => core.num.I32.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.I32.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I32.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i32_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I32) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I32.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i32_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i32_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I32.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i32_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i32_le.default bufInst s n)
    (fun n => core.num.I32.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.I32.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I32.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u64_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U64) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U64.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u64.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u64.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U64.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u64.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u64.default bufInst s n)
    (fun n => core.num.U64.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.U64.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U64.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u64_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U64) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U64.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u64_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u64_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U64.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u64_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u64_le.default bufInst s n)
    (fun n => core.num.U64.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.U64.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U64.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i64_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I64) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I64.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i64.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i64.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I64.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i64.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i64.default bufInst s n)
    (fun n => core.num.I64.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.I64.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I64.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i64_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I64) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I64.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i64_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i64_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I64.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i64_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i64_le.default bufInst s n)
    (fun n => core.num.I64.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.I64.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I64.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u128_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U128) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U128.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u128.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u128.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U128.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u128.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u128.default bufInst s n)
    (fun n => core.num.U128.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.U128.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U128.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u128_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U128) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.U128.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u128_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u128_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.U128.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_u128_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u128_le.default bufInst s n)
    (fun n => core.num.U128.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.U128.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.U128.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i128_be_appends_big_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I128) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I128.to_be_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i128.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i128.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I128.to_be_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i128.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i128.default bufInst s n)
    (fun n => core.num.I128.to_be_bytes n) self value after hmethod hput
  have hencoding : (core.num.I128.to_be_bytes value).val =
      value.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I128.to_be_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_i128_le_appends_little_endian_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I128) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (core.num.I128.to_le_bytes value)) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i128_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i128_le.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (core.num.I128.to_le_bytes value)) := by
    simp [bytes.buf.buf_mut.BufMut.put_i128_le.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i128_le.default bufInst s n)
    (fun n => core.num.I128.to_le_bytes n) self value after hmethod hput
  have hencoding : (core.num.I128.to_le_bytes value).val =
      value.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
    simp [core.num.I128.to_le_bytes]
  exact ⟨hbehavior.1, by rw [hbehavior.2, hencoding]⟩

theorem put_u16_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U16) (encoded : Array Std.U8 2#usize)
    (after : Self)
    (hencode : bytes.core.num.U16.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u16_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u16_ne.default bufInst self value = (do
        let encoded ← bytes.core.num.U16.to_ne_bytes value
        let slice ← lift (Array.to_slice encoded)
        bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_u16_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u16_ne.default bufInst s n)
    (fun n => bytes.core.num.U16.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_u8_appends_one_byte
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U8) (after : Self)
    (hput : bufInst.put_slice self
      (Array.to_slice (Array.make 1#usize [value])) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u8.default bufInst self value = ok after ∧
      contents after = contents self ++ [value] := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_u8.default bufInst self value =
        bufInst.put_slice self (Array.to_slice (Array.make 1#usize [value])) := by
    simp [bytes.buf.buf_mut.BufMut.put_u8.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u8.default bufInst s n)
    (fun n => Array.make 1#usize [n]) self value after hmethod hput
  exact ⟨hbehavior.1, by simpa using hbehavior.2⟩

theorem put_i8_appends_sign_preserving_byte
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I8) (after : Self)
    (hput : bufInst.put_slice self (Array.to_slice
      (Array.make 1#usize [IScalar.hcast UScalarTy.U8 value])) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i8.default bufInst self value = ok after ∧
      contents after = contents self ++ [IScalar.hcast UScalarTy.U8 value] := by
  have hmethod :
      bytes.buf.buf_mut.BufMut.put_i8.default bufInst self value =
        bufInst.put_slice self (Array.to_slice
          (Array.make 1#usize [IScalar.hcast UScalarTy.U8 value])) := by
    simp [bytes.buf.buf_mut.BufMut.put_i8.default, Aeneas.Std.lift]
  have hbehavior := put_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i8.default bufInst s n)
    (fun n => Array.make 1#usize [IScalar.hcast UScalarTy.U8 n])
    self value after hmethod hput
  exact ⟨hbehavior.1, by simpa using hbehavior.2⟩

theorem put_uint_ne_uses_little_endian_method
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (self : Self) (value : Std.U64) (nbytes : Std.Usize) :
    bytes.buf.buf_mut.BufMut.put_uint_ne.default bufInst self value nbytes =
      bufInst.put_uint_le self value nbytes := by
  rfl

theorem put_int_ne_uses_little_endian_method
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (self : Self) (value : Std.I64) (nbytes : Std.Usize) :
    bytes.buf.buf_mut.BufMut.put_int_ne.default bufInst self value nbytes =
      bufInst.put_int_le self value nbytes := by
  rfl

theorem put_i16_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I16)
    (encoded : Array Std.U8 2#usize) (after : Self)
    (hencode : bytes.core.num.I16.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i16_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_i16_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.I16.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_i16_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i16_ne.default bufInst s n)
    (fun n => bytes.core.num.I16.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_u32_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U32)
    (encoded : Array Std.U8 4#usize) (after : Self)
    (hencode : bytes.core.num.U32.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u32_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_u32_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.U32.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_u32_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u32_ne.default bufInst s n)
    (fun n => bytes.core.num.U32.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_i32_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I32)
    (encoded : Array Std.U8 4#usize) (after : Self)
    (hencode : bytes.core.num.I32.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i32_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_i32_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.I32.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_i32_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i32_ne.default bufInst s n)
    (fun n => bytes.core.num.I32.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_u64_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U64)
    (encoded : Array Std.U8 8#usize) (after : Self)
    (hencode : bytes.core.num.U64.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u64_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_u64_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.U64.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_u64_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u64_ne.default bufInst s n)
    (fun n => bytes.core.num.U64.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_i64_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I64)
    (encoded : Array Std.U8 8#usize) (after : Self)
    (hencode : bytes.core.num.I64.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i64_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_i64_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.I64.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_i64_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i64_ne.default bufInst s n)
    (fun n => bytes.core.num.I64.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_u128_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.U128)
    (encoded : Array Std.U8 16#usize) (after : Self)
    (hencode : bytes.core.num.U128.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_u128_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_u128_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.U128.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_u128_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_u128_ne.default bufInst s n)
    (fun n => bytes.core.num.U128.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_i128_ne_appends_native_converter_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_slice_appends : ∀ (self : Self) (slice : Slice Std.U8) (after : Self),
      bufInst.put_slice self slice = ok after →
        contents after = contents self ++ slice.val)
    (self : Self) (value : Std.I128)
    (encoded : Array Std.U8 16#usize) (after : Self)
    (hencode : bytes.core.num.I128.to_ne_bytes value = ok encoded)
    (hput : bufInst.put_slice self (Array.to_slice encoded) = ok after) :
    bytes.buf.buf_mut.BufMut.put_i128_ne.default bufInst self value = ok after ∧
      contents after = contents self ++ encoded.val := by
  have hmethod : bytes.buf.buf_mut.BufMut.put_i128_ne.default bufInst self value = (do
      let encoded ← bytes.core.num.I128.to_ne_bytes value
      let slice ← lift (Array.to_slice encoded)
      bufInst.put_slice self slice) := by
    simp [bytes.buf.buf_mut.BufMut.put_i128_ne.default, Aeneas.Std.lift]
  exact put_result_array_encoding_behavior bufInst contents put_slice_appends
    (fun s n => bytes.buf.buf_mut.BufMut.put_i128_ne.default bufInst s n)
    (fun n => bytes.core.num.I128.to_ne_bytes n) self value encoded after hmethod hencode hput

theorem put_f32_appends_be_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U32) (after : Self),
      bufInst.put_u32 self bits = ok after →
        contents after = contents self ++
          bits.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F32) (bits : Std.U32) (after : Self)
    (hencode : bytes.core.f32.F32.to_bits value = ok bits)
    (hput : bufInst.put_u32 self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f32.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u32 s word)
    (fun word => word.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f32.default bufInst s x)
    (fun x => bytes.core.f32.F32.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f32.default])
    hencode hput

theorem put_f32_le_appends_le_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U32) (after : Self),
      bufInst.put_u32_le self bits = ok after →
        contents after = contents self ++
          bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F32) (bits : Std.U32) (after : Self)
    (hencode : bytes.core.f32.F32.to_bits value = ok bits)
    (hput : bufInst.put_u32_le self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f32_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u32_le s word)
    (fun word => word.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f32_le.default bufInst s x)
    (fun x => bytes.core.f32.F32.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f32_le.default])
    hencode hput

theorem put_f32_ne_appends_le_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U32) (after : Self),
      bufInst.put_u32_ne self bits = ok after →
        contents after = contents self ++
          bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F32) (bits : Std.U32) (after : Self)
    (hencode : bytes.core.f32.F32.to_bits value = ok bits)
    (hput : bufInst.put_u32_ne self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f32_ne.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u32_ne s word)
    (fun word => word.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f32_ne.default bufInst s x)
    (fun x => bytes.core.f32.F32.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f32_ne.default])
    hencode hput

theorem put_f64_appends_be_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U64) (after : Self),
      bufInst.put_u64 self bits = ok after →
        contents after = contents self ++
          bits.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F64) (bits : Std.U64) (after : Self)
    (hencode : bytes.core.f64.F64.to_bits value = ok bits)
    (hput : bufInst.put_u64 self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f64.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u64 s word)
    (fun word => word.bv.toBEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f64.default bufInst s x)
    (fun x => bytes.core.f64.F64.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f64.default])
    hencode hput

theorem put_f64_le_appends_le_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U64) (after : Self),
      bufInst.put_u64_le self bits = ok after →
        contents after = contents self ++
          bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F64) (bits : Std.U64) (after : Self)
    (hencode : bytes.core.f64.F64.to_bits value = ok bits)
    (hput : bufInst.put_u64_le self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f64_le.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u64_le s word)
    (fun word => word.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f64_le.default bufInst s x)
    (fun x => bytes.core.f64.F64.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f64_le.default])
    hencode hput

theorem put_f64_ne_appends_le_bit_bytes
    {Self : Type _} (bufInst : bytes.buf.buf_mut.BufMut Self)
    (contents : Self → List Std.U8)
    (put_word_appends : ∀ (self : Self) (bits : Std.U64) (after : Self),
      bufInst.put_u64_ne self bits = ok after →
        contents after = contents self ++
          bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (self : Self) (value : F64) (bits : Std.U64) (after : Self)
    (hencode : bytes.core.f64.F64.to_bits value = ok bits)
    (hput : bufInst.put_u64_ne self bits = ok after) :
    bytes.buf.buf_mut.BufMut.put_f64_ne.default bufInst self value = ok after ∧
      contents after = contents self ++
        bits.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) := by
  exact put_bits_via_converter_behavior contents
    (fun s word => bufInst.put_u64_ne s word)
    (fun word => word.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8))
    (fun s word result => put_word_appends s word result)
    (fun s x => bytes.buf.buf_mut.BufMut.put_f64_ne.default bufInst s x)
    (fun x => bytes.core.f64.F64.to_bits x)
    self value bits after (by
      simp [bytes.buf.buf_mut.BufMut.put_f64_ne.default])
    hencode hput

/-!
The model below gives `put_slice` a concrete, bounded content interpretation:
successful calls append the supplied slice, and calls that exceed the model
capacity fail. Other `BufMut` operations fail explicitly, since this model is
only used to exercise the translated `put_u32_le` default through its
`put_slice` callback.
-/

structure BoundedWriterState where
  capacity : Nat
  contents : List Std.U8

def boundedListPutSlice
    (self : BoundedWriterState) (slice : Slice Std.U8) : Result BoundedWriterState :=
  if self.contents.length + slice.val.length ≤ self.capacity then
    ok { self with contents := self.contents ++ slice.val }
  else
    Result.fail Error.maximumSizeExceeded

def rejectBoundedWriterCall {A : Type _}
    (_self : BoundedWriterState) (_arg : A) : Result BoundedWriterState :=
  Result.fail Error.panic

def boundedListBufMut : bytes.buf.buf_mut.BufMut BoundedWriterState where
  remaining_mut := fun _ => Result.fail Error.panic
  advance_mut := fun _ _ => Result.fail Error.panic
  has_remaining_mut := fun _ => Result.fail Error.panic
  chunk_mut := fun _ => Result.fail Error.panic
  put := fun {_} _ _ _ => Result.fail Error.panic
  put_slice := boundedListPutSlice
  put_bytes := fun _ _ _ => Result.fail Error.panic
  put_u8 := rejectBoundedWriterCall
  put_i8 := rejectBoundedWriterCall
  put_u16 := rejectBoundedWriterCall
  put_u16_le := rejectBoundedWriterCall
  put_u16_ne := rejectBoundedWriterCall
  put_i16 := rejectBoundedWriterCall
  put_i16_le := rejectBoundedWriterCall
  put_i16_ne := rejectBoundedWriterCall
  put_u32 := rejectBoundedWriterCall
  put_u32_le := rejectBoundedWriterCall
  put_u32_ne := rejectBoundedWriterCall
  put_i32 := rejectBoundedWriterCall
  put_i32_le := rejectBoundedWriterCall
  put_i32_ne := rejectBoundedWriterCall
  put_u64 := rejectBoundedWriterCall
  put_u64_le := rejectBoundedWriterCall
  put_u64_ne := rejectBoundedWriterCall
  put_i64 := rejectBoundedWriterCall
  put_i64_le := rejectBoundedWriterCall
  put_i64_ne := rejectBoundedWriterCall
  put_u128 := rejectBoundedWriterCall
  put_u128_le := rejectBoundedWriterCall
  put_u128_ne := rejectBoundedWriterCall
  put_i128 := rejectBoundedWriterCall
  put_i128_le := rejectBoundedWriterCall
  put_i128_ne := rejectBoundedWriterCall
  put_uint := fun _ _ _ => Result.fail Error.panic
  put_uint_le := fun _ _ _ => Result.fail Error.panic
  put_uint_ne := fun _ _ _ => Result.fail Error.panic
  put_int := fun _ _ _ => Result.fail Error.panic
  put_int_le := fun _ _ _ => Result.fail Error.panic
  put_int_ne := fun _ _ _ => Result.fail Error.panic
  put_f32 := rejectBoundedWriterCall
  put_f32_le := rejectBoundedWriterCall
  put_f32_ne := rejectBoundedWriterCall
  put_f64 := rejectBoundedWriterCall
  put_f64_le := rejectBoundedWriterCall
  put_f64_ne := rejectBoundedWriterCall
  limit := fun _ _ => Result.fail Error.panic
  writer := fun _ => Result.fail Error.panic
  chain_mut := fun {_} _ _ => Result.fail Error.panic

theorem put_u32_le_bounded_list_model_appends_exact_four
    (state : BoundedWriterState) (value : Std.U32)
    (hcapacity : state.contents.length + 4 ≤ state.capacity) :
    ∃ after,
      bytes.buf.buf_mut.BufMut.put_u32_le.default boundedListBufMut state value = ok after ∧
      after.contents = state.contents ++ (core.num.U32.to_le_bytes value).val ∧
      after.contents.length = state.contents.length + 4 := by
  let encoded := core.num.U32.to_le_bytes value
  have hencoded_length : encoded.val.length = 4 := by
    simp [encoded, core.num.U32.to_le_bytes]
  have hslice_bound : state.contents.length + (Array.to_slice encoded).val.length ≤ state.capacity := by
    simpa [Array.to_slice, encoded, hencoded_length] using hcapacity
  have hwrite :
      bytes.buf.buf_mut.BufMut.put_u32_le.default boundedListBufMut state value =
        ok { state with contents := state.contents ++ encoded.val } := by
    simp [bytes.buf.buf_mut.BufMut.put_u32_le.default, boundedListBufMut,
      boundedListPutSlice, Aeneas.Std.lift, Array.to_slice, encoded, hcapacity]
  refine ⟨{ state with contents := state.contents ++ encoded.val }, hwrite, rfl, ?_⟩
  simp [List.length_append, hencoded_length]

end BytesProofs.Extensions.Writers

#print axioms BytesProofs.Extensions.Writers.chain_mut_default_places_buffers_in_order
#print axioms BytesProofs.Extensions.Writers.has_remaining_mut_iff_nonzero
#print axioms BytesProofs.Extensions.Writers.limit_default_preserves_inner_and_sets_limit
#print axioms BytesProofs.Extensions.Writers.put_f32_appends_be_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_f32_le_appends_le_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_f32_ne_appends_le_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_f64_appends_be_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_f64_le_appends_le_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_f64_ne_appends_le_bit_bytes
#print axioms BytesProofs.Extensions.Writers.put_i128_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i128_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i128_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_i16_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i16_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i16_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_i32_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i32_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i32_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_i64_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i64_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_i64_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_i8_appends_sign_preserving_byte
#print axioms BytesProofs.Extensions.Writers.put_int_ne_uses_little_endian_method
#print axioms BytesProofs.Extensions.Writers.put_u128_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u128_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u128_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_u16_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u16_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u16_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_u32_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u32_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u32_le_bounded_list_model_appends_exact_four
#print axioms BytesProofs.Extensions.Writers.put_u32_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_u64_be_appends_big_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u64_le_appends_little_endian_bytes
#print axioms BytesProofs.Extensions.Writers.put_u64_ne_appends_native_converter_bytes
#print axioms BytesProofs.Extensions.Writers.put_u8_appends_one_byte
#print axioms BytesProofs.Extensions.Writers.put_uint_ne_uses_little_endian_method
#print axioms BytesProofs.Extensions.Writers.uninit_slice_len_is_logical_slice_length
#print axioms BytesProofs.Extensions.Writers.writer_default_returns_factory_result
