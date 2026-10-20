import extensions.numeric.Generated
open Aeneas Aeneas.Std Result
open bytes
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
namespace BytesProofs.Numeric

theorem saturating_sub_success (a : Std.Usize) (b : Std.U64) (converted : Std.Usize)
    (hc : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from b =
      ok (.Ok converted)) :
    ∃ r, saturating_sub_usize_u64 a b = ok r ∧ r.val = a.val - converted.val := by
  refine ⟨core.num.Usize.saturating_sub a converted, ?_, ?_⟩
  · simp [saturating_sub_usize_u64, hc]
  · change (BitVec.ofNat _ (max 0 (a.val - converted.val))).toNat = _
    rw [Nat.zero_max, BitVec.toNat_ofNat, Nat.mod_eq_of_lt]
    exact Nat.lt_of_le_of_lt (Nat.sub_le _ _) a.hBounds

theorem saturating_sub_unrepresentable (a : Std.Usize) (b : Std.U64)
    (err : core.num.error.TryFromIntError)
    (hc : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from b = ok (.Err err)) :
    saturating_sub_usize_u64 a b = ok 0#usize := by
  simp [saturating_sub_usize_u64, hc]

theorem min_success (a : Std.U64) (b converted : Std.Usize)
    (hc : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from a = ok (.Ok converted)) :
    ∃ r, min_u64_usize a b = ok r ∧ r.val = min converted.val b.val := by
  refine ⟨core.cmp.impls.OrdUsize.min converted b, ?_, ?_⟩
  · simp [min_u64_usize, hc]
  · simp

theorem min_unrepresentable (a : Std.U64) (b : Std.Usize)
    (err : core.num.error.TryFromIntError)
    (hc : Usize.Insts.CoreConvertTryFromU64TryFromIntError.try_from a = ok (.Err err)) :
    min_u64_usize a b = ok b := by
  simp [min_u64_usize, hc]

theorem capacity_decode_zero : bytes_mut.original_capacity_from_repr 0#usize = ok 0#usize := by
  simp [bytes_mut.original_capacity_from_repr]

