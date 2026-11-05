import SetterNormal
open Aeneas Aeneas.Std Result
open bitfield_range_u64_inline U64RangeFailure
namespace U64RangeFitFailure
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U64.rMax

theorem low_word_value (word : BitVec 64) (width : Nat) (bound : width<=64) :
 ((word <<< (64-width)) >>> (64-width)).toNat=word.toNat%2^width := by
 apply Nat.eq_of_testBit_eq
 intro i
 simp only [BitVec.testBit_toNat,Nat.testBit_mod_two_pow]
 simpa only [Nat.sub_zero,Nat.zero_add,BitVec.ushiftRight_zero] using
  U64RangeGetter.clipped_bits word 0 width i bound (by omega)

theorem unfit_low_word_ne (word : BitVec 64) (width : Nat) (bound : width<=64)
 (unfit : ¬word.toNat<2^width) :
 ((word <<< (64-width)) >>> (64-width))≠word := by
 intro h
 have hv := congrArg BitVec.toNat h
 rw [low_word_value word width bound] at hv
 have hlt := Nat.mod_lt word.toNat (Nat.two_pow_pos width)
 rw [hv] at hlt
 exact unfit hlt

theorem bind_const {α β : Type} (m : Result α) (p : α → Prop) (h : WP.spec m p)
 (k : α → Result β) (result : Result β) (hk : ∀ v, p v → k v=result) :
 Std.bind m k=result := by
 cases m using Result.cases with
 | ret v =>
  have hp : p v := by simpa only [WP.spec_ok] using h
  simpa only [bind_ok] using hk v hp
 | vis e cont => exact False.elim (by simpa only [WP.spec_vis] using h)
 | div => exact False.elim (by simpa only [WP.spec_div] using h)

theorem set_nonempty_unfit (value replacement : U64) (start finish : Usize)
 (hs : start<64#usize) (he : finish<=64#usize) (positive : start<finish)
 (unfit : ¬replacement.val<2^(finish.val-start.val)) :
 original_u64_set_range value start finish replacement=.fail .assertionFailure := by
 have ho : start<=finish := by scalar_tac
 have same : start≠finish := by scalar_tac
 have hsn : start.val<64 := by scalar_tac
 have hen : finish.val<=64 := by scalar_tac
 have hpos : start.val<finish.val := by scalar_tac
 simp only [original_u64_set_range,bit_length,bind_ok,massert,if_pos hs,if_pos he,if_pos ho,if_neg same]
 refine bind_const _ _ (Usize.sub_spec (x:=finish) (y:=start) (by scalar_tac)) _ _ ?_
 intro width hw
 have hwv : width.val=finish.val-start.val := by scalar_tac
 have hwbound : width.val<=64 := by omega
 have hwpos : 0<width.val := by omega
 have unfitwidth : ¬replacement.val<2^width.val := by rw [hwv];exact unfit
 refine bind_const _ _ (Usize.sub_spec (x:=64#usize) (y:=width) (by scalar_tac)) _ _ ?_
 intro delta hd
 have hdv : delta.val=64-width.val := by scalar_tac
 refine bind_const _ _ (U64.ShiftLeft_spec replacement delta (by scalar_tac)) _ _ ?_
 intro left hl
 refine bind_const _ _ (Usize.sub_spec (x:=64#usize) (y:=width) (by scalar_tac)) _ _ ?_
 intro delta2 hd2
 have hdv2 : delta2.val=64-width.val := by scalar_tac
 refine bind_const _ _ (U64.ShiftRight_spec left delta2 (by scalar_tac)) _ _ ?_
 intro fitted hf
 have hne : fitted≠replacement := by
  intro h
  have hb : fitted.bv=replacement.bv := congrArg (fun x : U64 => x.bv) h
  simp only [U64.bv] at hl hf hb
  rw [hf.2,hl.2,hdv,hdv2] at hb
  exact unfit_low_word_ne replacement.bv width.val hwbound
   (by simpa only [UScalar.val] using unfitwidth) hb
 simp only [massert,if_neg hne,bind_fail]

#print axioms low_word_value
#print axioms unfit_low_word_ne
#print axioms bind_const
#print axioms set_nonempty_unfit
end U64RangeFitFailure
