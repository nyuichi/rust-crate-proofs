import Generated.HexdisplayConstructor
open Aeneas Aeneas.Std hexdisplay_constructor
namespace ConstructorProofs
theorem constructor_exact (bytes : Slice U8) : construct bytes = .ok bytes := rfl
theorem projection_exact (view : base16ct.display.HexDisplay) : project view = .ok view := rfl
theorem roundtrip_all_slices (bytes : Slice U8) : construct_project bytes = .ok bytes := rfl
#print axioms constructor_exact
#print axioms projection_exact
#print axioms roundtrip_all_slices
end ConstructorProofs
