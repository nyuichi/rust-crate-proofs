import Lake
open Lake DSL
require aeneas from git "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package endianproofs
@[default_target] lean_lib Proofs
lean_lib Generated

lean_lib SignedProofs
lean_lib ByteorderApiRoots

lean_lib FloatProofs
lean_lib ByteorderFloatRoots

lean_lib ReadVariableProofs
lean_lib ReadVariableGenerated

lean_lib ByteorderU24Roots
lean_lib U24Bodies
lean_lib U24WrapperProofs
lean_lib U24Decode24
lean_lib U24Exact24

lean_lib Byteorder
lean_lib NativeIdentityProofs

lean_lib ByteorderReadPureRoots
lean_lib ByteReadDecodeMath
lean_lib ByteReadSupport
lean_lib ByteSignedReadProofs
lean_lib ByteRead48Proofs
