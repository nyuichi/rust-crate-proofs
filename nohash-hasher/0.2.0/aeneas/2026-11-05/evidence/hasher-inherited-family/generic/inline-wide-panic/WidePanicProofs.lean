import NohashWidePanicInline
open Aeneas Aeneas.Std Result
namespace WidePanicProofs
open nohash_wide_panic_inline
theorem unsigned_always_panics {T : Type} (inst : nohash_hasher.IsEnabled T)
 (state : nohash_hasher.NoHashHasher T) (value : U128) :
 wide_unsigned inst state value = .fail .panic := by simp [wide_unsigned]
theorem signed_always_panics {T : Type} (inst : nohash_hasher.IsEnabled T)
 (state : nohash_hasher.NoHashHasher T) (value : I128) :
 wide_signed inst state value = .fail .panic := by simp [wide_signed]
#print axioms unsigned_always_panics
#print axioms signed_always_panics
end WidePanicProofs
