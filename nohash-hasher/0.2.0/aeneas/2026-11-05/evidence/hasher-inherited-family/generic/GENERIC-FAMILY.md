# Generic inherited Hasher contracts in four configurations

The original public calls write_length_prefix, write_u128 and write_i128 are
covered through compiler-inlined generic callers, for every represented T under
the original IsEnabled bound and every receiver state/input word. Debug prefix
fails assertionFailure when already written; otherwise it stores usize cast to
u64 and sets the flag. With debug assertions disabled it always overwrites the
stored word. Both wide methods fail panic for every u128/i128 and receiver state,
because their original standard defaults eventually call NoHashHasher.write.

These guarantees apply to dependency std/default and no-default-features, each
with debug assertions enabled/disabled, on pinned x86_64 nightly2026-09-17.
Actual feature resolution and compiler cfg guards/type layouts are saved.
Std/no_std prefix declaration blocks are byte-identical within each debug mode;
the wide panic declarations are identical in all four configurations. The
receiver types correctly differ between debug (u64,bool,PhantomData) and release
(u64,PhantomData). Different profile states are not silently identified.

Official Charon optimized MIR plus rustc +Inline is used throughout. Wide calls
also require +SimplifyLocals-final: it removes the unused, valid-for-all-inputs
native-endian byte-array computation after inlining the always-panicking write.
The first non-DCE unsupported transmute attempt remains historical evidence;
no transmute model, panic stub, source/LLBC or generated-body edit is introduced.
Actual body source spans connect the tiny UFCS callers to pinned core hash
and original nohash methods. This explicitly trusts compiler inlining/DCE and
official extraction. It is not a compiler-correctness proof or a direct default-
method source-card binding. Rust panic unwinding/messages/locations/global effects,
other profiles/targets/const evaluation remain outside these Result contracts.

Thirteen declarations (twelve configuration/API contracts plus the separate
debug already-written lemma) coimport with the old public Proofs module. All
transitive axiom prints use only propext, Classical.choice and Quot.sound. The
new guarded Lean jobs each exited0 in under3seconds and below2.8GB, with30second/
4500MiB limits. Pending states in STAGE-inline-family.json and the CHECKPOINT
Markdown describe an earlier paused stage; this final record supersedes them.
The separately frozen initial debug READY remains byte-identical.

The900-check Kani supplement is independent evidence with narrower concrete-T
and reachable-state domains; it is not a premise of these generic Lean proofs.
No source-card/recorded-source mapping, Aeneas primary count change, shared CLI,
installed tool or publication operation is made by this packet. Existing14
primary nohash claims and original source bytes are unchanged.
