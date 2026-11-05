import EndianVerificationDriver.Funs
open Aeneas Aeneas.Std Result
namespace ModelProofs
set_option linter.unusedSimpArgs false
/-- Reverse complete 8-bit byte groups, retaining the word width. -/
def byteReverse {n : Nat} (x : BitVec n) : BitVec n :=
  BitVec.ofNat n (BitVec.fromLEBytes x.toBEBytes).toNat

lemma cast_eq_ofNat {n m : Nat} (h : n = m) (x : BitVec n) :
    x.cast h = BitVec.ofNat m x.toNat := by
  subst m
  simp
theorem u16_swap_model (x : U16) :
    core.num.U16.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : U16) := by
  unfold core.num.U16.swap_bytes
  apply congrArg (fun (v : U16) => (.ok v : Result U16))
  apply U16.bv_eq_imp_eq
  have hb : (core.num.U16.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.U16.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.U16.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 16 (BitVec.fromLEBytes bs).toNat) hm
theorem u16_to_be_model (x : U16) :
    core.num.U16.to_be x = .ok (⟨byteReverse x.bv⟩ : U16) := by
  simpa only [core.num.U16.to_be, core.num.U16.swap_bytes] using u16_swap_model x

theorem u16_from_be_model (x : U16) :
    core.num.U16.from_be x = .ok (⟨byteReverse x.bv⟩ : U16) := by
  simpa only [core.num.U16.from_be, core.num.U16.swap_bytes] using u16_swap_model x

theorem u16_to_le_model (x : U16) : core.num.U16.to_le x = .ok x := by rfl

theorem u16_from_le_model (x : U16) : core.num.U16.from_le x = .ok x := by rfl

theorem u32_swap_model (x : U32) :
    core.num.U32.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : U32) := by
  unfold core.num.U32.swap_bytes
  apply congrArg (fun (v : U32) => (.ok v : Result U32))
  apply U32.bv_eq_imp_eq
  have hb : (core.num.U32.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.U32.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.U32.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 32 (BitVec.fromLEBytes bs).toNat) hm
theorem u32_to_be_model (x : U32) :
    core.num.U32.to_be x = .ok (⟨byteReverse x.bv⟩ : U32) := by
  simpa only [core.num.U32.to_be, core.num.U32.swap_bytes] using u32_swap_model x

theorem u32_from_be_model (x : U32) :
    core.num.U32.from_be x = .ok (⟨byteReverse x.bv⟩ : U32) := by
  simpa only [core.num.U32.from_be, core.num.U32.swap_bytes] using u32_swap_model x

theorem u32_to_le_model (x : U32) : core.num.U32.to_le x = .ok x := by rfl

theorem u32_from_le_model (x : U32) : core.num.U32.from_le x = .ok x := by rfl

theorem u64_swap_model (x : U64) :
    core.num.U64.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : U64) := by
  unfold core.num.U64.swap_bytes
  apply congrArg (fun (v : U64) => (.ok v : Result U64))
  apply U64.bv_eq_imp_eq
  have hb : (core.num.U64.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.U64.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.U64.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 64 (BitVec.fromLEBytes bs).toNat) hm
theorem u64_to_be_model (x : U64) :
    core.num.U64.to_be x = .ok (⟨byteReverse x.bv⟩ : U64) := by
  simpa only [core.num.U64.to_be, core.num.U64.swap_bytes] using u64_swap_model x

theorem u64_from_be_model (x : U64) :
    core.num.U64.from_be x = .ok (⟨byteReverse x.bv⟩ : U64) := by
  simpa only [core.num.U64.from_be, core.num.U64.swap_bytes] using u64_swap_model x

theorem u64_to_le_model (x : U64) : core.num.U64.to_le x = .ok x := by rfl

theorem u64_from_le_model (x : U64) : core.num.U64.from_le x = .ok x := by rfl

