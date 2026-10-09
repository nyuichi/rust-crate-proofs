# Independent audit: automatic Drop final1

## Archive and proof results

- Archive: `automatic-positive-final1-2026-10-09.tar.gz`
- SHA-256: `a550b2564160823fe425157ed45dba90471577d3124e6677e7d101b2bacb55c9`
- The archive contains 1,717 unique regular-file members. I checked each path for traversal, compared its bytes with the external manifest, and independently recomputed every member SHA-256. All 1,717 matched.
- The archive contains 77 included Coma targets and a matching result JSON for each. I checked each target and result hash against the manifest. The target policy has no exclusions, no enabled features, no diagnostic mode, and correspondence exit status 0.
- I recomputed the Coma leaf totals from the archived result files: **77 files, 445 prover leaves, 0 null leaves, 0 structural leaves**. The archived `run.log` is the automatic-drop proof run (SHA-256 `d451f3e6f14cbec8c7f58cf71170b49933b47703f7b2c79bb962146cdd25b986`) and ends with `Proved (77 files)`.

## Captured inputs

The 1,717 members comprise 872 probe files, 728 repository and prior-probe files, 110 private Std files, 6 tool/configuration files, and the proof run log. Every archived member matches its mapped current workspace input except `probe/README.md`. The current README adds the post-capture validation section; its change is documentation only. The probe source, checker, controls, generated proof source and mapping, MIR, manifests, proof results, private Std, repository sources, tools/configuration, and native-run log match the archive. No proof source or prover target changed after capture.

## Reconstructed checker replay

I reconstructed the source tree from archive members under `/workspace/work/automatic-drop-final1-reconstruction` and ran the checker there without a prover. For portability, I changed only the two absolute local path values in the scratch copy of `generated/native-harness/Cargo.toml` so its library and bytes dependency resolved within that reconstructed tree; the archived manifest was hash-verified before rebasing.

- Explicit `check_correspondence.py --shadow generated/active.rs --mapping generated/mapping.json`: **`correspondence_pass`**.
- Checker SHA-256: `2947d8d5bb1531b520f4e81596df2ae1b39f4987a71a31f8d804be803903ab2c`.
- `check_checker_controls.py`: **62/62 controls rejected**, `prover_invoked: false`. The replayed control result exactly matches the archived `fixtures/checker-control-results.json`.
- Replayed client, production source chain, eight MIR hashes and derived normal edges, proof-shadow prefix/helper/caller, mapping, toolchain, lockfile, module routes, and native harness facts match the stored correspondence report. Only the expected path-dependent harness-manifest hash and absolute root strings differ in the relocated replay.

## Scope and remaining TCB

This evidence covers the exact default-std, two-owner Shared client’s normal return. MIR shows the return value saved before `second` is dropped, followed by `first`; the proof shadow preserves that order and checks per-owner removal and the final reclaim receipt. The source/MIR checker traces production `Bytes::drop` through the selected vtable callback, `AtomicMut::get_mut`, `release_shared`, final Acquire, and both frees.

The checker replay is structural evidence, not a proof of compiler or MIR correctness. Generic native terminal-place-to-consuming-helper equivalence, borrowed-reference no-drop behavior, Ghost erasure/reification, and reviewed generic field/event/cursor/free primitives remain explicit TCB. Unwind correctness, other representations, arbitrary moves/drop glue, arbitrary concurrency, and whole-crate verification remain outside this result.
