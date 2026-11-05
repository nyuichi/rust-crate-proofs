import Generated.NormalizeLineEndings
open Aeneas Aeneas.Std
namespace OriginalConstructor
 theorem initializes {I : Type} (inst : core.iter.traits.iterator.Iterator I Char) (iter : I) :
 normalize_line_endings.normalized inst iter = .ok ⟨iter, false⟩ := by rfl
#print axioms initializes
end OriginalConstructor