theorem u128_swap_model (x : U128) :
    core.num.U128.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : U128) := by
  unfold core.num.U128.swap_bytes
  apply congrArg (fun (v : U128) => (.ok v : Result U128))
  apply U128.bv_eq_imp_eq
  have hb : (core.num.U128.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.U128.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.U128.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 128 (BitVec.fromLEBytes bs).toNat) hm
theorem u128_to_be_model (x : U128) :
    core.num.U128.to_be x = .ok (⟨byteReverse x.bv⟩ : U128) := by
  simpa only [core.num.U128.to_be, core.num.U128.swap_bytes] using u128_swap_model x

theorem u128_from_be_model (x : U128) :
    core.num.U128.from_be x = .ok (⟨byteReverse x.bv⟩ : U128) := by
  simpa only [core.num.U128.from_be, core.num.U128.swap_bytes] using u128_swap_model x

theorem u128_to_le_model (x : U128) : core.num.U128.to_le x = .ok x := by rfl

theorem u128_from_le_model (x : U128) : core.num.U128.from_le x = .ok x := by rfl

theorem i16_swap_model (x : I16) :
    core.num.I16.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : I16) := by
  unfold core.num.I16.swap_bytes
  apply congrArg (fun (v : I16) => (.ok v : Result I16))
  apply I16.bv_eq_imp_eq
  have hb : (core.num.I16.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.I16.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.I16.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 16 (BitVec.fromLEBytes bs).toNat) hm
theorem i16_to_be_model (x : I16) :
    core.num.I16.to_be x = .ok (⟨byteReverse x.bv⟩ : I16) := by
  simpa only [core.num.I16.to_be, core.num.I16.swap_bytes] using i16_swap_model x

theorem i16_from_be_model (x : I16) :
    core.num.I16.from_be x = .ok (⟨byteReverse x.bv⟩ : I16) := by
  simpa only [core.num.I16.from_be, core.num.I16.swap_bytes] using i16_swap_model x

theorem i16_to_le_model (x : I16) : core.num.I16.to_le x = .ok x := by rfl

theorem i16_from_le_model (x : I16) : core.num.I16.from_le x = .ok x := by rfl

theorem i32_swap_model (x : I32) :
    core.num.I32.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : I32) := by
  unfold core.num.I32.swap_bytes
  apply congrArg (fun (v : I32) => (.ok v : Result I32))
  apply I32.bv_eq_imp_eq
  have hb : (core.num.I32.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.I32.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.I32.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 32 (BitVec.fromLEBytes bs).toNat) hm
theorem i32_to_be_model (x : I32) :
    core.num.I32.to_be x = .ok (⟨byteReverse x.bv⟩ : I32) := by
  simpa only [core.num.I32.to_be, core.num.I32.swap_bytes] using i32_swap_model x

theorem i32_from_be_model (x : I32) :
    core.num.I32.from_be x = .ok (⟨byteReverse x.bv⟩ : I32) := by
  simpa only [core.num.I32.from_be, core.num.I32.swap_bytes] using i32_swap_model x

theorem i32_to_le_model (x : I32) : core.num.I32.to_le x = .ok x := by rfl

theorem i32_from_le_model (x : I32) : core.num.I32.from_le x = .ok x := by rfl

theorem i64_swap_model (x : I64) :
    core.num.I64.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : I64) := by
  unfold core.num.I64.swap_bytes
  apply congrArg (fun (v : I64) => (.ok v : Result I64))
  apply I64.bv_eq_imp_eq
  have hb : (core.num.I64.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.I64.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.I64.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 64 (BitVec.fromLEBytes bs).toNat) hm
theorem i64_to_be_model (x : I64) :
    core.num.I64.to_be x = .ok (⟨byteReverse x.bv⟩ : I64) := by
  simpa only [core.num.I64.to_be, core.num.I64.swap_bytes] using i64_swap_model x

theorem i64_from_be_model (x : I64) :
    core.num.I64.from_be x = .ok (⟨byteReverse x.bv⟩ : I64) := by
  simpa only [core.num.I64.from_be, core.num.I64.swap_bytes] using i64_swap_model x

