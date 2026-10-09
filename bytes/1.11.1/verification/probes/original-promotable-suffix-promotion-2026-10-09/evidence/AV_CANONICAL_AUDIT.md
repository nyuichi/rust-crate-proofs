# AV canonical archive independent audit

**Result: PASS for the captured, narrow AV admission.** The evidence does not admit the complete `bytes` crate.

## Canonical archive and reused proof

- Archive: `av-positive-reuse-canonical-v1.tar.gz`
- SHA-256: `a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489`
- Receipt SHA-256: `a9e2852bfec95740eb2e7a2751e185a198267cc89f4ff829bf4724aba5c40f64`
- Members: 2164 regular files; 430,384,941 uncompressed bytes. All member paths were safe and unique, and every member hash matched the receipt.
- Proof inventory: **167 targets, 1683 prover leaves, 0 null leaves, 0 structural leaves**. The selected `.coma` and `proof.json` set is complete and has no exclusions/features.
- This is `admitted_reuse`: it reuses the completed full proof inventory and adds fresh source/native/Cargo correspondence. It does not claim a second full proof run.

The proof origin archive is `a28ac41b5d7d447f0324b37ade94c9cbbd22ff3f3d7aa95effeee5764e82cb47` (2128 members). All 334 target COMA/proof files are byte-identical between origin and AV canonical. Its log contains `Proved (167 files)`, but the numeric prover exit is not recorded; origin correspondence exited 2 and was diagnostic-only. The canonical correspondence then exited 0.

## Independent checks

- Verified all 550 unique `{path, sha256}` identity rows against both captured origin and final archive: 334 target files, 24 probe Rust files, 64 production/manifests, 110 private Std files, 7 tool/config snapshots, and 11 additional critical probe files.
- Production review manifest pins 63 paths at base commit `361c7cd261507ac0a705b3b836f73240070891c6` (61 Rust files plus `Cargo.toml` and `Cargo.lock`). `Cargo.toml.orig` is an additional captured production input.
- Reconstructed the captured ancestry only from archive contents: AU (`0924cfd015be3e7c7284c94b88dc4921ccdbf6e0c9821172774568ebe86ef701`, 157/1447/0/0), AT (`958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d`, 155/1417/0/0), AS, AR, AQ, and AP. Each nested canonical receipt/member table and proof inventory was checked. The AT target pair plus `run.log` (311 members total) is recovered from the nested AT archive; it is not duplicated in AU’s repository snapshot. Captured README/TCB differences were preserved as documentation and did not replace archived source inputs.
- Independently replayed `check_correspondence.py --audit-compiled-capture-only` from a scratch tree hydrated from the outer archive and nested canonical archives. It passed (exit 0; report SHA-256 `9c440585605b80025cb49aebca9f21f763d0ad8d6e9546502778cb46c5264d27`). This path invokes neither Cargo nor a proof tool.
- The replay confirmed the native inventory of 30 MIR bodies (29 production plus the client); the client normal drop order is `root`, then `child`. The archived native test log records 292 suffix cases. The 25-case native mutation receipt records 25 expected rejections and no acceptance/errors; I inspected that receipt but did not rerun its suite.
- Verified the single structural mapping control `refreshed_suffix_capacity_recovery_claim` rejected as expected. Its receipt is hash-bound in the archive.
- Verified four captured Cargo outputs against their receipts and hashes: generated `public_records.rs`, fingerprint, build output, and Cargo root output. These are captured snapshots; I did not rebuild them or consult the live target directory.
- Proof budget: 1024 MiB, at most one running prover, `why3find -j 1`, `sc_drf` disabled. Seven config/tool snapshot files and the installation manifest are in the archive. The eight installed executables were hashed separately at their recorded paths and all matched this archive’s installation manifest. Executable binaries are external rather than embedded; older-manifest mismatches are historical comparisons, not current-run drift.

## Reconstructing the archive-only checker replay

1. Extract the outer archive to a scratch directory, retaining `probe/`, `admission/`, and `inputs/`.
2. Copy `inputs/repository/bytes/1.11.1/` to a scratch package root. Place `probe/` under `verification/probes/original-promotable-suffix-promotion-2026-10-09/`.
3. Verify/extract each nested canonical archive from its captured sibling evidence directory: AU, AT, AS, AR, AQ, and AP. Overlay the archived canonical probe tree onto each sibling while preserving the separately captured `README.md`, `TCB.md`, and `evidence/` files.
4. From the extracted AU archive, restore `inputs/au-proof-origin/au-full-diagnostic-v1.{log,json,tar.gz}` into the AU sibling’s `evidence/` directory. The AV proof origin is directly under outer `inputs/av-proof-origin/`.
5. From the AV probe directory, run:

   ```sh
   python3 check_correspondence.py --audit-compiled-capture-only --output <scratch-report.json>
   ```

This is a source/native correspondence replay over the archived snapshots. It is not a fresh Cargo build or proof run.

## Scope and limits

This gate covers one selected client: an owned root advances in place, performs its first stored-vtable promotable `Clone`, then the root automatically drops before the suffix is read and the child automatically drops after the returned `Vec` is evaluated. The native test does not claim observed allocator-parity coverage. General callers, arbitrary or concurrent owners, unwind, full allocator parity, and complete crate behavior remain outside this evidence. **Full `bytes` crate behavior is not admitted.**

The captured Cargo receipt uses the absolute target directory `/workspace/bytes-proof-tools/targets/bytes`. Original `OUT_DIR` and fingerprint paths are location-bound; this audit verified the captured artifact snapshots and did not rebuild or read the live target directory. Tool binaries are external, so reproducing the original execution environment requires matching tools at the recorded paths or equivalent matching installations.
