# Retained proof results

`current.zip` directly contains the 191-function AZ proof, its exact source and
private-Std inputs, and the separately successful native/shadow correspondence
record. `files.json` maps names to deduplicated `objects/<sha256>` members inside
this one ZIP. It never imports another archive. `current.json` binds the live
inputs and relevant default-library manifest fields to that reviewed result.

Run `python3 scripts/check-current-proof.py` from the crate directory. This
checks retained proof reuse, not a fresh prover run or a general source-equivalence
proof. A changed runtime/contract input requires new proof and correspondence;
updating expected hashes alone is insufficient. The original diagnostic run and
later accepted correspondence retain their distinct recorded dispositions.

The current archive was extracted from the SHA-256-pinned origin at Git commit
`e5f127fb`; its original digest and reuse digest remain in `current.json`.
Full original bytes/API verification remains incomplete.

`components.zip` retains historical component contracts, sources, proof tasks and
results (including substantive TCB notes), deduplicated with the same flat format.
Its paths identify original files/archive members; they do not imply current
coverage or that every retained partial record is a completed proof. It is not
read by the daily check. No nested archive hydration or old audit execution is
required. Failed-experiment suites and process metadata remain only in Git history.
