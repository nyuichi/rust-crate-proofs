import VariableGenerated.ByteorderLite
open Aeneas Aeneas.Std Result Std.WP
open byteorder_lite
namespace VariableProofs
/-- Byte capacity computed from the leading-zero count after setting the low bit. -/
def requiredBytes64 (n : U64) : Nat :=
  8 - (UScalar.bv n ||| UScalar.bv (1#u64)).leadingZeros / 8

private theorem leading_zeros_val (x : U64) :
    (core.num.U64.leading_zeros x).val = (UScalar.bv x).leadingZeros := by
  have hx : (UScalar.bv x).leadingZeros ≤ 64 := by
    unfold BitVec.leadingZeros
    split <;> simp only [UScalarTy.U64_numBits_eq] <;> omega
  change (BitVec.ofNat UScalarTy.U32.numBits (UScalar.bv x).leadingZeros).toNat = _
  simp only [BitVec.toNat_ofNat, UScalarTy.U32_numBits_eq]
  apply Nat.mod_eq_of_lt
  omega

theorem pack_size_spec (n : U64) :
    pack_size n ⦃ r => r.val = requiredBytes64 n ⦄ := by
  have hz : (UScalar.bv n ||| UScalar.bv (1#u64)).leadingZeros ≤ 64 := by
    unfold BitVec.leadingZeros
    split <;> simp only [UScalarTy.U64_numBits_eq] <;> omega
  unfold pack_size
  step
  step
  have hi1 : i1.val = (UScalar.bv n ||| UScalar.bv (1#u64)).leadingZeros := by
    rw [i1_post, leading_zeros_val]
    exact congrArg BitVec.leadingZeros i_post1
  step
  have hi2 : i2.val = (UScalar.bv n ||| UScalar.bv (1#u64)).leadingZeros / 8 := by
    simpa only [hi1, Nat.shiftRight_eq_div_pow] using i2_post
  step
  rw [UScalar.cast_val_eq]
  have hsmall : i3.val < 2 ^ UScalarTy.Usize.numBits := by scalar_tac
  rw [Nat.mod_eq_of_lt hsmall]
  simp only [requiredBytes64]
  omega
end VariableProofs
#print axioms VariableProofs.pack_size_spec
namespace VariableProofs
theorem be_write_uint_spec (buf : Slice U8) (n : U64) (nbytes : Usize)
    (fit : requiredBytes64 n ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_be_bytes n).val.drop (8 - nbytes.val)) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_uint
  step with pack_size_spec
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  step
  all_goals simp_all [Array.to_slice, core.num.U64.to_be_bytes, List.slice]
  all_goals omega
end VariableProofs
#print axioms VariableProofs.be_write_uint_spec

namespace VariableProofs
def requiredBytes128 (n : U128) : Nat :=
  16 - (UScalar.bv n ||| UScalar.bv (1#u128)).leadingZeros / 8

private theorem leading_zeros128_val (x : U128) :
    (core.num.U128.leading_zeros x).val = (UScalar.bv x).leadingZeros := by
  have hx : (UScalar.bv x).leadingZeros ≤ 128 := by
    unfold BitVec.leadingZeros
    split <;> simp only [UScalarTy.U128_numBits_eq] <;> omega
  change (BitVec.ofNat UScalarTy.U32.numBits (UScalar.bv x).leadingZeros).toNat = _
  simp only [BitVec.toNat_ofNat, UScalarTy.U32_numBits_eq]
  apply Nat.mod_eq_of_lt
  omega

theorem pack_size128_spec (n : U128) :
    pack_size128 n ⦃ r => r.val = requiredBytes128 n ⦄ := by
  have hz : (UScalar.bv n ||| UScalar.bv (1#u128)).leadingZeros ≤ 128 := by
    unfold BitVec.leadingZeros
    split <;> simp only [UScalarTy.U128_numBits_eq] <;> omega
  unfold pack_size128
  step
  step
  have hi1 : i1.val = (UScalar.bv n ||| UScalar.bv (1#u128)).leadingZeros := by
    rw [i1_post, leading_zeros128_val]
    exact congrArg BitVec.leadingZeros i_post1
  step
  have hi2 : i2.val = (UScalar.bv n ||| UScalar.bv (1#u128)).leadingZeros / 8 := by
    simpa only [hi1, Nat.shiftRight_eq_div_pow] using i2_post
  step
  rw [UScalar.cast_val_eq]
  have hsmall : i3.val < 2 ^ UScalarTy.Usize.numBits := by scalar_tac
  rw [Nat.mod_eq_of_lt hsmall]
  simp only [requiredBytes128]
  omega

theorem le_write_uint_spec (buf : Slice U8) (n : U64) (nbytes : Usize)
    (fit : requiredBytes64 n ≤ nbytes.val) (maximum : nbytes.val ≤ 8)
    (buffer : nbytes.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U64.to_le_bytes n).val.take nbytes.val) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_uint
  step with pack_size_spec
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  all_goals simp_all [Array.to_slice, core.num.U64.to_le_bytes, List.slice]
  all_goals omega

theorem be_write_uint128_spec (buf : Slice U8) (n : U128) (nbytes : Usize)
    (fit : requiredBytes128 n ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_be_bytes n).val.drop (16 - nbytes.val)) ⦄ := by
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_uint128
  step with pack_size128_spec
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  step
  all_goals simp_all [Array.to_slice, core.num.U128.to_be_bytes, List.slice]
  all_goals omega

theorem le_write_uint128_spec (buf : Slice U8) (n : U128) (nbytes : Usize)
    (fit : requiredBytes128 n ≤ nbytes.val) (maximum : nbytes.val ≤ 16)
    (buffer : nbytes.val ≤ buf.length) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0
        ((core.num.U128.to_le_bytes n).val.take nbytes.val) ⦄ := by
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128
  step with pack_size128_spec
  step
  step
  step
  step as ⟨s, back, hs, hlen, hback⟩
  step
  step
  step
  all_goals simp_all [Array.to_slice, core.num.U128.to_le_bytes, List.slice]
  all_goals omega

end VariableProofs
#print axioms VariableProofs.be_write_uint_spec
#print axioms VariableProofs.be_write_uint128_spec
#print axioms VariableProofs.le_write_uint_spec
#print axioms VariableProofs.le_write_uint128_spec
namespace VariableProofs
private theorem normal_result_exists {T : Type} {m : Result T} {post : T → Prop}
    (h : m ⦃ post ⦄) : ∃ value, m = .ok value ∧ post value := by
  cases m using Result.cases with
  | ret value => exact ⟨value, rfl, (spec_ok value).mp h⟩
  | vis effect k => simp only [spec_vis] at h
  | div => simp only [spec_div] at h

theorem be_write_uint_invalid (buf : Slice U8) (n : U64) (nbytes : Usize)
    (bad : ¬ (requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes = .fail .assertionFailure := by
  obtain ⟨packed, hp, hpv⟩ := normal_result_exists (pack_size_spec n)
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_uint
  rw [hp]
  simp only [Std.bind_ok, massert]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
end VariableProofs
#print axioms VariableProofs.be_write_uint_invalid

namespace VariableProofs
theorem be_write_uint128_invalid (buf : Slice U8) (n : U128) (nbytes : Usize)
    (bad : ¬ (requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure := by
  obtain ⟨packed, hp, hpv⟩ := normal_result_exists (pack_size128_spec n)
  unfold BigEndian.Insts.Byteorder_liteByteOrder.write_uint128
  rw [hp]
  simp only [Std.bind_ok, massert]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]

theorem le_write_uint_invalid (buf : Slice U8) (n : U64) (nbytes : Usize)
    (bad : ¬ (requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes = .fail .assertionFailure := by
  obtain ⟨packed, hp, hpv⟩ := normal_result_exists (pack_size_spec n)
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_uint
  rw [hp]
  simp only [Std.bind_ok, massert]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]

theorem le_write_uint128_invalid (buf : Slice U8) (n : U128) (nbytes : Usize)
    (bad : ¬ (requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length)) :
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure := by
  obtain ⟨packed, hp, hpv⟩ := normal_result_exists (pack_size128_spec n)
  unfold LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128
  rw [hp]
  simp only [Std.bind_ok, massert]
  split
  · split
    · split
      · exfalso
        apply bad
        exact ⟨by scalar_tac, by scalar_tac, by scalar_tac⟩
      · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
    · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]
  · simp only [Std.bind_ok, bind_tc_ok, Std.bind_fail, bind_tc_fail]

theorem be_write_uint_contract (buf : Slice U8) (n : U64) (nbytes : Usize) :
  ((requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_be_bytes n).val.drop (8 - nbytes.val)) ⦄) ∧
  (¬ (requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes = .fail .assertionFailure) := by
  exact ⟨fun ⟨fit, maximum, buffer⟩ => be_write_uint_spec buf n nbytes fit maximum buffer,
    fun bad => be_write_uint_invalid buf n nbytes bad⟩
theorem be_write_uint128_contract (buf : Slice U8) (n : U128) (nbytes : Usize) :
  ((requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U128.to_be_bytes n).val.drop (16 - nbytes.val)) ⦄) ∧
  (¬ (requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
    BigEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure) := by
  exact ⟨fun ⟨fit, maximum, buffer⟩ => be_write_uint128_spec buf n nbytes fit maximum buffer,
    fun bad => be_write_uint128_invalid buf n nbytes bad⟩
theorem le_write_uint_contract (buf : Slice U8) (n : U64) (nbytes : Usize) :
  ((requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U64.to_le_bytes n).val.take nbytes.val) ⦄) ∧
  (¬ (requiredBytes64 n ≤ nbytes.val ∧ nbytes.val ≤ 8 ∧ nbytes.val ≤ buf.length) →
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint buf n nbytes = .fail .assertionFailure) := by
  exact ⟨fun ⟨fit, maximum, buffer⟩ => le_write_uint_spec buf n nbytes fit maximum buffer,
    fun bad => le_write_uint_invalid buf n nbytes bad⟩
theorem le_write_uint128_contract (buf : Slice U8) (n : U128) (nbytes : Usize) :
  ((requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes
      ⦃ r => r.val = buf.val.setSlice! 0 ((core.num.U128.to_le_bytes n).val.take nbytes.val) ⦄) ∧
  (¬ (requiredBytes128 n ≤ nbytes.val ∧ nbytes.val ≤ 16 ∧ nbytes.val ≤ buf.length) →
    LittleEndian.Insts.Byteorder_liteByteOrder.write_uint128 buf n nbytes = .fail .assertionFailure) := by
  exact ⟨fun ⟨fit, maximum, buffer⟩ => le_write_uint128_spec buf n nbytes fit maximum buffer,
    fun bad => le_write_uint128_invalid buf n nbytes bad⟩
end VariableProofs
#print axioms VariableProofs.be_write_uint_contract
#print axioms VariableProofs.be_write_uint128_contract
#print axioms VariableProofs.le_write_uint_contract
#print axioms VariableProofs.le_write_uint128_contract
