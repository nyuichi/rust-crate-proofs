import Lake
open Lake DSL
require aeneas from git "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package endianproofs
@[default_target] lean_lib Proofs
lean_lib Generated

lean_lib SignedProofs
lean_lib ByteorderLiteApiRoots

lean_lib VariableProofs
lean_lib VariableGenerated

lean_lib FloatProofs
lean_lib ByteorderLiteFloatRoots
