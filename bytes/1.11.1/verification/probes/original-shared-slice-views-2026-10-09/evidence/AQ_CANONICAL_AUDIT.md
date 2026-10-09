# AQ canonical archive audit

**Verdict:** GO for archive integrity, proof accounting, and the selected source/MIR correspondence. The complete original `bytes` API remains **NOT ADMITTED**.

## Archive and proof accounting

I independently hashed `aq-positive-canonical-v1.tar.gz`: it matches the receipt SHA-256 `41e8e9c164a1110c7f611bb1726f490e111c6b10a77af6b57bfc707ce49ceba5`. All 1,178 regular archive members match the external receipt, with no duplicate names. The receipt itself hashes to `69723dab5c79d46d6b69c53d23b4a08a934f4920121964fff0280a66d5d50969`.

All 139 captured `.coma` targets and 139 `proof.json` files match their receipt hashes, and the archive's `.coma` set is exactly the selected target set. Recounting every Coma proof tree gives **139 files, 1,202 prover leaves, 0 null leaves, and 0 structural leaves**. The archived target policy agrees with the receipt: 139 included, none excluded, no feature/source-control/terminal-control flags, `diagnostic=false`, and correspondence exit status 0. The archived run log ends `Proved (139 files)`.

## Inputs and provenance

The reviewed production manifest has 63 entries (the pinned base commit is `361c7cd261507ac0a705b3b836f73240070891c6`) and all 63 hashes match both the archived snapshot and current `bytes/1.11.1` worktree. All 110 captured private Creusot Std files match the current tree; its source archive SHA-256 is `17ca7c53dcfac9b67abef67588f54286b153c1bc7e2d351a8722b9be48d22e8d`, matching the installed manifest. All eight tool binary hashes and all six captured tool/configuration files match their current installed copies. The captured rustc/Cargo versions are rustc 1.98.0-nightly (`91fe22da8`, 2026-06-21) and Cargo 1.98.0-nightly (`a595d0da2`, 2026-06-20).

The four recorded Cargo build artifacts were hash-checked at their original absolute paths under `/workspace/bytes-proof-tools/targets/bytes`; all four match the receipt and archived copies. The reconstructed `public_records.rs` hash also matches the actual `OUT_DIR` copy. Those paths are location-bound and external to the archive; I did not relocate or rebuild them. A selected set of 81 captured probe proof/checker/native inputs also matches the current worktree byte-for-byte.

## Independent checker replay

I extracted the canonical archive to a temporary directory and reconstructed the captured repository/probe layout from its archived inputs. With no Cargo, rustc, Creusot, Why3, or solver invocation, the archived main correspondence checker and native checker both returned `pass`. I separately replayed the structural mutation suites: **76/76 main-checker controls rejected** and **47/47 native-checker controls rejected**, with no accepted controls. Replays wrote only into the temporary extraction.

The native gate checked the actual production `slice`, `new_empty_with_ptr`, `static_clone`, `static_drop`, and `without_provenance` bodies, source extraction and mapping, and 25 captured `ElaborateDrops` MIR bodies (24 production plus the client). It binds the selected `Range<usize>` specialization and valid-range normal path: original, first, and owner are dropped before the selected view is read; the result is saved before selected Drop. Empty uses `wrapping_add` and the provenance-free `STATIC_VTABLE` path, with no ticket/allocation authority and a no-op static Drop; nonempty clones, sets `len = end - begin`, and uses `ptr.add(begin)`. The captured 100 native runtime cases are execution corroboration, not proof.

The source correspondence records exactly one proof-only `OriginalSharedProof` enum declaration replacement relative to the AP positive. Root and Child payload tokens are preserved and View/Empty are added; it explicitly does not claim the inherited prefix is byte-exact.

## Scope limits

The proved target set is this selected nested-slicing probe, not complete crate-wide refinement. It covers the actual built-in `Range<usize>` path under valid bounds and normal completion. Arbitrary `RangeBounds`, invalid-range panic/unwind completion, concurrency, generic pointer provenance/arithmetical adequacy, and MIR adequacy remain outside this result or in the stated TCB. In particular, full original `bytes` API admission remains **NOT ADMITTED**.

Machine-readable checks and hashes are recorded in [AQ_CANONICAL_AUDIT.json](AQ_CANONICAL_AUDIT.json).
