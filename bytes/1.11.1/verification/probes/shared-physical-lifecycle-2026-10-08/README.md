# Reclaimable shared physical lifecycle — interim A40

This bounded component consumes actual B1 Vec capabilities, freezes the physical
region using stock FullBorrow/GhostShared, reads it through two separate lifetime
fractions (including after the peer retires), and returns the original region
through EndBorrow only after joining both fractions and ending the lifetime.
Native count starts at 2. Retirement uses actual Release fetch_sub; the last
retirement uses actual Acquire load and stock AtView synchronization.

`positive-conditional-40.tar.gz` preserves 40 complete proof files, zero null,
engine exit0. Native one test covers six empty/nonempty/spare-capacity allocations.
This milestone commits exact archived Rust sources; subsequent agent worktree
edits are not part of its proof claim. Referenced production sources remain
unchanged from commit6cbdca16; root-audit-conditional-40.json and the selected
Std source manifest provide supplementary provenance. The old archive itself
predates these source snapshots and does not contain every path dependency.

This is CONDITIONAL cleanup: if a final result is produced, actual B3
deallocation preconditions hold. Exact-one completion, dynamic Relaxed clone,
actual Shared.ref_cnt/control connection and automatic Drop are not proved.
Subjective Recovery/EndBorrow stay in the caller; general last-thread resource
transport is not claimed. See PLAN.md for the next obligations.

Generic native/model/event synchronization and RMW release-sequence publication
remain explicit trusted rules. Retirement/lifetime/recovery caller bodies are
not trusted. Stock lifetime and AtView contracts remain inherited TCB; caller
proofs do not prove adequacy. Objective and ghost-move frontend failures are
archived separately; these were not failed VCs.

Run only this package via run-proof.sh with elevated Why3 socket access. The
wrapper locks the shared slot, uses one prover and 1024 MiB, and rejects sc-drf.
Missing-Acquire/premature-end/duplicate controls belong to the next validated
increment; this interim snapshot does not claim them already checked.
