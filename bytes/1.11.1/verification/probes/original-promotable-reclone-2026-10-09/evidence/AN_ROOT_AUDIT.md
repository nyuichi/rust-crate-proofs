# AN canonical archive audit

**Result:** pass for the archived bounded AN gate; the complete bytes 1.11.1 target remains **NOT ADMITTED**.

## Archive and proof results

- Archive SHA-256: `7181ae4c07997888e1af1dfad41fdb109a489b4ca101e1dd972c9b6d4d13843e`; sidecar SHA-256: `6a8d29a8c09e8c0fdae1ccaefbf15f7f047f21feb29ca3eaf653749142f5a5fe`. Recomputed all 875 regular-member hashes against the sidecar: no missing, extra, duplicate or mismatched members.
- Independently matched the included policy, receipt rows and archived files: 120 Coma targets and 120 proof trees, with no excluded targets, no feature flags, diagnostic false, checker status 0. Recursing every Coma proof tree yielded **914 prover leaves, 0 null, 0 structural**; each target's Coma and proof hashes match its row. The archived run log ends with `Proved (120 files)` and `No dangling files found` (SHA-256 `138760858eceefe773f5ac0546c03a69a607e8d22ca9fdddaf2dfd2269f531e2`).
- Target list digest (newline-joined, policy order): `47df1dd38389dcc734eadd9ca954c2cbd7e38319dea693df9b8acadc932c2d80`. Exact target names and all per-target hashes are in the immutable archive manifest and the companion JSON audit.

## Input and correspondence checks

- Archived private Std contains 110 files (104 Rust files), package `creusot-std 0.13.0`, VCS `afd365f7a8ba33a90c67b0809de4e7a097421057`. All 110 hashes match the installed source tree.
- The reviewed production manifest pins 63 files (61 Rust source files plus package manifests) at base `361c7cd261507ac0a705b3b836f73240070891c6`. Archive bytes match both `git show` at that commit and the current worktree for all files. The correspondence checker's ten direct support imports also match checker-pinned hashes, archive bytes, current files and the same base commit.
- Native capture includes 19 MIR bodies from `2-2-004.ElaborateDrops.after.mir` (18 production bodies plus the client); rustc `rustc 1.98.0-nightly (91fe22da8 2026-06-21)`, Cargo `cargo 1.98.0-nightly (a595d0da2 2026-06-20)`. The captured native harness log reports one passing test; that is execution corroboration, not proof.
- Replayed the extracted archive's native checker and main correspondence checker: both pass. Replayed archived structural suites: main controls 49/49 rejected as expected; native source/MIR controls 52/52 rejected as expected. Controls report no solver invocation; they test checker coverage, not semantic proof.

## Cargo outputs and toolchain

Compared all four archived build receipts with both the captured bytes and already-existing Cargo outputs under `/workspace/bytes-proof-tools/targets/bytes`. All hashes match (fingerprint, build output, root output, generated `public_records.rs`). No rebuild was done. The replay used the recorded target directory because the build receipt pins an absolute `OUT_DIR`; this compiled-output check is location-sensitive.

The archive includes six tool/config files and an installation manifest for eight binaries. All eight current external binary files match the hashes recorded in the archived installation manifest, but executable payloads are not inside the archive.

## Scope boundary

The passing claim is limited to the archived closed, nonempty Box first-promotion and promoted-root re-clone trace, with second then first child normal automatic Drop, root read, saved return and root normal Drop, for both parity branches. It excludes CAS losers/concurrency, unwind completion, arbitrary clone/Drop contexts and whole-crate behavior. Compiler/MIR adequacy, pointer provenance, allocator behavior and Rust memory-model assumptions remain TCB. No full-crate admission follows from this audit.
