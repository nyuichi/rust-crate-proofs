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

lean_lib ReadVariableProofs
lean_lib ReadVariableGenerated

lean_lib ByteorderLiteU24Roots
lean_lib U24ReadSupport
lean_lib U24WrapperProofs
lean_lib U24Decode24
lean_lib U24Exact24
lean_lib U24WriteSupport
lean_lib U24Write24

lean_lib SignedVariableRoots
lean_lib SignedVariableWriteSupport
lean_lib SignedVariableProofs
lean_lib SignedVariableNormal
lean_lib SignedVariableFailure
lean_lib SignedVariableI24
lean_lib SignedVariableContracts

lean_lib SignedReadRoots
lean_lib SignedReadSupport
lean_lib SignedDecodeMath
lean_lib SignedReadProofs

lean_lib ByteorderLite
lean_lib NativeIdentityProofs
