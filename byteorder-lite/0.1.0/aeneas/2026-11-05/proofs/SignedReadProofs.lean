import SignedReadSupport
import SignedDecodeMath
open Aeneas Aeneas.Std Result Std.WP
open signed_read_roots.byteorder_lite
namespace SignedReadProofs
theorem extend_sign_spec (word : U64) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) :
    extend_sign word count
      ⦃ r => r.val = ((UScalar.bv word).setWidth (8*count.val)).toInt ⦄ := by
  unfold extend_sign
  step
  step
  step
  step
  step
  have hi2 : IScalar.bv i2 = UScalar.bv i1 := by
    rw [i2_post, UScalar.hcast_bv_eq]
    simp
  have hs : shift.val < 64 := by omega
  calc
    r.val = ((IScalar.bv i2).sshiftRight shift.val).toInt := congrArg BitVec.toInt r_post1
    _ = (((UScalar.bv word) <<< shift.val).sshiftRight shift.val).toInt :=
      congrArg (fun v : BitVec 64 => (v.sshiftRight shift.val).toInt) (hi2.trans i1_post1)
    _ = (((UScalar.bv word).setWidth (64-shift.val)).signExtend 64).toInt :=
      congrArg BitVec.toInt (SignedDecodeMath.signed_low_bits _ _ hs)
    _ = ((UScalar.bv word).setWidth (64-shift.val)).toInt := BitVec.toInt_signExtend_of_le (by omega)
    _ = _ := congrArg (fun width : Nat => ((UScalar.bv word).setWidth width).toInt) (by omega)
end SignedReadProofs
#print axioms SignedReadProofs.extend_sign_spec

namespace SignedReadProofs
private theorem signed_setWidth {w : Nat} (word : BitVec 64) (target : BitVec w)
    (hn : word.toNat = target.toNat) : (word.setWidth w).toInt = target.toInt := by
  apply congrArg BitVec.toInt
  apply BitVec.eq_of_toNat_eq
  rw [BitVec.toNat_setWidth, hn]
  exact Nat.mod_eq_of_lt target.isLt
