import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package genericconversions
@[default_target] lean_lib Proofs
lean_lib Oorandom
lean_lib Generated
lean_lib GeneratedDelegations
lean_lib OriginalDelegationProofs