theorem sign_extend_full_width (v : Std.U64) :
    buf.buf_impl.sign_extend v 8#usize = ok (UScalar.hcast .I64 v) := by
  have hs : (8#usize - 8#usize : Result Std.Usize) = ok 0#usize := by rfl
  have hm : (0#usize * 8#usize : Result Std.Usize) = ok 0#usize := by
    change UScalar.tryMk .Usize 0 = ok 0#usize
    have hb : UScalar.check_bounds .Usize 0 := by scalar_tac
    simp only [UScalar.tryMk, UScalar.tryMkOpt, hb, _root_.dite_true, ofOption]
  unfold buf.buf_impl.sign_extend
  simp only [hs]
  simp only [bind_ok]
  simp only [hm]
  simp only [bind_ok, lift]
  change (do
    let x ← UScalar.shiftLeft v 0
    IScalar.shiftRight (UScalar.hcast .I64 x) 0) = _
  simp only [UScalar.shiftLeft, IScalar.shiftRight, UScalarTy.U64_numBits_eq,
    IScalarTy.I64_numBits_eq, Nat.zero_lt_succ, ite_true, bind_tc_ok,
    BitVec.sshiftRight_zero]
  have hz : v.bv.shiftLeft 0 = v.bv := by bv_decide
  rw [hz]

/-- The seven nonzero encodings decode to capacities 1024 through 65536. -/
theorem capacity_decode_valid (repr : Std.Usize) (h : repr.val ≤ 7) :
    bytes_mut.original_capacity_from_repr repr ⦃ r =>
      r.val = if repr.val = 0 then 0 else 2 ^ (repr.val + 9) ⦄ := by
  have hp := System.Platform.numBits_eq
  have cases : repr = 0#usize ∨ repr = 1#usize ∨ repr = 2#usize ∨
      repr = 3#usize ∨ repr = 4#usize ∨ repr = 5#usize ∨
      repr = 6#usize ∨ repr = 7#usize := by scalar_tac
  rcases cases with rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl
  all_goals
    unfold bytes_mut.original_capacity_from_repr
    simp only [bytes_mut.MIN_ORIGINAL_CAPACITY_WIDTH]
    simp
  all_goals step* <;> simp_all [Usize.size, Usize.numBits]
  all_goals rcases hp with hp | hp <;> norm_num [hp] at *


/-- Every supported byte width succeeds, with exact signed bit-shift semantics. -/
theorem sign_extend_bits (v : Std.U64) (nbytes : Std.Usize)
    (hlo : 1 ≤ nbytes.val) (hhi : nbytes.val ≤ 8) :
    buf.buf_impl.sign_extend v nbytes ⦃ r =>
      r.bv = (v.bv.shiftLeft ((8 - nbytes.val) * 8)).sshiftRight
        ((8 - nbytes.val) * 8) ⦄ := by
  unfold buf.buf_impl.sign_extend
  step*
  all_goals simp_all [UScalar.hcast_bv_eq]

/-- Sign extension preserves exactly the low `nbytes` bytes and extends their sign bit. -/
theorem sign_extend_low_bytes (v : Std.U64) (nbytes : Std.Usize)
    (hlo : 1 ≤ nbytes.val) (hhi : nbytes.val ≤ 8) :
    buf.buf_impl.sign_extend v nbytes ⦃ r =>
      r.bv = (v.bv.setWidth (8 * nbytes.val)).signExtend 64 ⦄ := by
  have bits : (v.bv.shiftLeft ((8 - nbytes.val) * 8)).sshiftRight
      ((8 - nbytes.val) * 8) = (v.bv.setWidth (8 * nbytes.val)).signExtend 64 := by
    have cases : nbytes.val = 1 ∨ nbytes.val = 2 ∨ nbytes.val = 3 ∨
        nbytes.val = 4 ∨ nbytes.val = 5 ∨ nbytes.val = 6 ∨
        nbytes.val = 7 ∨ nbytes.val = 8 := by omega
    rcases cases with hn | hn | hn | hn | hn | hn | hn | hn
    all_goals rw [hn]
    all_goals norm_num
    all_goals apply BitVec.eq_of_getLsbD_eq_iff.mpr
    all_goals intro i hi
    all_goals simp only [BitVec.getLsbD_sshiftRight, BitVec.getLsbD_signExtend,
      BitVec.msb_eq_getLsbD_last, BitVec.getLsbD_shiftLeft, BitVec.getLsbD_setWidth]
    all_goals split_ifs <;> simp_all <;> congr 1 <;> omega
  exact WP.spec_mono (sign_extend_bits v nbytes hlo hhi)
    (fun _ hr => hr.trans bits)

theorem leading_zeros_bounded {w : Nat} (v : BitVec w) :
    BitVec.leadingZeros v ≤ w := by
  unfold BitVec.leadingZeros
  split <;> omega

/-- The capacity encoding fits the three-bit representation on the selected target. -/
theorem capacity_encode_bounded (cap : Std.Usize)
    (hplatform : System.Platform.numBits = 64) :
    bytes_mut.original_capacity_to_repr cap ⦃ r => r.val ≤ 7 ⦄ := by
  unfold bytes_mut.original_capacity_to_repr
  simp only [bytes_mut.MIN_ORIGINAL_CAPACITY_WIDTH,
    bytes_mut.MAX_ORIGINAL_CAPACITY_WIDTH, bytes_mut.PTR_WIDTH]
  step*
  · rw [i2_post, i1_post, UScalar.cast_val_eq]
    have hb := leading_zeros_bounded i.bv
    have hz : (core.num.Usize.leading_zeros i).val ≤ 64 := by
      simp only [core.num.Usize.leading_zeros, UScalar.val]
      have hb' : BitVec.leadingZeros i.bv ≤ 64 := by simpa [hplatform] using hb
      exact le_trans (Nat.mod_le _ _) hb'
    have hm := Nat.mod_le (core.num.Usize.leading_zeros i).val
      (2 ^ UScalarTy.Usize.numBits)
    scalar_tac
  · simp only [core.cmp.min, WP.spec_ok]
    have hm : (core.cmp.impls.OrdUsize.min width i3).val = min width.val i3.val := by simp
    rw [hm]
    have := Nat.min_le_right width.val i3.val
    omega


theorem capacity_encode_exact (cap : Std.Usize)
    (hplatform : System.Platform.numBits = 64) :
    bytes_mut.original_capacity_to_repr cap ⦃ r => r.val = min (64 - BitVec.leadingZeros (cap.bv.ushiftRight 10)) 7 ⦄ := by
  unfold bytes_mut.original_capacity_to_repr
  simp only [bytes_mut.MIN_ORIGINAL_CAPACITY_WIDTH,
    bytes_mut.MAX_ORIGINAL_CAPACITY_WIDTH, bytes_mut.PTR_WIDTH]
  step*
  · rw [i2_post, i1_post, UScalar.cast_val_eq]
    have hb := leading_zeros_bounded i.bv
    have hz : (core.num.Usize.leading_zeros i).val ≤ 64 := by
      simp only [core.num.Usize.leading_zeros, UScalar.val]
      have hb' : BitVec.leadingZeros i.bv ≤ 64 := by simpa [hplatform] using hb
      exact le_trans (Nat.mod_le _ _) hb'
    have hm := Nat.mod_le (core.num.Usize.leading_zeros i).val
      (2 ^ UScalarTy.Usize.numBits)
    scalar_tac
  · simp only [core.cmp.min, WP.spec_ok]
    have hm : (core.cmp.impls.OrdUsize.min width i3).val = min width.val i3.val := by simp
    rw [hm]
    have hi2 : i2.val = BitVec.leadingZeros i.bv := by
      rw [i2_post, i1_post, UScalar.cast_val_eq]
      have hb : BitVec.leadingZeros i.bv ≤ 64 := by
        simpa [hplatform] using leading_zeros_bounded i.bv
      have hb32 : BitVec.leadingZeros i.bv < 2 ^ 32 := by omega
      simp only [core.num.Usize.leading_zeros, UScalar.val, BitVec.toNat_ofNat,
        UScalarTy.Usize_numBits_eq, hplatform]
      change (BitVec.ofNat 32 (BitVec.leadingZeros i.bv)).toNat % 2 ^ 64 = _
      rw [BitVec.toNat_ofNat, Nat.mod_eq_of_lt hb32]
      rw [Nat.mod_eq_of_lt (by omega)]
    have hi : i.bv = cap.bv.ushiftRight 10 := by simpa using i_post1
    rw [width_post, hi2, hi, i3_post]


end BytesProofs.Numeric
#print axioms BytesProofs.Numeric.saturating_sub_success
#print axioms BytesProofs.Numeric.saturating_sub_unrepresentable
#print axioms BytesProofs.Numeric.min_success
#print axioms BytesProofs.Numeric.min_unrepresentable
#print axioms BytesProofs.Numeric.capacity_decode_zero
#print axioms BytesProofs.Numeric.sign_extend_full_width

#print axioms BytesProofs.Numeric.capacity_decode_valid

#print axioms BytesProofs.Numeric.sign_extend_bits

#print axioms BytesProofs.Numeric.capacity_encode_bounded

#print axioms BytesProofs.Numeric.sign_extend_low_bytes

#print axioms BytesProofs.Numeric.capacity_encode_exact

#print axioms BytesProofs.Numeric.capacity_decode_valid
#print axioms BytesProofs.Numeric.capacity_decode_zero
#print axioms BytesProofs.Numeric.capacity_encode_bounded
#print axioms BytesProofs.Numeric.capacity_encode_exact
#print axioms BytesProofs.Numeric.min_success
#print axioms BytesProofs.Numeric.min_unrepresentable
#print axioms BytesProofs.Numeric.saturating_sub_success
#print axioms BytesProofs.Numeric.saturating_sub_unrepresentable
#print axioms BytesProofs.Numeric.sign_extend_bits
#print axioms BytesProofs.Numeric.sign_extend_full_width
#print axioms BytesProofs.Numeric.sign_extend_low_bytes