theorem i64_to_le_model (x : I64) : core.num.I64.to_le x = .ok x := by rfl

theorem i64_from_le_model (x : I64) : core.num.I64.from_le x = .ok x := by rfl

theorem i128_swap_model (x : I128) :
    core.num.I128.swap_bytes x = .ok (⟨byteReverse x.bv⟩ : I128) := by
  unfold core.num.I128.swap_bytes
  apply congrArg (fun (v : I128) => (.ok v : Result I128))
  apply I128.bv_eq_imp_eq
  have hb : (core.num.I128.to_be_bytes x).val =
      x.bv.toBEBytes.map (fun b => (⟨b⟩ : U8)) := by
    exact Aeneas.Std.Array.from_val _ _
  change (BitVec.fromLEBytes ((core.num.I128.to_be_bytes x).val.map U8.bv)).cast (by simp) = _
  have hm : (core.num.I128.to_be_bytes x).val.map U8.bv = x.bv.toBEBytes := by
    rw [hb]
    simp only [List.map_map, Function.comp_def, U8.bv]
    exact List.map_id _
  rw [cast_eq_ofNat]
  exact congrArg (fun bs => BitVec.ofNat 128 (BitVec.fromLEBytes bs).toNat) hm
theorem i128_to_be_model (x : I128) :
    core.num.I128.to_be x = .ok (⟨byteReverse x.bv⟩ : I128) := by
  simpa only [core.num.I128.to_be, core.num.I128.swap_bytes] using i128_swap_model x

theorem i128_from_be_model (x : I128) :
    core.num.I128.from_be x = .ok (⟨byteReverse x.bv⟩ : I128) := by
  simpa only [core.num.I128.from_be, core.num.I128.swap_bytes] using i128_swap_model x

theorem i128_to_le_model (x : I128) : core.num.I128.to_le x = .ok x := by rfl

theorem i128_from_le_model (x : I128) : core.num.I128.from_le x = .ok x := by rfl

/-- 64-bit pointer-sized bit-pattern interpretation; public theorems require h64. -/
def reverseUsize (x : Usize) : Usize :=
  UScalar.cast .Usize (⟨byteReverse (UScalar.cast .U64 x).bv⟩ : U64)

theorem usize_swap_model (x : Usize) (h64 : System.Platform.numBits = 64) :
    core.num.Usize.swap_bytes x = .ok (reverseUsize x) := by
  simp only [core.num.Usize.swap_bytes, h64, ↓reduceIte]
  have hv : core.num.U64.from_le_bytes (core.num.U64.to_be_bytes (UScalar.cast .U64 x)) =
      (⟨byteReverse (UScalar.cast .U64 x).bv⟩ : U64) :=
    Result.ok_injective (u64_swap_model (UScalar.cast .U64 x))
  exact congrArg (fun v => (.ok (UScalar.cast .Usize v) : Result Usize)) hv

theorem usize_to_be_model (x : Usize) (h64 : System.Platform.numBits = 64) :
    core.num.Usize.to_be x = .ok (reverseUsize x) := by
  simpa only [core.num.Usize.to_be, core.num.Usize.swap_bytes] using usize_swap_model x h64

theorem usize_from_be_model (x : Usize) (h64 : System.Platform.numBits = 64) :
    core.num.Usize.from_be x = .ok (reverseUsize x) := by
  simpa only [core.num.Usize.from_be, core.num.Usize.swap_bytes] using usize_swap_model x h64

theorem usize_to_le_model (x : Usize) : core.num.Usize.to_le x = .ok x := by rfl

theorem usize_from_le_model (x : Usize) : core.num.Usize.from_le x = .ok x := by rfl

/-- 64-bit pointer-sized bit-pattern interpretation; public theorems require h64. -/
def reverseIsize (x : Isize) : Isize :=
  IScalar.cast .Isize (⟨byteReverse (IScalar.cast .I64 x).bv⟩ : I64)

