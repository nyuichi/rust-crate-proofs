import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package proofs
@[default_target] lean_lib Proofs
lean_lib Oorandom

lean_lib OorandomRangeOriginal
lean_lib EstablishedRange
lean_lib NegBridge
lean_lib Checkpoint
lean_lib FullRange
lean_lib Zero64
lean_lib PublicDomain
lean_lib NatRangeArithmetic
lean_lib RangeArithmetic
lean_lib PartialLoop
lean_lib RangePartial32
lean_lib RangePartial64
lean_lib RangeComplete32
lean_lib RangeComplete64
