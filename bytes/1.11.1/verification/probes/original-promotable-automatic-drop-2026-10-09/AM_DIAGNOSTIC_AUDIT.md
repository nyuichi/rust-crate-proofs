# AM diagnostic archive audit — 2026-10-09

## Result

Independent read-only audit of `evidence/am-positive-diagnostic-v1.tar.gz` and its receipt. The capture is internally hash-consistent and records a successful 115-file Creusot proof run. It is still **diagnostic, not admitted**: archived `probe/generated/correspondence.json` says `not_run`; receipt policy has `correspondence_exit_status: 2` and `diagnostic: true`. The mapping is marked `generated_unchecked`. I did not run the prover or correspondence checker.

## Archive and proof capture checks

- Archive SHA-256: `33eb52121a726455712ed12586a4dd002c4bb62bb024764f18fde11430199ea0`.
- Verified 5,913/5,913 regular tar members against the receipt SHA-256 rows; no duplicate, missing, extra, or hash-mismatched members.
- The receipt’s 115 target COMA paths exactly match `target_policy.included`; all 115 COMA and `proof.json` hashes match their receipt rows. Recounted proof trees: 824 prover leaves, 0 null leaves, 0 structural leaves, matching receipt statistics. `run.log` ends `Proved (115 files)`. This is proof-run evidence only, separate from correspondence status.
- Captured external inputs include 4,807 repository files, 110 private-Std files, and 6 tool configuration/activation files. They are hash-checked as ordinary archive members.
- Receipt scope is the closed nonempty `Box` first-promotion case, lexical child Drop, root read/return evaluation and normal terminal root Drop, both parity handlers declared; it excludes CAS-loser/concurrent behavior, unwind completion, arbitrary Drop/move equivalence, and whole-crate admission.

## Shadow source correspondence inspected

- The archive’s 12 `probe/src/*.rs` files compare against the captured AL source tree under `inputs/repository/bytes/1.11.1/verification/probes/original-promotable-first-clone-2026-10-09/src/`. Eleven are byte-identical (`erased_call.rs`, `event.rs`, `field_event.rs`, `free_effect.rs`, `lifecycle.rs`, `native_client.rs`, `owned_pointer.rs`, `physical_projection.rs`, `promotion.rs`, `provenance_specs.rs`, `public_shared.rs`). `lib.rs` changes only the Creusot promotion-module path to `generated/active.rs`.
- Base `promotion.rs` SHA-256 is `2c51555cc0c5eb9da7719609b2b2826e3e354251b802ceb3b440efa2b5f5f3f2`. `generated/active.rs` and `generated/positive.rs` are identical, SHA-256 `659d84cce0f5419f53e8cc22bb40902bf1bf8e6992057c9d03ee776b21d09826`; the active file is exactly the 722-line base as prefix plus the 49-line suffix containing `bytes_child_terminal_drop`, `bytes_root_terminal_drop`, and `promoted_automatic_scope`.
- The two ordinary adapter bodies forward to `cleanup_child` and `cleanup_root`, with strong ghost postconditions for root/child identity, map removal, phase, and completion/reclamation receipts. The added client calls the child adapter, checks child-only removal and root retention, reads the root to a `Vec`, saves that return value, then calls the root adapter and checks final completion. These are body-proved shadow operations, not trusted Drop axioms.
- This only establishes the shadow-side construction. The archived main native-to-shadow checker was deliberately not run, so no wrapper-to-native callback binding is established by this audit.

## Native source and MIR inspected

- Native client source hash: `e058fef98b928d8cbac872e7741ad6fe1bdd7f33f2059cae8a2375f7718cbdaf`. It constructs `Bytes` from `Box<[u8]>`, clones in an inner lexical scope, then reads via `AsRef<[u8]>::as_ref(&original).to_vec()` and returns the vector.
- Native test manifest/lock, test source, native test log, production manifest, and production `src/bytes.rs` all match the hashes recorded by archived `native-mir/capture.json`. The test source covers lengths 1, 2, 31, 256; archived log records one passing native test.
- All 19 selected `2-2-004.ElaborateDrops.after.mir` files match capture-manifest hashes. Pinned dump metadata is rustc `1.98.0-nightly (91fe22da8 2026-06-21)`, MIR opt level 0.
- In client MIR, the child `drop(_5)` is in `bb2` with normal successor `bb3`; after that, `AsRef` and `to_vec` run, and `move _7` to return slot `_0` occurs in `bb5`; root `drop(_2)` is in `bb6` with normal successor `bb7`, before `return` in `bb8`. This confirms the captured normal-edge order: child Drop, root read/copy, return-value evaluation, root Drop. Unwind paths are present in MIR but are excluded from scope.
- Production `Bytes::drop` MIR loads a drop function from its vtable and calls it with the `AtomicPtr` data field, pointer and length. The captured even/odd promotable Drop MIRs each construct a closure and call `AtomicMut::with_mut`; selected `release_shared`/`free_shared` MIRs are also archived. The source `bytes.rs` hash is `95789896965446187ecd5cc7bf52f447435fcdc40f4e19c0c49c5a59b22495fd`.

## Remaining admission boundary

`generated/mapping.json` explicitly lists native compiler/MIR and normal terminal-place elaboration, non-observation of receiver/data-field addresses and absence of independent field-drop glue, plus AL pointer/field/physical/erased-callback boundaries as TCB. Because the correspondence checker was not run, this archive alone does not verify the selected vtable entry and closure path bind to the adapters for both parity branches, nor the field-drop/no-observer premise. The native unit test does not force both pointer parities. Keep the result diagnostic until the frozen independent checker accepts the archived source, MIR and mapping.

## Post-capture live-tree delta

I compared current files to the immutable archive. None of `probe/src/*.rs`, `generated/active.rs`, `generated/mapping.json`, `native.rs`, or the 19 selected MIR body files differ. Current live copies of `capture-native.sh`, `check_native.py`, `native-check-controls.py`, `native-mir/capture.json`, `native-test/native-run.log`, `generated/native-check-controls.json`, and `generated/proof-targets.json` do differ. In particular, the live capture script and capture receipt include later native field-profile additions that are not members of this archive; I have not used those later files to claim archive provenance. Current `.why3find` caches and `verif` outputs also differ as expected after subsequent control runs.
