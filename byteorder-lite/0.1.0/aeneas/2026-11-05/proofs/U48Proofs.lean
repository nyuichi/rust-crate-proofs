import ByteorderLiteU48Roots.Funs
import ReadVariableProofs
import VariableProofs
import SignedReadProofs
import SignedVariableContracts
open Aeneas Aeneas.Std Result Std.WP
namespace U48Proofs
theorem be_read_u48_body (buf : Slice U8) : byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_uint buf 6#usize := by rfl
theorem be_write_u48_body (buf : Slice U8) (n : U64) : byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_uint buf n 6#usize := by rfl
theorem be_read_i48_body (buf : Slice U8) : byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = signed_read_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_int buf 6#usize := by rfl
theorem be_write_i48_body (buf : Slice U8) (n : I64) : byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = signed_variable_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_int buf n 6#usize := by rfl
theorem le_read_u48_body (buf : Slice U8) : byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_uint buf 6#usize := by rfl
theorem le_write_u48_body (buf : Slice U8) (n : U64) : byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_uint buf n 6#usize := by rfl
theorem le_read_i48_body (buf : Slice U8) : byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = signed_read_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_int buf 6#usize := by rfl
theorem le_write_i48_body (buf : Slice U8) (n : I64) : byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = signed_variable_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_int buf n 6#usize := by rfl
theorem be_read_u48_contract (buf : Slice U8) :
    (∀ (minimum : 1 ≤ (6#usize).val) (maximum : (6#usize).val ≤ 8)
       (buffer : (6#usize).val ≤ buf.length), byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf
      ⦃ r => r.val = (BitVec.fromBEBytes
        ((List.replicate (8 - (6#usize).val) (0#u8) ++ buf.val.take (6#usize).val).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = .fail .assertionFailure) := by
  simpa only [be_read_u48_body] using ReadVariableProofs.be_read_uint_contract buf (6#usize)
theorem be_write_u48_contract (buf : Slice U8) (n : U64) :
  ((VariableProofs.requiredBytes64 n ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
    byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_be_bytes n).val.drop (8 - (6#usize).val)) ⦄) ∧
  (¬ (VariableProofs.requiredBytes64 n ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
    byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = .fail .assertionFailure) := by
  simpa only [be_write_u48_body] using VariableProofs.be_write_uint_contract buf n (6#usize)
theorem be_read_i48_contract (buf : Slice U8) :
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf
        ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take (6#usize).val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
  simpa only [be_read_i48_body] using SignedReadProofs.be_read_int_contract buf (6#usize)
theorem be_write_i48_contract (buf : Slice U8) (n : I64) :
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_be_bytes (SignedWrites.truncatedWord n (6#usize))).val.drop (8-(6#usize).val)) ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .integerOverflow) ∧
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ buf.length < (6#usize).val) →
      byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .assertionFailure) := by
  simpa only [be_write_i48_body] using SignedWrites.be_write_int_contract buf n (6#usize)
theorem le_read_u48_contract (buf : Slice U8) :
    (∀ (minimum : 1 ≤ (6#usize).val) (maximum : (6#usize).val ≤ 8)
       (buffer : (6#usize).val ≤ buf.length), byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf
      ⦃ r => r.val = (BitVec.fromLEBytes
        ((buf.val.take (6#usize).val ++ List.replicate (8 - (6#usize).val) (0#u8)).map U8.bv)).toNat ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = .fail .assertionFailure) := by
  simpa only [le_read_u48_body] using ReadVariableProofs.le_read_uint_contract buf (6#usize)
theorem le_write_u48_contract (buf : Slice U8) (n : U64) :
  ((VariableProofs.requiredBytes64 n ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
    byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_le_bytes n).val.take (6#usize).val) ⦄) ∧
  (¬ (VariableProofs.requiredBytes64 n ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
    byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = .fail .assertionFailure) := by
  simpa only [le_write_u48_body] using VariableProofs.le_write_uint_contract buf n (6#usize)
theorem le_read_i48_contract (buf : Slice U8) :
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf
        ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take (6#usize).val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
  simpa only [le_read_i48_body] using SignedReadProofs.le_read_int_contract buf (6#usize)
theorem le_write_i48_contract (buf : Slice U8) (n : I64) :
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ (6#usize).val ≤ buf.length) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n
        ⦃ r => r.val = buf.val.setSlice! 0
          ((core.num.U64.to_le_bytes (SignedWrites.truncatedWord n (6#usize))).val.take (6#usize).val) ⦄) ∧
    (¬ (1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .integerOverflow) ∧
    ((1 ≤ (6#usize).val ∧ (6#usize).val ≤ 8 ∧ buf.length < (6#usize).val) →
      byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .assertionFailure) := by
  simpa only [le_write_i48_body] using SignedWrites.le_write_int_contract buf n (6#usize)
theorem fits48 (n : U64) : VariableProofs.requiredBytes64 n ≤ 6 ↔ n.val < 2^48 := by
  let x := UScalar.bv n
  let y := x ||| UScalar.bv (1#u64)
  have hy : y.toNat = n.val ||| 1 := by
    simp only [y, x, BitVec.toNat_or]
    rfl
  have hypos : y.toNat ≠ 0 := by
    have h : 1 ≤ n.val ||| 1 := Nat.right_le_or
    omega
  have hyzero : y ≠ 0 := by
    intro h
    apply hypos
    rw [h]
    rfl
  have hylog : Nat.log 2 y.toNat < 64 := by
    apply Nat.log_lt_of_lt_pow hypos
    exact y.isLt
  have hor : y.toNat < 2^48 ↔ n.val < 2^48 := by
    rw [hy]
    constructor
    · intro h
      exact lt_of_le_of_lt Nat.left_le_or h
    · intro h
      exact Nat.or_lt_two_pow h (by decide)
  have hl : Nat.log 2 y.toNat < 48 ↔ y.toNat < 2^48 :=
    Nat.log_lt_iff_lt_pow (by decide) hypos
  change 8 - y.leadingZeros / 8 ≤ 6 ↔ _
  rw [BitVec.leadingZeros, if_neg hyzero]
  simp only [UScalarTy.U64_numBits_eq] at *
  omega

theorem be_write_u48_plain (buf : Slice U8) (n : U64) :
 ((n.val < 2^48 ∧ 6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n
 ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_be_bytes n).val.drop 2) ⦄) ∧
 (¬ (n.val < 2^48 ∧ 6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = .fail .assertionFailure) := by
 have h6 : (6#usize).val = 6 := by decide
 simpa only [h6, Nat.reduceSub, fits48, Nat.reduceLeDiff, and_true, true_and] using be_write_u48_contract buf n
theorem be_read_i48_plain (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take (6#usize).val).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
 have h6 : (6#usize).val = 6 := by decide
 simpa only [h6, Nat.reduceLeDiff, true_and, Nat.not_le] using be_read_i48_contract buf
theorem le_write_u48_plain (buf : Slice U8) (n : U64) :
 ((n.val < 2^48 ∧ 6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n
 ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_le_bytes n).val.take 6) ⦄) ∧
 (¬ (n.val < 2^48 ∧ 6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_u48 buf n = .fail .assertionFailure) := by
 have h6 : (6#usize).val = 6 := by decide
 simpa only [h6, Nat.reduceSub, fits48, Nat.reduceLeDiff, and_true, true_and] using le_write_u48_contract buf n
theorem le_read_i48_plain (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take (6#usize).val).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
 have h6 : (6#usize).val = 6 := by decide
 simpa only [h6, Nat.reduceLeDiff, true_and, Nat.not_le] using le_read_i48_contract buf
theorem signed_low48 (n : I64) : (SignedWrites.truncatedWord n (6#usize)).val = (IScalar.bv n).toNat % 2^48 := by
 simpa using SignedWrites.truncatedWord_val n (6#usize) (by decide)
theorem be_read_u48_exact (buf : Slice U8) (enough : 6 ≤ buf.length) :
 byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄ := by
 have h6 : (6#usize).val = 6 := by decide
 have h := (be_read_u48_contract buf).1 (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have hp : (List.replicate 2 (0#u8) ++ buf.val.take (6#usize).val).map U8.bv = List.replicate 2 (0 : BitVec 8) ++ (buf.val.take (6#usize).val).map U8.bv := by simp
 have hd := congrArg (fun xs : List (BitVec 8) => (BitVec.fromBEBytes xs).toNat) hp
 have hx := hr.trans (hd.trans (SignedDecodeMath.zero_padding_be_val _ _))
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 have hdecode := congrArg (fun xs : List U8 => (BitVec.fromBEBytes (xs.map U8.bv)).toNat) he
 have hex := hx.trans hdecode
 refine ⟨hex, ?_⟩
 rw [hex]
 have hb := (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).isLt
 have hl : ((buf.val.take 6).map U8.bv).length = 6 := by simp [List.length_take, Nat.min_eq_left enough]
 simpa only [hl] using hb

theorem be_read_i48_exact (buf : Slice U8) (enough : 6 ≤ buf.length) :
 byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄ := by
 have h := (be_read_i48_plain buf).1 enough
 apply spec_mono h
 intro r hr
 have h6 : (6#usize).val = 6 := by decide
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 exact hr.trans (congrArg (fun xs : List U8 => (BitVec.fromBEBytes (xs.map U8.bv)).toInt) he)
theorem le_read_u48_exact (buf : Slice U8) (enough : 6 ≤ buf.length) :
 byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄ := by
 have h6 : (6#usize).val = 6 := by decide
 have h := (le_read_u48_contract buf).1 (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have hp : (buf.val.take (6#usize).val ++ List.replicate 2 (0#u8)).map U8.bv = (buf.val.take (6#usize).val).map U8.bv ++ List.replicate 2 (0 : BitVec 8) := by simp
 have hd := congrArg (fun xs : List (BitVec 8) => (BitVec.fromLEBytes xs).toNat) hp
 have hx := hr.trans (hd.trans (SignedDecodeMath.zero_padding_le_val _ _))
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 have hdecode := congrArg (fun xs : List U8 => (BitVec.fromLEBytes (xs.map U8.bv)).toNat) he
 have hex := hx.trans hdecode
 refine ⟨hex, ?_⟩
 rw [hex]
 have hb := (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).isLt
 have hl : ((buf.val.take 6).map U8.bv).length = 6 := by simp [List.length_take, Nat.min_eq_left enough]
 simpa only [hl] using hb

theorem le_read_i48_exact (buf : Slice U8) (enough : 6 ≤ buf.length) :
 byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄ := by
 have h := (le_read_i48_plain buf).1 enough
 apply spec_mono h
 intro r hr
 have h6 : (6#usize).val = 6 := by decide
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 exact hr.trans (congrArg (fun xs : List U8 => (BitVec.fromLEBytes (xs.map U8.bv)).toInt) he)
theorem be_read_u48_final (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = .fail .assertionFailure) := by
 refine ⟨be_read_u48_exact buf, ?_⟩
 intro short
 apply (be_read_u48_contract buf).2
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem be_read_i48_final (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
 refine ⟨be_read_i48_exact buf, ?_⟩
 intro short
 apply (be_read_i48_contract buf).2
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem be_write_i48_final (buf : Slice U8) (n : I64) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n ⦃ r => r.val = buf.val.setSlice! 0
 ((core.num.U64.to_be_bytes (SignedWrites.truncatedWord n (6#usize))).val.drop 2) ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.BigEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .assertionFailure) ∧
 ((SignedWrites.truncatedWord n (6#usize)).val = (IScalar.bv n).toNat % 2^48) := by
 have h6 : (6#usize).val = 6 := by decide
 refine ⟨?_, ?_, signed_low48 n⟩
 · intro enough
   have h := (be_write_i48_contract buf n).1 (by simp only [h6]; omega)
   simpa only [h6, Nat.reduceSub] using h
 · intro short
   exact (be_write_i48_contract buf n).2.2 (by simp only [h6]; omega)
theorem le_read_u48_final (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_u48 buf = .fail .assertionFailure) := by
 refine ⟨le_read_u48_exact buf, ?_⟩
 intro short
 apply (le_read_u48_contract buf).2
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem le_read_i48_final (buf : Slice U8) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.read_i48 buf = .fail .assertionFailure) := by
 refine ⟨le_read_i48_exact buf, ?_⟩
 intro short
 apply (le_read_i48_contract buf).2
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem le_write_i48_final (buf : Slice U8) (n : I64) :
 ((6 ≤ buf.length) → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n ⦃ r => r.val = buf.val.setSlice! 0
 ((core.num.U64.to_le_bytes (SignedWrites.truncatedWord n (6#usize))).val.take 6) ⦄) ∧
 (buf.length < 6 → byteorder_lite_u48_roots.byteorder_lite.LittleEndian.Insts.Byteorder_liteByteOrder.write_i48 buf n = .fail .assertionFailure) ∧
 ((SignedWrites.truncatedWord n (6#usize)).val = (IScalar.bv n).toNat % 2^48) := by
 have h6 : (6#usize).val = 6 := by decide
 refine ⟨?_, ?_, signed_low48 n⟩
 · intro enough
   have h := (le_write_i48_contract buf n).1 (by simp only [h6]; omega)
   simpa only [h6, Nat.reduceSub] using h
 · intro short
   exact (le_write_i48_contract buf n).2.2 (by simp only [h6]; omega)
end U48Proofs
#print axioms U48Proofs.be_read_u48_contract
#print axioms U48Proofs.be_write_u48_contract
#print axioms U48Proofs.be_read_i48_contract
#print axioms U48Proofs.be_write_i48_contract
#print axioms U48Proofs.le_read_u48_contract
#print axioms U48Proofs.le_write_u48_contract
#print axioms U48Proofs.le_read_i48_contract
#print axioms U48Proofs.le_write_i48_contract

#print axioms U48Proofs.fits48
#print axioms U48Proofs.signed_low48

#print axioms U48Proofs.be_read_u48_exact
#print axioms U48Proofs.be_read_i48_exact
#print axioms U48Proofs.be_write_u48_plain
#print axioms U48Proofs.le_read_u48_exact
#print axioms U48Proofs.le_read_i48_exact
#print axioms U48Proofs.le_write_u48_plain

#print axioms U48Proofs.be_read_u48_final
#print axioms U48Proofs.be_read_i48_final
#print axioms U48Proofs.be_write_i48_final
#print axioms U48Proofs.le_read_u48_final
#print axioms U48Proofs.le_read_i48_final
#print axioms U48Proofs.le_write_i48_final
