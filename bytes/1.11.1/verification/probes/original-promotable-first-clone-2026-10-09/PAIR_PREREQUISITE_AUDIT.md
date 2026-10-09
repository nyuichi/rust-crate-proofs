# Pair initialization prerequisite audit

Date: 2026-10-09. Read-only archive audit; no prover or native command was run.

## Capture integrity and targets

Audited `evidence/pair-positive-diagnostic-v3.tar.gz`, SHA-256
`09387c3550e92ad212f13d5b8277d64fe4f4d1cc76b5fff54befd75b72e2f3e5`. All
1,846 member paths are unique and match the receipt's path and SHA-256 list.
The archive contains 666 `probe/` members, 1,063 captured repository files,
110 private-Std files, six tool/configuration files, and `run.log`.

The archived `probe/verif/**/*.coma` set, receipt target list, and
`probe/generated/proof-targets.json` `included` list are identical: 76 targets,
no excluded entries, no feature overrides, and no terminal feature. I parsed
each target proof tree: 76 files, 450 prover leaves, zero null leaves, and zero
structural leaves. The log ends with `Proved (76 files)`.

This is recorded as a **diagnostic prerequisite**, not a clean correspondence
gate: `target_policy.diagnostic` is true and
`correspondence_exit_status` is 2. The archived correspondence file says
`not_run` for a deliberate semantic development run. These are translation and
proof results only; this audit makes no full-gate-checker or admission claim.

## Tool and standard-library inputs

All 110 archived private-Std members exactly match both the installed
`creusot-std 0.13.0` tree and its 110-entry `bytes-proof-std-source.SHA256SUMS`
manifest. The captured source archive fingerprint is
`17ca7c53dcfac9b67abef67588f54286b153c1bc7e2d351a8722b9be48d22e8d`.
The captured Cargo patch directs `creusot-std` to that private source.

The log records `sc-drf disabled`, a 1024 MiB limit, and one prover worker.
The captured Why3 configurations pin magic 14, `running_provers_max = 1`,
1024 MiB, and a five-second limit. Four prover backends are configured, so
“one” means at most one running at a time; it does not mean only one backend is
available. The proof script checks the locked Cargo feature tree and aborts if
the private `creusot-std` `sc-drf` feature appears, and invokes Why3find with
`-j 1`. The archived installation manifest pins Rust/Creusot/Why3 versions
and tool binary hashes; the binary payloads themselves are not included in this
archive.

## Source and imports

The archived `probe/src/lifecycle.rs` SHA-256 is
`d848cca99078d2960d0554d12515f706fc554c1199843c84381739056c3e222b`, equal to
the analyzed current file. The extracted `initialize_pair` item is byte-for-byte
identical. It requires the atomic permission to contain exactly the count-2
singleton at the current timestamp, then consumes the full lifetime token in
two splits and inserts separate ticket fragments at IDs 0 and 1. The method
itself performs no native atomic RMW; it assumes the count-2 permission as an
input, so it does not establish which native operation produced that state.

I reconstructed the captured repository tree under a temporary root, restored
the archived probe at its relative sibling location, and checked all nine
`#[path]` module imports and the direct production `include!` path: every target
exists in captured inputs. The `OUT_DIR` records are also archived; the captured
`build.rs` and `extract_public.py` source the production `bytes.rs`, record
files, and helpers present in the captured repository tree. No uncaptured live
Rust source was needed to resolve those imports. The proof launcher and captured
tool configuration retain `/workspace/...` installation paths, so rerunning
the prover requires restoring the pinned tool installation or adapting those
environment paths; the source import tree itself is reconstructible from the
archive.

## Scope

This supports the first-promotion pair/CAS development prerequisite only. The
count-2 premise, native clone connection, later lifecycle/client correspondence,
and full-crate `From` refinement are outside this capture. It does not establish
full-gate admission, unwind behavior, other representations, or arbitrary
concurrent closures.
