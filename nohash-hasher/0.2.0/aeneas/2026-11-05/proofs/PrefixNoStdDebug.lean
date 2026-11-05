import NohashNostdDebug
open Aeneas Aeneas.Std Result
namespace PrefixNoStdDebug
open nohash_nostd_debug
theorem exact_transition {T : Type} (inst : nohash_hasher.IsEnabled T) (old : U64) (written : Bool) (pd : core.marker.PhantomData T) (length : Usize) :
 nohash_nostd_debug.prefix inst (old,written,pd) length = (if written then .fail .assertionFailure else .ok (UScalar.cast .U64 length,true,pd)) := by
 cases written <;> simp [nohash_nostd_debug.prefix, massert, lift]
theorem wide_unsigned_panic {T : Type} (inst : nohash_hasher.IsEnabled T) (state : nohash_hasher.NoHashHasher T) (value : U128) :
 wide_unsigned inst state value = .fail .panic := by simp [wide_unsigned]
theorem wide_signed_panic {T : Type} (inst : nohash_hasher.IsEnabled T) (state : nohash_hasher.NoHashHasher T) (value : I128) :
 wide_signed inst state value = .fail .panic := by simp [wide_signed]
#print axioms exact_transition
#print axioms wide_unsigned_panic
#print axioms wide_signed_panic
end PrefixNoStdDebug
