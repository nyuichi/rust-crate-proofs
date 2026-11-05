import NohashPrefixInline
open Aeneas Aeneas.Std Result
namespace PrefixProofs
open nohash_prefix_inline
/-- Compiler-inlined original public call, all represented states and generic enabled T. -/
theorem exact_transition {T : Type} (inst : nohash_hasher.IsEnabled T)
 (old : U64) (written : Bool) (pd : core.marker.PhantomData T) (length : Usize) :
 nohash_prefix_inline.prefix inst (old,written,pd) length =
 if written then .fail .assertionFailure else .ok (UScalar.cast .U64 length,true,pd) := by
 cases written <;> simp [nohash_prefix_inline.prefix, massert, lift]
/-- Every already-written debug state fails, with no input-dependent exception. -/
theorem already_written_panics {T : Type} (inst : nohash_hasher.IsEnabled T)
 (old : U64) (pd : core.marker.PhantomData T) (length : Usize) :
 nohash_prefix_inline.prefix inst (old,true,pd) length = .fail .assertionFailure := by simp [nohash_prefix_inline.prefix, massert, lift]
#print axioms exact_transition
#print axioms already_written_panics
end PrefixProofs
