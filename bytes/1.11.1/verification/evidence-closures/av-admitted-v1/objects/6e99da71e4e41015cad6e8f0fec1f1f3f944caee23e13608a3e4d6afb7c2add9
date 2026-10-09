# AV full diagnostic archive audit

Audited `av-full-diagnostic-v1.tar.gz` read-only. This is an input/proof-result integrity audit of a diagnostic capture, not an admission decision.

## Archive and proof targets

- Archive SHA-256: `a28ac41b5d7d447f0324b37ade94c9cbbd22ff3f3d7aa95effeee5764e82cb47` (matches the manifest).
- All 2,128 manifest members are present once in the tar and match their SHA-256 entries.
- The manifest has 167 unique COMA/proof pairs. Every pair hash matches, and the pair list equals the included and translated target lists; there are no excluded targets or features.
- Recounted proof JSON: 1,683 prover leaves, 0 null leaves, and 0 zero-leaf files. Distribution: 1,586 Alt-Ergo, 92 Z3, 5 CVC5.
- The completion log ends in `Proved (167 files)`. The archive records no numeric prover process exit status. The correspondence exit is 2 and `generated/correspondence.json` explicitly says `not_run`, so this archive does not establish source/proof correspondence or admission.

## Captured inputs

- All 63 reviewed production paths (61 source files plus `Cargo.toml` and `Cargo.lock`) match the manifest pins both in the archive’s repository snapshot and in the current checkout.
- The AV proof source has 24 files. `probe/src/promotion.rs` (SHA `f3627094e2121b4b52d5d2c881adc6cd23d1f80ac8c33cdf0e21a0ed56ecc1e9`) matches AU `generated/positive.rs` byte-for-byte. AU's archived repository `src/promotion.rs` is the earlier AT source (SHA `b5790f53182c689186442dc54428693a0c5144bb46febdb1a0824e8ccffee945`); it is not the correct AV prefix comparator. The AV suffix helper/caller are appended in its generated positive/active source. `probe/src/lib.rs` preserves the AU file as a prefix and adds the module routes; `suffix_extension.rs` and `suffix_pointer.rs` are the two added source modules.
- All 110 captured private Std files match the current `/workspace/bytes-proof-tools/bytes-proof-std` tree.
- The seven archived tool/config inputs are covered by the member hashes. Proof budget metadata and config contents specify one concurrent prover, 1,024 MiB, and `sc-drf` disabled; the captured launcher also passes `why3find -j 1` and checks the feature tree. All eight executable hashes match the captured installation manifest on this machine. Four `matches_prior_manifest=false` values describe a historical comparison only; they are not current hash mismatches.

## Native and compiled capture

- The archived native capture pins stage `2-2-004.ElaborateDrops.after.mir`, 30 MIR bodies (29 production bodies plus `promotable_suffix_scope`), and the native source, harness, lockfile, logs, and compiler versions. All selected MIR hashes match the archive; the 29 production MIR bodies are byte-identical to the captured AT baseline.
- The native test log records 292 passing suffix cases. The archived native correspondence fixture receipt records 25/25 in-memory controls rejected as expected, with no source mutation, Cargo/Rust build, or solver invocation during that control run.
- The four actual Cargo capture files (generated `public_records.rs`, run-build fingerprint, build output, and root output) match the archived receipt and their current location-bound files under `/workspace/bytes-proof-tools/targets/bytes`. This corroborates the captured build inputs on this host; it is not a portable rebuild.

## Scope

The proved targets cover the selected promotable-suffix trace with root advance, first Clone, root drop, surviving-child read, and final child drop, plus the retained AU prefix. The archive itself marks correspondence as skipped. It does not admit general callers, raw suffix drop without promotion, arbitrary representations, concurrent/unbounded owners, unwind behavior, or the full crate.
