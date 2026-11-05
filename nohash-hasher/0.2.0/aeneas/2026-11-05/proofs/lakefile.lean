import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package nohashproofs
@[default_target] lean_lib Proofs
@[default_target] lean_lib SupplementalProofs
lean_lib Generated
lean_lib ReleaseGenerated
lean_lib NoStdGenerated
lean_lib NoStdReleaseGenerated
lean_lib StdProofs
lean_lib NoStdProofs
lean_lib DebugProofs
lean_lib ReleaseProofs
lean_lib NoStdDebugProofs
lean_lib NoStdReleaseProofs
