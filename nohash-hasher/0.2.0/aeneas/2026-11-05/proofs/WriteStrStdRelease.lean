import NohashStringStdRelease
open Aeneas Aeneas.Std Result
namespace WriteStrStdRelease
open nohash_string_std_release
theorem always_panics {T : Type} (inst : nohash_hasher.IsEnabled T)
 (state : nohash_hasher.NoHashHasher T) (text : Str) :
 write_text inst state text = .fail .panic := by simp [write_text]
#print axioms always_panics
end WriteStrStdRelease
