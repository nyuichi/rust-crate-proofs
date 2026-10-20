import Lake
open Lake DSL

require aeneas from "../aeneas/backends/lean"

package bytesVerification where
  leanOptions := #[⟨`autoImplicit, false⟩]

@[default_target]
lean_lib Generated

lean_lib FloatTypes
lean_lib AdaptersGenerated

lean_lib Extensions where
  roots := #[]
  globs := #[.submodules `extensions]
