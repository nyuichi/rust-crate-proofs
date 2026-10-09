# Independent audit of the canonical first-promotion capture

Date: 2026-10-09. Read-only review of the immutable canonical receipt and archive, with the source correspondence gate reconstructed from captured inputs. This audit did not run Creusot, Why3, a solver, a Rust build, or a Rust test.

## Archive and proof receipt

The canonical archive is `evidence/promotion-canonical-v1.tar.gz`, SHA-256 `53a500c22c725a259f20377e4da56c9e82bb4fc695400397c55940bf28e8eed1`. I recomputed all 4,428 regular-file member hashes and paths from the archive: all match the receipt, with no duplicate paths. The verified member ledger hash is `2aa0d5645bb3aebdf4b184a04d9d65884b7a86289c65d857120f7408a38fb729`.

All 112 archived Coma targets have matching archived Coma/proof hashes and no target is excluded. The included set, extracted Coma set, and receipt target set are identical; the feature list is empty and correspondence status is 0. Independent proof-tree recount gives **112 / 808 / 0 / 0** (files / prover leaves / null leaves / structural leaves), matching the receipt. Successful leaves use Alt-Ergo 761 times, Z3 40 times, and CVC5 7 times. The configuration limits concurrent prover processes to one; this is not a single-backend-only run.

The archived run log hash matches its member entry and ends in `Proved (112 files)`. Its first line records the 1024 MiB limit, one prover process, native weak orderings, and disabled sc-drf. The archived launcher checks the dependency feature tree for sc-drf and invokes Why3 with one worker. The captured installation manifest pins the tool and binary digests; the binaries themselves are not included in the archive.

The archive contains all 110 private `creusot-std` files at version 0.13.0. Their paths and bytes match the live pinned source directory and the captured tool manifest's source-file count and revision. All six captured tool/config files match their live counterparts, as do the eight live binaries named in the installation manifest.

## Independent correspondence replay

I reconstructed the captured `inputs/repository/` tree and `probe/` directory under `/tmp/al-promotion-canonical-replay`, restored the captured Cargo build-output evidence under the checker’s expected relative target layout, and ran only the Python correspondence scripts:

- Main source/native correspondence: **pass**.
- Main checker mutation controls: **25/25 rejected as expected**.
- Native source/MIR mutation controls: **24/24 rejected**, baseline pass, all preserved branch markers intact.

This replay used archived source and support inputs, not mutable live proof sources. The Cargo root-output sentinel contains a location-specific path. I verified the original captured sentinel hash and its actual live Cargo path separately; in the scratch replay I regenerated only that path string for the scratch output directory. The main checker then verified the reconstructed output location, source-derived record bytes, and compiled-record equality. No source or proof input in the workspace was changed by the replay.

All 61 archived production `src` files, production Cargo manifest/lock, all 12 proof `src/*.rs` files, imported checker modules, selected probe code/build inputs, and 19 native MIR bodies match their current live copies. The captured Cargo fingerprint, build-script output, root-output record, and compiled `public_records.rs` each match their receipt hashes at the actual recorded live paths. The compiled record bytes also match the generated source copy and the independent source reconstruction performed by the replayed main checker.

The MIR capture records 19 distinct bodies (18 production plus the client) at `2-2-004.ElaborateDrops.after.mir`, pinned to rustc 1.98.0-nightly (2026-06-21) and cargo 1.98.0-nightly (2026-06-20). The archived native test source covers lengths 1, 2, 31, and 256; its archived log records the single test passing. The native checker describes this as source/compiler-output correspondence and execution corroboration, not a bytes proof.

## Scope and limits

This closes the earlier source-correspondence blocker for this exact closed client: nonempty `Box<[u8]>`, first successful promotion, child explicit cleanup, root read, then root explicit cleanup on normal return. The source checker binds that path to production API source and selected native MIR and validates the recorded affine/state wiring.

It does not verify all of bytes 1.11.1. CAS loser behavior is retained in the native trace but excluded from the mathematical client premise; concurrent writers, automatic Drop, unwind, allocator failure, arbitrary API compositions, and other configurations remain outside scope. Generic AtomicPtr/Std interpretation, pointer provenance, allocation and borrow/free primitives, callback erasure, and compiler/MIR adequacy remain explicit TCB boundaries. The proof establishes typed storage deallocation, not arbitrary `T::drop` behavior.

The seven semantic diagnostic captures and five frontend rejection captures are separately bound in `evidence/promotion-negative-controls-audit.json` and `evidence/promotion-type-controls-audit.json`. Missing-free `Ghost::conjure` cases are proof-sensitivity VCs, not direct physical-free receipt theorems. The initial E0507 direct-move experiment is an ownership rejection, not a stale-history theorem.

The structured record is `ROOT_AUDIT.json`.
