import SplitMixState.Funs
open Aeneas Aeneas.Std Result Std.WP
open SplitMixStateModel
namespace StateProofs
theorem splitmix_state_spec (s : splitmix64.SplitMix64) :
    s.state ⦃ bytes => bytes.val = s.x.bv.toLEBytes.map (@UScalar.mk UScalarTy.U8) ⦄ := by
  unfold splitmix64.SplitMix64.state
  exact core.num.U64.to_le_bytes.step_spec s.x
end StateProofs
#print axioms StateProofs.splitmix_state_spec
