import ByteSignedReadProofs
open Aeneas Aeneas.Std Result Std.WP
open byteorder_read_pure_roots.byteorder
namespace ByteRead48Proofs
theorem be_read_u48_spec (buf : Slice U8) (enough : 6 ≤ buf.length) :
 BigEndian.Insts.ByteorderByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄ := by
 unfold BigEndian.Insts.ByteorderByteOrder.read_u48
 have h6 : (6#usize).val = 6 := by decide
 have h := ByteReadSupport.be_read_uint_spec buf (6#usize) (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have hp : (List.replicate 2 (0#u8) ++ buf.val.take (6#usize).val).map U8.bv = List.replicate 2 (0 : BitVec 8) ++ (buf.val.take (6#usize).val).map U8.bv := by simp
 have hd := congrArg (fun xs : List (BitVec 8) => (BitVec.fromBEBytes xs).toNat) hp
 have hx := hr.trans (hd.trans (ByteReadDecodeMath.zero_padding_be_val _ _))
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 have hex := hx.trans (congrArg (fun xs : List U8 => (BitVec.fromBEBytes (xs.map U8.bv)).toNat) he)
 refine ⟨hex, ?_⟩
 rw [hex]
 have hb := (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).isLt
 have hl : ((buf.val.take 6).map U8.bv).length = 6 := by simp [List.length_take, Nat.min_eq_left enough]
 simpa only [hl] using hb
theorem be_read_u48_contract (buf : Slice U8) :
 ((6 ≤ buf.length) → BigEndian.Insts.ByteorderByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄) ∧
 (buf.length < 6 → BigEndian.Insts.ByteorderByteOrder.read_u48 buf = .fail .assertionFailure) := by
 refine ⟨be_read_u48_spec buf, ?_⟩
 intro short
 unfold BigEndian.Insts.ByteorderByteOrder.read_u48
 apply ByteReadSupport.be_read_uint_invalid
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem be_read_i48_spec (buf : Slice U8) (enough : 6 ≤ buf.length) :
 BigEndian.Insts.ByteorderByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄ := by
 unfold BigEndian.Insts.ByteorderByteOrder.read_i48
 have h6 : (6#usize).val = 6 := by decide
 have h := ByteSignedReadProofs.be_read_int_spec buf (6#usize) (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 exact hr.trans (congrArg (fun xs : List U8 => (BitVec.fromBEBytes (xs.map U8.bv)).toInt) he)
theorem be_read_i48_contract (buf : Slice U8) :
 ((6 ≤ buf.length) → BigEndian.Insts.ByteorderByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromBEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → BigEndian.Insts.ByteorderByteOrder.read_i48 buf = .fail .assertionFailure) := by
 refine ⟨be_read_i48_spec buf, ?_⟩
 intro short
 unfold BigEndian.Insts.ByteorderByteOrder.read_i48
 apply ByteSignedReadProofs.be_read_int_invalid
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem le_read_u48_spec (buf : Slice U8) (enough : 6 ≤ buf.length) :
 LittleEndian.Insts.ByteorderByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄ := by
 unfold LittleEndian.Insts.ByteorderByteOrder.read_u48
 have h6 : (6#usize).val = 6 := by decide
 have h := ByteReadSupport.le_read_uint_spec buf (6#usize) (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have hp : (buf.val.take (6#usize).val ++ List.replicate 2 (0#u8)).map U8.bv = (buf.val.take (6#usize).val).map U8.bv ++ List.replicate 2 (0 : BitVec 8) := by simp
 have hd := congrArg (fun xs : List (BitVec 8) => (BitVec.fromLEBytes xs).toNat) hp
 have hx := hr.trans (hd.trans (ByteReadDecodeMath.zero_padding_le_val _ _))
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 have hex := hx.trans (congrArg (fun xs : List U8 => (BitVec.fromLEBytes (xs.map U8.bv)).toNat) he)
 refine ⟨hex, ?_⟩
 rw [hex]
 have hb := (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).isLt
 have hl : ((buf.val.take 6).map U8.bv).length = 6 := by simp [List.length_take, Nat.min_eq_left enough]
 simpa only [hl] using hb
theorem le_read_u48_contract (buf : Slice U8) :
 ((6 ≤ buf.length) → LittleEndian.Insts.ByteorderByteOrder.read_u48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toNat ∧ r.val < 2^48 ⦄) ∧
 (buf.length < 6 → LittleEndian.Insts.ByteorderByteOrder.read_u48 buf = .fail .assertionFailure) := by
 refine ⟨le_read_u48_spec buf, ?_⟩
 intro short
 unfold LittleEndian.Insts.ByteorderByteOrder.read_u48
 apply ByteReadSupport.le_read_uint_invalid
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
theorem le_read_i48_spec (buf : Slice U8) (enough : 6 ≤ buf.length) :
 LittleEndian.Insts.ByteorderByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄ := by
 unfold LittleEndian.Insts.ByteorderByteOrder.read_i48
 have h6 : (6#usize).val = 6 := by decide
 have h := ByteSignedReadProofs.le_read_int_spec buf (6#usize) (by decide) (by decide) enough
 apply spec_mono h
 intro r hr
 have he : buf.val.take (6#usize).val = buf.val.take 6 := by rw [h6]
 exact hr.trans (congrArg (fun xs : List U8 => (BitVec.fromLEBytes (xs.map U8.bv)).toInt) he)
theorem le_read_i48_contract (buf : Slice U8) :
 ((6 ≤ buf.length) → LittleEndian.Insts.ByteorderByteOrder.read_i48 buf ⦃ r => r.val = (BitVec.fromLEBytes ((buf.val.take 6).map U8.bv)).toInt ⦄) ∧
 (buf.length < 6 → LittleEndian.Insts.ByteorderByteOrder.read_i48 buf = .fail .assertionFailure) := by
 refine ⟨le_read_i48_spec buf, ?_⟩
 intro short
 unfold LittleEndian.Insts.ByteorderByteOrder.read_i48
 apply ByteSignedReadProofs.le_read_int_invalid
 have h6 : (6#usize).val = 6 := by decide
 simp only [h6]
 omega
end ByteRead48Proofs
#print axioms ByteRead48Proofs.be_read_u48_contract
#print axioms ByteRead48Proofs.be_read_i48_contract
#print axioms ByteRead48Proofs.le_read_u48_contract
#print axioms ByteRead48Proofs.le_read_i48_contract