theorem isize_swap_model (x : Isize) (h64 : System.Platform.numBits = 64) :
    core.num.Isize.swap_bytes x = .ok (reverseIsize x) := by
  simp only [core.num.Isize.swap_bytes, h64, ↓reduceIte]
  have hv : core.num.I64.from_le_bytes (core.num.I64.to_be_bytes (IScalar.cast .I64 x)) =
      (⟨byteReverse (IScalar.cast .I64 x).bv⟩ : I64) :=
    Result.ok_injective (i64_swap_model (IScalar.cast .I64 x))
  exact congrArg (fun v => (.ok (IScalar.cast .Isize v) : Result Isize)) hv

theorem isize_to_be_model (x : Isize) (h64 : System.Platform.numBits = 64) :
    core.num.Isize.to_be x = .ok (reverseIsize x) := by
  simpa only [core.num.Isize.to_be, core.num.Isize.swap_bytes] using isize_swap_model x h64

theorem isize_from_be_model (x : Isize) (h64 : System.Platform.numBits = 64) :
    core.num.Isize.from_be x = .ok (reverseIsize x) := by
  simpa only [core.num.Isize.from_be, core.num.Isize.swap_bytes] using isize_swap_model x h64

theorem isize_to_le_model (x : Isize) : core.num.Isize.to_le x = .ok x := by rfl

theorem isize_from_le_model (x : Isize) : core.num.Isize.from_le x = .ok x := by rfl

end ModelProofs
#print axioms ModelProofs.u16_swap_model
#print axioms ModelProofs.u16_to_be_model
#print axioms ModelProofs.u16_from_be_model
#print axioms ModelProofs.u16_to_le_model
#print axioms ModelProofs.u16_from_le_model
#print axioms ModelProofs.u32_swap_model
#print axioms ModelProofs.u32_to_be_model
#print axioms ModelProofs.u32_from_be_model
#print axioms ModelProofs.u32_to_le_model
#print axioms ModelProofs.u32_from_le_model
#print axioms ModelProofs.u64_swap_model
#print axioms ModelProofs.u64_to_be_model
#print axioms ModelProofs.u64_from_be_model
#print axioms ModelProofs.u64_to_le_model
#print axioms ModelProofs.u64_from_le_model
#print axioms ModelProofs.u128_swap_model
#print axioms ModelProofs.u128_to_be_model
#print axioms ModelProofs.u128_from_be_model
#print axioms ModelProofs.u128_to_le_model
#print axioms ModelProofs.u128_from_le_model
#print axioms ModelProofs.i16_swap_model
#print axioms ModelProofs.i16_to_be_model
#print axioms ModelProofs.i16_from_be_model
#print axioms ModelProofs.i16_to_le_model
#print axioms ModelProofs.i16_from_le_model
#print axioms ModelProofs.i32_swap_model
#print axioms ModelProofs.i32_to_be_model
#print axioms ModelProofs.i32_from_be_model
#print axioms ModelProofs.i32_to_le_model
#print axioms ModelProofs.i32_from_le_model
#print axioms ModelProofs.i64_swap_model
#print axioms ModelProofs.i64_to_be_model
#print axioms ModelProofs.i64_from_be_model
#print axioms ModelProofs.i64_to_le_model
#print axioms ModelProofs.i64_from_le_model
#print axioms ModelProofs.i128_swap_model
#print axioms ModelProofs.i128_to_be_model
#print axioms ModelProofs.i128_from_be_model
#print axioms ModelProofs.i128_to_le_model
#print axioms ModelProofs.i128_from_le_model
#print axioms ModelProofs.usize_swap_model
#print axioms ModelProofs.usize_to_be_model
#print axioms ModelProofs.usize_from_be_model
#print axioms ModelProofs.usize_to_le_model
#print axioms ModelProofs.usize_from_le_model
#print axioms ModelProofs.isize_swap_model
#print axioms ModelProofs.isize_to_be_model
#print axioms ModelProofs.isize_from_be_model
#print axioms ModelProofs.isize_to_le_model
#print axioms ModelProofs.isize_from_le_model
