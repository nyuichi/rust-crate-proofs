import FitFailure
open Aeneas Aeneas.Std Result
open bitfield_range_u64_inline U64RangeFailure
namespace U64RangeComplete

abbrev Valid (start finish : Usize) : Prop :=
 start<64#usize ∧ finish<=64#usize ∧ start<=finish

def GetterSpecification (value : U64) (start finish : Usize) : Prop :=
 if Valid start finish then
  WP.spec (original_u64_get_range value start finish)
   (fun out => out.val=(value.val/2^start.val)%2^(finish.val-start.val))
 else original_u64_get_range value start finish=.fail .assertionFailure

def SetterSpecification (value replacement : U64) (start finish : Usize) : Prop :=
 if Valid start finish then
  if start=finish then
   if replacement=0#u64 then original_u64_set_range value start finish replacement=.ok (value,fun x=>x)
   else original_u64_set_range value start finish replacement=.fail .integerOverflow
  else
   if replacement.val<2^(finish.val-start.val) then
    WP.spec (original_u64_set_range value start finish replacement) (U64RangeSetter.Post value replacement start finish)
   else original_u64_set_range value start finish replacement=.fail .assertionFailure
 else original_u64_set_range value start finish replacement=.fail .assertionFailure

theorem invalid_get (value : U64) (start finish : Usize) (bad : ¬Valid start finish) :
 original_u64_get_range value start finish=.fail .assertionFailure := by
 by_cases hs : start<64#usize
 · by_cases he : finish<=64#usize
   · exact get_reversed value start finish hs he (by intro ho;exact bad ⟨hs,he,ho⟩)
   · exact get_invalid_end value start finish hs he
 · exact get_invalid_start value start finish hs

theorem invalid_set (value replacement : U64) (start finish : Usize) (bad : ¬Valid start finish) :
 original_u64_set_range value start finish replacement=.fail .assertionFailure := by
 by_cases hs : start<64#usize
 · by_cases he : finish<=64#usize
   · exact set_reversed value replacement start finish hs he (by intro ho;exact bad ⟨hs,he,ho⟩)
   · exact set_invalid_end value replacement start finish hs he
 · exact set_invalid_start value replacement start finish hs

theorem get_all_inputs (value : U64) (start finish : Usize) : GetterSpecification value start finish := by
 by_cases valid : Valid start finish
 · simp only [GetterSpecification,if_pos valid]
   exact U64RangeGetter.get_valid_value value start finish valid.1 valid.2.1 valid.2.2
 · simp only [GetterSpecification,if_neg valid]
   exact invalid_get value start finish valid

theorem set_all_inputs (value replacement : U64) (start finish : Usize) : SetterSpecification value replacement start finish := by
 by_cases valid : Valid start finish
 · have hs:=valid.1
   have he:=valid.2.1
   have ho:=valid.2.2
   by_cases same : start=finish
   · by_cases zero : replacement=0#u64
     · simp only [SetterSpecification,if_pos valid,if_pos same,if_pos zero]
       subst finish
       subst replacement
       exact set_empty_zero value start hs
     · simp only [SetterSpecification,if_pos valid,if_pos same,if_neg zero]
       subst finish
       exact U64RangeEmptyNonzero.set_empty_nonzero value replacement start hs zero
   · by_cases fit : replacement.val<2^(finish.val-start.val)
     · simp only [SetterSpecification,if_pos valid,if_neg same,if_pos fit]
       exact U64RangeSetter.set_valid value replacement start finish hs he ho fit
     · simp only [SetterSpecification,if_pos valid,if_neg same,if_neg fit]
       have positive : start<finish := by scalar_tac
       exact U64RangeFitFailure.set_nonempty_unfit value replacement start finish hs he positive fit
 · simp only [SetterSpecification,if_neg valid]
   exact invalid_set value replacement start finish valid

#print axioms invalid_get
#print axioms invalid_set
#print axioms get_all_inputs
#print axioms set_all_inputs
end U64RangeComplete
