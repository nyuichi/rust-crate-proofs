import Byte128Support
import Byte128Math
open Aeneas Aeneas.Std Result Std.WP
open byteorder_read128_roots.byteorder
namespace Byte128ReadProofs
theorem extend_sign128_spec (word : U128) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) :
    extend_sign128 word count
      ⦃ r => r.val = ((UScalar.bv word).setWidth (8*count.val)).toInt ⦄ := by
  unfold extend_sign128
  step
  step
  step
  step
  step
  have hi2 : IScalar.bv i2 = UScalar.bv i1 := by
    rw [i2_post, UScalar.hcast_bv_eq]
    simp
  have hs : shift.val < 128 := by omega
  calc
    r.val = ((IScalar.bv i2).sshiftRight shift.val).toInt := congrArg BitVec.toInt r_post1
    _ = (((UScalar.bv word) <<< shift.val).sshiftRight shift.val).toInt :=
      congrArg (fun v : BitVec 128 => (v.sshiftRight shift.val).toInt) (hi2.trans i1_post1)
    _ = (((UScalar.bv word).setWidth (128-shift.val)).signExtend 128).toInt :=
      congrArg BitVec.toInt (Byte128Math.signed_low_bits _ _ hs)
    _ = ((UScalar.bv word).setWidth (128-shift.val)).toInt := BitVec.toInt_signExtend_of_le (by omega)
    _ = _ := congrArg (fun width : Nat => ((UScalar.bv word).setWidth width).toInt) (by omega)
private theorem signed_setWidth {w : Nat} (word : BitVec 128) (target : BitVec w)
    (hn : word.toNat = target.toNat) : (word.setWidth w).toInt = target.toInt := by
  apply congrArg BitVec.toInt
  apply BitVec.eq_of_toNat_eq
  rw [BitVec.toNat_setWidth, hn]
  exact Nat.mod_eq_of_lt target.isLt
theorem be_read_int128_spec (buf : Slice U8) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (enough : count.val ≤ buf.length) :
    BigEndian.Insts.ByteorderByteOrder.read_int128 buf count
      ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄ := by
  unfold BigEndian.Insts.ByteorderByteOrder.read_int128
  step with Byte128Support.be_read_uint128_spec
  have hpad : (List.replicate (16-count.val) (0#u8) ++ buf.val.take count.val).map U8.bv = List.replicate (16-count.val) (0 : BitVec 8) ++ (buf.val.take count.val).map U8.bv := by simp
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromBEBytes bytes).toNat) hpad
  have hi : i.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toNat :=
    i_post.trans (hd.trans (Byte128Math.zero_padding_be_val _ _))
  step with extend_sign128_spec
  have hlen : ((buf.val.take count.val).map U8.bv).length = count.val := by
    simp [List.length_take, Nat.min_eq_left enough]
  calc
    r.val = ((UScalar.bv i).setWidth (8*count.val)).toInt := r_post
    _ = ((UScalar.bv i).setWidth (8*((buf.val.take count.val).map U8.bv).length)).toInt :=
      congrArg (fun width : Nat => ((UScalar.bv i).setWidth width).toInt) (by rw [hlen])
    _ = _ := signed_setWidth _ _ hi
theorem le_read_int128_spec (buf : Slice U8) (count : Usize)
    (minimum : 1 ≤ count.val) (maximum : count.val ≤ 16) (enough : count.val ≤ buf.length) :
    LittleEndian.Insts.ByteorderByteOrder.read_int128 buf count
      ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄ := by
  unfold LittleEndian.Insts.ByteorderByteOrder.read_int128
  step with Byte128Support.le_read_uint128_spec
  have hpad : (buf.val.take count.val ++ List.replicate (16-count.val) (0#u8)).map U8.bv = (buf.val.take count.val).map U8.bv ++ List.replicate (16-count.val) (0 : BitVec 8) := by simp
  have hd := congrArg (fun bytes : List (BitVec 8) => (BitVec.fromLEBytes bytes).toNat) hpad
  have hi : i.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toNat :=
    i_post.trans (hd.trans (Byte128Math.zero_padding_le_val _ _))
  step with extend_sign128_spec
  have hlen : ((buf.val.take count.val).map U8.bv).length = count.val := by
    simp [List.length_take, Nat.min_eq_left enough]
  calc
    r.val = ((UScalar.bv i).setWidth (8*count.val)).toInt := r_post
    _ = ((UScalar.bv i).setWidth (8*((buf.val.take count.val).map U8.bv).length)).toInt :=
      congrArg (fun width : Nat => ((UScalar.bv i).setWidth width).toInt) (by rw [hlen])
    _ = _ := signed_setWidth _ _ hi
theorem be_read_int128_invalid (buf : Slice U8) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length)) :
    BigEndian.Insts.ByteorderByteOrder.read_int128 buf count = .fail .assertionFailure := by
  unfold BigEndian.Insts.ByteorderByteOrder.read_int128
  rw [Byte128Support.be_read_uint128_invalid buf count bad]
  simp only [Std.bind_fail]

theorem be_read_int128_contract (buf : Slice U8) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.ByteorderByteOrder.read_int128 buf count
        ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      BigEndian.Insts.ByteorderByteOrder.read_int128 buf count = .fail .assertionFailure) := by
  exact ⟨fun h => be_read_int128_spec buf count h.1 h.2.1 h.2.2,
    fun h => be_read_int128_invalid buf count h⟩

theorem le_read_int128_invalid (buf : Slice U8) (count : Usize)
    (bad : ¬ (1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length)) :
    LittleEndian.Insts.ByteorderByteOrder.read_int128 buf count = .fail .assertionFailure := by
  unfold LittleEndian.Insts.ByteorderByteOrder.read_int128
  rw [Byte128Support.le_read_uint128_invalid buf count bad]
  simp only [Std.bind_fail]

theorem le_read_int128_contract (buf : Slice U8) (count : Usize) :
    ((1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.ByteorderByteOrder.read_int128 buf count
        ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take count.val).map U8.bv)).toInt ⦄) ∧
    (¬ (1 ≤ count.val ∧ count.val ≤ 16 ∧ count.val ≤ buf.length) →
      LittleEndian.Insts.ByteorderByteOrder.read_int128 buf count = .fail .assertionFailure) := by
  exact ⟨fun h => le_read_int128_spec buf count h.1 h.2.1 h.2.2,
    fun h => le_read_int128_invalid buf count h⟩

end Byte128ReadProofs
#print axioms Byte128ReadProofs.be_read_int128_contract
#print axioms Byte128ReadProofs.le_read_int128_contract
