import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package asciiround6
@[default_target] lean_lib Proofs
lean_lib DigitProofs
lean_lib Generated

lean_lib StringProofs

lean_lib SliceProofs

lean_lib ErrorProjectionProofs

lean_lib VectorIdentityProofs

lean_lib BorrowViewProofs
lean_lib StringBorrowViewProofs

lean_lib SelfBorrowProofs
lean_lib ErrorDescriptionProofs
