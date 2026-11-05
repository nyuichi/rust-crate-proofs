import GetterNormal
open Aeneas Aeneas.Std Result
open bitfield_range_usize_inline UsizeRangeFailure
namespace UsizeRangeComplete

abbrev Valid (start finish : Usize) : Prop :=
 start<64#usize ∧ finish<=64#usize ∧ start<=finish

def GetterSpecification (value : Usize) (start finish : Usize) : Prop :=
 if Valid start finish then
  WP.spec (original_usize_get_range value start finish)
   (fun out => out.val=(value.val/2^start.val)%2^(finish.val-start.val))
 else original_usize_get_range value start finish=.fail .assertionFailure


theorem invalid_get (value : Usize) (start finish : Usize) (bad : ¬Valid start finish) :
 original_usize_get_range value start finish=.fail .assertionFailure := by
 by_cases hs : start<64#usize
 · by_cases he : finish<=64#usize
   · exact get_reversed value start finish hs he (by intro ho;exact bad ⟨hs,he,ho⟩)
   · exact get_invalid_end value start finish hs he
 · exact get_invalid_start value start finish hs


theorem get_all_inputs (hPlatform : System.Platform.numBits=64) (value : Usize) (start finish : Usize) : GetterSpecification value start finish := by
 by_cases valid : Valid start finish
 · simp only [GetterSpecification,if_pos valid]
   exact UsizeRangeGetter.get_valid_value hPlatform value start finish valid.1 valid.2.1 valid.2.2
 · simp only [GetterSpecification,if_neg valid]
   exact invalid_get value start finish valid


#print axioms invalid_get
#print axioms get_all_inputs
end UsizeRangeComplete
