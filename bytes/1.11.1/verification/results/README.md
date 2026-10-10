# Saved proof results

The current direct-source evidence is in
[`direct-bytesmut-len-api`](direct-bytesmut-len-api/README.md). Its receipt binds
the live source and proof inputs to the exact COMA files, prover results,
feature tree, translation diagnostics, pinned tool binaries and generic Std
overlay identity. `./verify-all.bash --check` validates this saved evidence
without running a solver; a changed input requires a fresh run.

`current.zip` and `current.json` retain the earlier Root lifecycle result and
its captured inputs as historical provenance. They predate the current source
changes and are not used by the active runner or checker. `components.zip`
retains earlier component contracts, source, tasks and partial results; those
records do not imply current implementation coverage. The archived proof
inputs and TCB records remain unchanged.
