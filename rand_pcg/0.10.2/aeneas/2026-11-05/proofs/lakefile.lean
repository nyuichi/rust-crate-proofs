import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package proofs
@[default_target] lean_lib Proofs
lean_lib RandPcg

@[default_target] lean_lib AdvanceProofs
lean_lib PcgAdvance where
 roots := #[`PcgAdvance.Types, `PcgAdvance.Funs]

@[default_target] lean_lib AffineProofs
lean_lib Affine
lean_lib PcgAffineAdvance where
 roots := #[`PcgAffineAdvance.Types, `PcgAffineAdvance.Funs]
