import FailureProofs
open Aeneas Aeneas.Std Result
open bitfield_range_u8_inline U8RangeFailure
namespace U8RangeGetter
set_option maxHeartbeats 500000
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U8.rMax

theorem clipped_bits (word : BitVec 8) (start finish i : Nat) (he : finish<=8) (ho : start<=finish) :
 (((word <<< (8-finish)) >>> (8-finish)) >>> start).getLsbD i =
 (decide (i<finish-start) && word.getLsbD (start+i)) := by
 have hc : (8-finish+(start+i)<8) ↔ i<finish-start := by omega
 have hn : ¬8-finish+(start+i)<8-finish := by omega
 have hd : 8-finish+(start+i)-(8-finish)=start+i := by omega
 simp only [BitVec.getLsbD_ushiftRight,BitVec.getLsbD_shiftLeft]
 simp [hc,hn,hd]

theorem get_valid_bits (value : U8) (start finish : Usize)
 (hs : start<8#usize) (he : finish<=8#usize) (ho : start<=finish) :
 original_u8_get_range value start finish ⦃ out => ∀ i,
  out.bv.getLsbD i = (decide (i<finish.val-start.val) && value.bv.getLsbD (start.val+i)) ⦄ := by
 by_cases same : start=finish
 · subst finish
   rw [get_empty value start hs]
   simp only [WP.spec_ok]
   intro i
   simp
 · have hsn : start.val<8 := by scalar_tac
   have hen : finish.val<=8 := by scalar_tac
   have hon : start.val<=finish.val := by scalar_tac
   have positive : start.val<finish.val := by scalar_tac
   simp only [original_u8_get_range,bit_length,bind_ok,massert,if_pos hs,if_pos he,if_pos ho,if_neg same]
   refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=finish) (by scalar_tac)) ?_
   intro delta hd
   have hdv : delta.val=8-finish.val := by scalar_tac
   refine WP.spec_bind (U8.ShiftLeft_spec value delta (by scalar_tac)) ?_
   intro left hl
   refine WP.spec_bind (Usize.sub_spec (x:=8#usize) (y:=finish) (by scalar_tac)) ?_
   intro delta2 hd2
   have hdv2 : delta2.val=8-finish.val := by scalar_tac
   refine WP.spec_bind (U8.ShiftRight_spec left delta2 (by scalar_tac)) ?_
   intro clipped hc
   refine WP.spec_mono (U8.ShiftRight_spec clipped start (by scalar_tac)) ?_
   intro out hout i
   simp only [U8.bv] at hl hc hout ⊢
   rw [hout.2,hc.2,hl.2,hdv,hdv2]
   exact clipped_bits value.bv start.val finish.val i hen hon

theorem value_from_bits (out value : U8) (start finish : Nat)
 (bits : ∀ i, out.bv.getLsbD i=(decide (i<finish-start) && value.bv.getLsbD (start+i))) :
 out.val=(value.val/2^start)%2^(finish-start) := by
 apply Nat.eq_of_testBit_eq
 intro i
 simpa only [UScalar.val,U8.bv,←Nat.shiftRight_eq_div_pow,Nat.testBit_mod_two_pow,Nat.testBit_shiftRight,BitVec.testBit_toNat] using bits i

theorem get_valid_value (value : U8) (start finish : Usize)
 (hs : start<8#usize) (he : finish<=8#usize) (ho : start<=finish) :
 original_u8_get_range value start finish ⦃ out => out.val=(value.val/2^start.val)%2^(finish.val-start.val) ⦄ := by
 refine WP.spec_mono (get_valid_bits value start finish hs he ho) ?_
 intro out hout
 exact value_from_bits out value start.val finish.val hout

#print axioms value_from_bits
#print axioms get_valid_value
#print axioms clipped_bits
#print axioms get_valid_bits
end U8RangeGetter
