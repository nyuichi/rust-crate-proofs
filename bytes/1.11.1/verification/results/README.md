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
