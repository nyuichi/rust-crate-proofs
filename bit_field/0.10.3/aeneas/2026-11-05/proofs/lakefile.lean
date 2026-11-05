import Lake
open Lake DSL
require aeneas from git "https://github.com/AeneasVerif/aeneas" @ "be86ab0f1989d04944292b345b24c435be3ad10a" / "backends/lean"
package endianproofs

-- Always execute all six isolated audits; no build-file cache skips this job.
target rangeSupplementAudits pkg : System.FilePath := do
  Job.async do
    let runnerPath := pkg.dir / ".." / "tools" / "run-range-workspaces.py"
    proc { cmd := "python3", args := #[runnerPath.toString], cwd := some pkg.dir }
    return runnerPath

@[default_target] lean_lib Proofs where
  needs := #[.packageTarget .anonymous `rangeSupplementAudits]
lean_lib BitField

lean_lib BitArrayGenerated
lean_lib ArrayProofs where
  needs := #[.packageTarget .anonymous `rangeSupplementAudits]

lean_lib ArrayBitsProofs where
  needs := #[.packageTarget .anonymous `rangeSupplementAudits]
