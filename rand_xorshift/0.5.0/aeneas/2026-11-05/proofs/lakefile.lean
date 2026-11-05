import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package randproofs
@[default_target] lean_lib Proofs where
  roots := #[`Proofs, `Nonzero]
lean_lib RandXorshift where
  roots := #[`RandXorshift.Types, `RandXorshift.FunsExternal, `RandXorshift.Funs]
