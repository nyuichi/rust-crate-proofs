import Lake
open Lake DSL
require aeneas from git "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package usizegetterproofs
lean_lib BitField
lean_lib BitfieldRangeUsizeInline
lean_lib LayoutAudit
lean_lib FailureProofs
lean_lib GetterNormal
lean_lib CompleteSpec
@[default_target] lean_lib UsizeGetterAudit