theorem be_read_int_spec (buf : Slice U8) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (enough : count.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_int buf count
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_int
  step with SignedReadSupport.be_read_uint_spec
  have hpad : (List.replicate (8-count.val) (0#u8) ++ buf.val.take count.val).map U8.bv = List.replicate (8-count.val) (0 : BitVec 8) ++ (buf.val.take count.val).map U8.bv := by simp
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromBEBytes bytes).toNat) hpad
  have hi : i.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toNat :=
    i_post.trans (hd.trans (SignedDecodeMath.zero_padding_be_val _ _))
  step with extend_sign_spec
  have hlen : ((buf.val.take count.val).map U8.bv).length = count.val := by
    simp [List.length_take, Nat.min_eq_left enough]
  calc
    r.val = ((UScalar.bv i).setWidth (8*count.val)).toInt := r_post
    _ = ((UScalar.bv i).setWidth (8*((buf.val.take count.val).map U8.bv).length)).toInt :=
      congrArg (fun width : Nat => ((UScalar.bv i).setWidth width).toInt) (by rw [hlen])
    _ = _ := signed_setWidth _ _ hi
theorem le_read_int_spec (buf : Slice U8) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 8) (enough : count.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_int buf count
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_int
  step with SignedReadSupport.le_read_uint_spec
  have hpad : (buf.val.take count.val ++ List.replicate (8-count.val) (0#u8)).map U8.bv = (buf.val.take count.val).map U8.bv ++ List.replicate (8-count.val) (0 : BitVec 8) := by simp
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromLEBytes bytes).toNat) hpad
  have hi : i.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toNat :=
    i_post.trans (hd.trans (SignedDecodeMath.zero_padding_le_val _ _))
  step with extend_sign_spec
  have hlen : ((buf.val.take count.val).map U8.bv).length = count.val := by
    simp [List.length_take, Nat.min_eq_left enough]
  calc
    r.val = ((UScalar.bv i).setWidth (8*count.val)).toInt := r_post
    _ = ((UScalar.bv i).setWidth (8*((buf.val.take count.val).map U8.bv).length)).toInt :=
      congrArg (fun width : Nat => ((UScalar.bv i).setWidth width).toInt) (by rw [hlen])
    _ = _ := signed_setWidth _ _ hi
end SignedReadProofs
#print axioms SignedReadProofs.be_read_int_spec
#print axioms SignedReadProofs.le_read_int_spec

namespace SignedReadProofs
theorem be_read_int_invalid (buf : Slice U8) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_int buf count = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_int
  rw [SignedReadSupport.be_read_uint_invalid buf count bad]
  simp only [Std.bind_fail]

theorem be_read_i24_spec (buf : Slice U8) (enough : 3 ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_i24 buf
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 3).map U8.bv)).toInt ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_i24
  step with be_read_int_spec
  have h3 : (3#usize).val = 3 := by decide
  have hp : (buf.val.take (3#usize).val).map U8.bv = (buf.val.take 3).map U8.bv := by rw [h3]
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromBEBytes bytes).toInt) hp
  have hi := i_post.trans hd
  have hlen : ((buf.val.take 3).map U8.bv).length = 3 := by simp [Nat.min_eq_left enough]
  have hlo := BitVec.le_toInt (BitVec.fromBEBytes ((buf.val.take 3).map U8.bv))
  have hhi := (BitVec.toInt_lt (x := BitVec.fromBEBytes ((buf.val.take 3).map U8.bv)))
  simp only [hlen, Nat.reduceMul, Nat.reduceSub] at hlo hhi
  rw [IScalar.cast_val_eq, hi]
  apply Int.bmod_eq_of_le <;> norm_num [IScalarTy.I32_numBits_eq, IScalarTy.I64_numBits_eq] <;> omega
theorem le_read_int_invalid (buf : Slice U8) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_int buf count = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_int
  rw [SignedReadSupport.le_read_uint_invalid buf count bad]
  simp only [Std.bind_fail]

theorem le_read_i24_spec (buf : Slice U8) (enough : 3 ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_i24 buf
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 3).map U8.bv)).toInt ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_i24
  step with le_read_int_spec
  have h3 : (3#usize).val = 3 := by decide
  have hp : (buf.val.take (3#usize).val).map U8.bv = (buf.val.take 3).map U8.bv := by rw [h3]
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromLEBytes bytes).toInt) hp
  have hi := i_post.trans hd
  have hlen : ((buf.val.take 3).map U8.bv).length = 3 := by simp [Nat.min_eq_left enough]
  have hlo := BitVec.le_toInt (BitVec.fromLEBytes ((buf.val.take 3).map U8.bv))
  have hhi := (BitVec.toInt_lt (x := BitVec.fromLEBytes ((buf.val.take 3).map U8.bv)))
  simp only [hlen, Nat.reduceMul, Nat.reduceSub] at hlo hhi
  rw [IScalar.cast_val_eq, hi]
  apply Int.bmod_eq_of_le <;> norm_num [IScalarTy.I32_numBits_eq, IScalarTy.I64_numBits_eq] <;> omega
end SignedReadProofs
#print axioms SignedReadProofs.be_read_int_invalid
#print axioms SignedReadProofs.be_read_i24_spec
#print axioms SignedReadProofs.le_read_int_invalid
#print axioms SignedReadProofs.le_read_i24_spec

namespace SignedReadProofs
theorem be_read_i24_short (buf : Slice U8) (short : buf.length < 3) :
    BigEndian.Insts.Byteorder_liteByteOrder.read_i24 buf = .fail .assertionFailure := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.read_i24
  have bad : ¬ (1 ≤ (3#usize).val ∧ (3#usize).val ≤ 8 ∧ (3#usize).val ≤ buf.length) := by
    have h3 : (3#usize).val = 3 := by decide
    intro h
    simp only [h3] at h
    omega
  rw [be_read_int_invalid buf (3#usize) bad]
  simp only [Std.bind_fail]

theorem be_read_int_contract (buf : Slice U8) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.read_int buf count
        ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.Byteorder_liteByteOrder.read_int buf count = .fail .assertionFailure) := by
  exact ⟨fun h => be_read_int_spec buf count h.1 h.2.1 h.2.2,
    fun h => be_read_int_invalid buf count h⟩

theorem be_read_i24_contract (buf : Slice U8) :
    ((3 ≤ buf.length) → BigEndian.Insts.Byteorder_liteByteOrder.read_i24 buf
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 3).map U8.bv)).toInt ⦄) ∧
    (buf.length < 3 → BigEndian.Insts.Byteorder_liteByteOrder.read_i24 buf = .fail .assertionFailure) := by
  exact ⟨fun h => be_read_i24_spec buf h, fun h => be_read_i24_short buf h⟩
theorem le_read_i24_short (buf : Slice U8) (short : buf.length < 3) :
    LittleEndian.Insts.Byteorder_liteByteOrder.read_i24 buf = .fail .assertionFailure := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.read_i24
  have bad : ¬ (1 ≤ (3#usize).val ∧ (3#usize).val ≤ 8 ∧ (3#usize).val ≤ buf.length) := by
    have h3 : (3#usize).val = 3 := by decide
    intro h
    simp only [h3] at h
    omega
  rw [le_read_int_invalid buf (3#usize) bad]
  simp only [Std.bind_fail]

theorem le_read_int_contract (buf : Slice U8) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.read_int buf count
        ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 8 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.Byteorder_liteByteOrder.read_int buf count = .fail .assertionFailure) := by
  exact ⟨fun h => le_read_int_spec buf count h.1 h.2.1 h.2.2,
    fun h => le_read_int_invalid buf count h⟩

theorem le_read_i24_contract (buf : Slice U8) :
    ((3 ≤ buf.length) → LittleEndian.Insts.Byteorder_liteByteOrder.read_i24 buf
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 3).map U8.bv)).toInt ⦄) ∧
    (buf.length < 3 → LittleEndian.Insts.Byteorder_liteByteOrder.read_i24 buf = .fail .assertionFailure) := by
  exact ⟨fun h => le_read_i24_spec buf h, fun h => le_read_i24_short buf h⟩
end SignedReadProofs
#print axioms SignedReadProofs.be_read_int_contract
#print axioms SignedReadProofs.be_read_i24_contract
#print axioms SignedReadProofs.le_read_int_contract
#print axioms SignedReadProofs.le_read_i24_contract
