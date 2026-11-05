import Lake
open Lake DSL
require aeneas from git
  "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package proofs
@[default_target] lean_lib Proofs
lean_lib RandXoshiro

@[default_target] lean_lib WrapperProofs
lean_lib XoshiroApiRoots where
 roots := #[`XoshiroApiRoots.Types, `XoshiroApiRoots.Funs]

@[default_target] lean_lib SeedProofs
lean_lib Seed512 where
 roots := #[`Seed512.Types, `Seed512.Funs]

@[default_target] lean_lib RngProofs
lean_lib XoshiroRngApiRoots where
 roots := #[`XoshiroRngApiRoots.Types, `XoshiroRngApiRoots.Funs]
