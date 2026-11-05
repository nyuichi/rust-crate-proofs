import FailureProofs
open Aeneas Aeneas.Std Result
open bitfield_range_u32_inline U32RangeFailure
namespace U32RangeEmptyNonzero
attribute [local implicit_reducible] UScalar.cMax IScalar.cMax IScalar.cMin UScalar.rMax IScalar.rMax IScalar.rMin Usize.rMax U32.rMax

theorem sub_self (start : Usize) : (start-start : Result Usize)=.ok 0#usize := by
 apply spec_eq_ok
 refine WP.spec_mono (Usize.sub_spec (x:=start) (y:=start) (by scalar_tac)) ?_
 intro result h
 apply UScalar.val_eq_imp
 scalar_tac

theorem eight_sub_zero : (32#usize-0#usize : Result Usize)=.ok 32#usize := by
 apply spec_eq_ok
 refine WP.spec_mono (Usize.sub_spec (x:=32#usize) (y:=0#usize) (by scalar_tac)) ?_
 intro result h
 apply UScalar.val_eq_imp
 scalar_tac

theorem shift_eight_fails (replacement : U32) :
 (replacement <<< 32#usize : Result U32)=.fail .integerOverflow := by
 change UScalar.shiftLeft replacement (32#usize).val=.fail .integerOverflow
 unfold UScalar.shiftLeft
 rw [if_neg (by scalar_tac)]

theorem set_empty_nonzero (value replacement : U32) (start : Usize)
 (hs : start<32#usize) (nonzero : replacement≠0#u32) :
 original_u32_set_range value start start replacement=.fail .integerOverflow := by
 have hsn : start.val<32 := by scalar_tac
 have hen : start.val<=32 := by omega
 have hn : replacement.val≠0 := by scalar_tac
 simp [original_u32_set_range,bit_length,massert,hsn,hen,hn,sub_self,eight_sub_zero,shift_eight_fails]

#print axioms sub_self
#print axioms eight_sub_zero
#print axioms shift_eight_fails
#print axioms set_empty_nonzero
end U32RangeEmptyNonzero
