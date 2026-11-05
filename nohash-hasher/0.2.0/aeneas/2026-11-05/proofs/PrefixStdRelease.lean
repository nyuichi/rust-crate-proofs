import NohashStdReleaseAll
open Aeneas Aeneas.Std Result
namespace PrefixStdRelease
open nohash_std_release_all
theorem exact_transition {T : Type} (inst : nohash_hasher.IsEnabled T) (old : U64) (pd : core.marker.PhantomData T) (length : Usize) :
 nohash_std_release_all.prefix inst (old,pd) length = .ok (UScalar.cast .U64 length,pd) := by simp [nohash_std_release_all.prefix, lift]
theorem wide_unsigned_panic {T : Type} (inst : nohash_hasher.IsEnabled T) (state : nohash_hasher.NoHashHasher T) (value : U128) :
 wide_unsigned inst state value = .fail .panic := by simp [wide_unsigned]
theorem wide_signed_panic {T : Type} (inst : nohash_hasher.IsEnabled T) (state : nohash_hasher.NoHashHasher T) (value : I128) :
 wide_signed inst state value = .fail .panic := by simp [wide_signed]
#print axioms exact_transition
#print axioms wide_unsigned_panic
#print axioms wide_signed_panic
end PrefixStdRelease
