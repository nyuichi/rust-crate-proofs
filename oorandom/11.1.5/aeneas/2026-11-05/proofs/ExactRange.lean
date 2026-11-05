import TraceAudit
open Aeneas Aeneas.Std Result
open oorandom_range_original
namespace ExactRange
theorem rand32_contract (self : Rand32) (range : core.ops.range.Range U32) (width : U32)
 (hw : width.val=range.end.val-range.start.val) :
 (¬range.start<range.end → Rand32.rand_range self range=.fail .assertionFailure) ∧
 (range.start<range.end →
  (∀ n, (∀ k, k<n → ¬ActualLoop32.accepted (PublicTrace32.threshold width) (GenericTrace.trace (ActualLoop32.next width) (PublicTrace32.initial self width) k)) →
   ActualLoop32.accepted (PublicTrace32.threshold width) (GenericTrace.trace (ActualLoop32.next width) (PublicTrace32.initial self width) n) →
   WP.spec (Rand32.rand_range self range) (fun out =>
    out.1.val=(GenericTrace.trace (ActualLoop32.next width) (PublicTrace32.initial self width) n).2.1.val/2^32+range.start.val ∧
    out.2=(GenericTrace.trace (ActualLoop32.next width) (PublicTrace32.initial self width) n).1)) ∧
  ((∀ k, ¬ActualLoop32.accepted (PublicTrace32.threshold width) (GenericTrace.trace (ActualLoop32.next width) (PublicTrace32.initial self width) k)) →
   Rand32.rand_range self range=.div)) := by
 constructor
 · exact PublicDomain.range32_invalid self range
 · intro valid
   exact ⟨fun n prior accept => PublicTrace32.public_first_accept self range width n valid hw prior accept,
    fun reject => PublicTrace32.public_all_rejected self range width valid hw reject⟩
#print axioms rand32_contract
theorem rand64_contract (self : Rand64) (range : core.ops.range.Range U64) (width : U64)
 (hw : width.val=range.end.val-range.start.val) :
 (¬range.start<range.end → Rand64.rand_range self range=.fail .assertionFailure) ∧
 (range.start<range.end →
  (∀ n, (∀ k, k<n → ¬ActualLoop64.accepted (PublicTrace64.threshold width) (GenericTrace.trace (ActualLoop64.next width) (PublicTrace64.initial self width) k)) →
   ActualLoop64.accepted (PublicTrace64.threshold width) (GenericTrace.trace (ActualLoop64.next width) (PublicTrace64.initial self width) n) →
   WP.spec (Rand64.rand_range self range) (fun out =>
    out.1.val=(GenericTrace.trace (ActualLoop64.next width) (PublicTrace64.initial self width) n).2.1.val/2^64+range.start.val ∧
    out.2=(GenericTrace.trace (ActualLoop64.next width) (PublicTrace64.initial self width) n).1)) ∧
  ((∀ k, ¬ActualLoop64.accepted (PublicTrace64.threshold width) (GenericTrace.trace (ActualLoop64.next width) (PublicTrace64.initial self width) k)) →
   Rand64.rand_range self range=.div)) := by
 constructor
 · exact PublicDomain.range64_invalid self range
 · intro valid
   exact ⟨fun n prior accept => PublicTrace64.public_first_accept self range width n valid hw prior accept,
    fun reject => PublicTrace64.public_all_rejected self range width valid hw reject⟩
#print axioms rand64_contract
end ExactRange
