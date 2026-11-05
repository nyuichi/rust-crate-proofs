import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package enumproofs
@[default_target] lean_lib Proofs where
  needs := #[.facet (.module `ReferenceSupplementProofs) `leanArts]
lean_lib Generated

lean_lib CallbackProofs

lean_lib IntoEitherProofs

lean_lib ResultProofs

lean_lib DefaultProofs

lean_lib BothProofs

lean_lib MapInspectProofs

lean_lib IteratorProofs

lean_lib ReferenceSupplement
@[default_target] lean_lib ReferenceSupplementProofs

@[default_target] lean_lib PrimaryCoimport

lean_lib NextBackProofs
