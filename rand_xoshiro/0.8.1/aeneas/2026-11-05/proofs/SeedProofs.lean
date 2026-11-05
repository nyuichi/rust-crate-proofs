import Seed512.Funs
open Aeneas Aeneas.Std Result Std.WP
open rand_xoshiro
namespace SeedProofs

theorem default_spec : common.Seed512.Insts.CoreDefaultDefault.default
    ⦃ s => s.val = List.replicate 64 0#u8 ⦄ := by
  simp [common.Seed512.Insts.CoreDefaultDefault.default, Array.repeat]

theorem as_ref_spec (s : common.Seed512) :
  common.Seed512.Insts.CoreConvertAsRefSliceU8.as_ref s
    ⦃ view => view.val = s.val ⦄ := by
  simp [common.Seed512.Insts.CoreConvertAsRefSliceU8.as_ref, Array.to_slice]

theorem as_mut_spec (s : common.Seed512) :
  common.Seed512.Insts.CoreConvertAsMutSliceU8.as_mut s
    ⦃ r => r.1.val = s.val ∧ ∀ view : Slice U8,
      view.val.length = 64 → (r.2 view).val = view.val ⦄ := by
  simp [common.Seed512.Insts.CoreConvertAsMutSliceU8.as_mut, Array.to_slice_mut,
    Array.to_slice, lift]
  intro view h
  exact Array.from_slice_val s view (by simpa using h)

theorem iter_spec (s : common.Seed512) : common.Seed512.iter s
    ⦃ it => it.slice.val = s.val ∧ it.i = 0 ⦄ := by
  simp [common.Seed512.iter, core.slice.Slice.iter, Array.to_slice, lift]
end SeedProofs
#print axioms SeedProofs.default_spec
#print axioms SeedProofs.as_ref_spec
#print axioms SeedProofs.as_mut_spec
#print axioms SeedProofs.iter_spec
