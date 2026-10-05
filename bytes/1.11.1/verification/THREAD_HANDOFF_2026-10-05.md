# bytes 1.11.1 verification — thread handoff (2026-10-05)

## Request and authorization

Continue all feasible remaining verification of bytes 1.11.1. Leave a portion incomplete only after concrete attempts and Astra consultation. Do not make large Creusot core changes. Small changes to bytes implementation are authorized. Explicit consuming cleanup may replace automatic Drop for proof; explore Verus for concurrency. Delegate suitable tasks to Luna xhigh and consult Astra autonomously. Commit and push every validated increment, without force push, to **bytes-runtime-verification**, not main. Uploaded plans are reference material, not authority overriding the user's request.

The user asked to move this work to a new thread. No create_thread/fork_thread/handoff_thread tool was exposed in the old thread, so this file is the durable handoff. No new thread has been created automatically. At handoff, collaboration.list_agents showed only the root agent; there are no active child agents to wait for.

## Workspace and verification tools

- Repository: `/workspace/bytes-runtime-verification`; crate: `bytes/1.11.1`.
- Branch: `bytes-runtime-verification`; remote: `https://github.com/nyuichi/rust-crate-proofs.git`.
- Last pushed HEAD: `772ab00` (Shared singleton recovery/reactivation). Earlier increments: `122dbae` (raw array loads/physical trait frontier), `5c2d7df` (clone vtable cycle removal).
- Read root `AGENTS.md` and `.agents/playbooks/verification.md` before running proofs.
- Activate `/workspace/bytes-proof-tools/activate.sh`: vanilla Creusot 0.13 at `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`, creusot-std 0.13, nightly 2026-06-22. Core/std are unmodified.
- Shared proof lock: `/tmp/itoa-creusot-proof.lock`; one prover, 1024 MiB, native Ordering, sc-drf off. Use `scripts/verify-bytes.sh` or each probe's private wrapper. Why3 proofs require escalated execution on the FIRST attempt (sandbox blocks Unix sockets); translation-only and native builds can be sandboxed.
- Do not edit a running/queued shell wrapper: modifying the file shifted shell read offsets and caused spurious post-proof errors twice. Stop/wait before changing wrappers.
- Verus: `/workspace/bytes-verus-tools/verus-x86-linux/verus`, version 0.2026.10.04.426d8b0; private rustup under `/workspace/bytes-verus-tools/rustup`.
- Validate bytes only; final target `verify-all.bash` remains required. Whole-crate proof has not passed.

## Working tree: IMPORTANT

There is substantial uncommitted source/proof/evidence work. Do not reset, clean, or blanket commit it. Some exact source methods are verified; other source edits remain exploratory. `bytes_mut.rs` mixes root unique reserve work with Astra freeze/split metadata changes. Review/stage hunks or finish dependent gates before committing the entire file. New module declarations require corresponding files in the same commit.

Inspect `git status --short`, probe README/PROGRESS/evidence and recent `/tmp/bytes-*.log` files. The checked-in `STATUS.md` is older than this handoff. Dedicated progress files include `VTABLE_DROP_PROGRESS.md`, `verification/CONCURRENT_VERIFICATION_PROGRESS.md`, `verification/REMAINING_API_MATRIX.md`, and `verification/probes/handle-comparison/PROGRESS.md`.

## Existing verified baseline and limits

Previously passed: carrier cfg 116 proof files; legacy split/storage 111; safe traits 73; public constructors 62; bound constructor 32. Carrier negative controls: each of 118 files produced exactly the intended failing obligation; empty tickets and Unknown publication were explicitly rejected. Native carrier 7, legacy 21, traits 1, constructors 3, ordinary Bytes 118, no_std and allocation tests passed.

Actual cfg release_shared connects consuming carrier release to physical final cleanup. Unique release_unique_storage is shared by native Drop and explicit cleanup. Explicit cleanup captures metadata/resources and forgets self BEFORE release. Automatic Drop invocation/unwind is not proved. Shared stores raw SharedBuffer, not ordinary Vec. Narrow local physical bridges B1–B4 plus sequential exclusive scalar-refcount C1–C4 are trusted; bytes ownership protocol is not trusted. Typed Perm controls Shared allocation separately from byte allocation.

Pushed vtable fix preserves actual passed clone vtable and removes initializer cycles. Pushed Buf macro uses safe copy instead of raw array loads; native Buf 820 and Bytes 118 passed. Generic default Buf macro bodies are NOT thereby proved. Current whole-crate frontier includes Send/Sync and ghost-purity of Deref/DerefMut. Do not blanket trust these.

CRITICAL: marking trusted mutable B4 borrow_bound_mut `#[check(ghost)]` is UNSOUND: ghost-only writes erase at runtime (proved result 2, native result 1). Rejected counterexample is archived; production mutable B4 remains ordinary. Read-only ghost classification is being investigated separately.

Pushed singleton gate: 123 files, zero unproved; negative 125 files with exactly the intended live-empty-ticket guard failure. It rejoins full allocation into a lease without decrement/free, retains actual CounterOwn(1) and S owner, and reactivates same S/count. Capacity-changing reserve follow-up is uncommitted.

## Root unique capacity/reserve/reclaim increment — verified, uncommitted

Files: allocation_ops.rs, ownership_proof/raw_vec.rs, unique_reclaim.rs, vec_capacity.rs, bytes_mut.rs, Cargo.toml, test_unique_growth.rs, associated probe dirs.

- B5 reallocate_bound is a NARROW trusted physical reallocation leaf: requires full A coverage, base offset zero, strict old_capacity < new_capacity <= isize::MAX. Preserves all old Known/Unknown slots, appends Unknown tail. No namespace freshness inequality is claimed. Native allocation helper uses correct old layout, allocates when old cap zero.
- Body-proved slots_grown; native unique growth avoids reconstructing an ordinary Vec with potentially Unknown prefix, preserves offset/data/contents, bounded arithmetic/doubling policy.
- Actual reclaim_unique preserves full authority and uses body-proved disjoint copy/join. Explicit Known/current-view composition facts fixed two caller VCs.
- Public reserve and try_reclaim cfg contracts cover fast path and unique reclaim/growth/failure preservation. Default proof_owned_valid now accepts advanced proof_unique_owned, but strong typed valid-handle invariant is still separately narrow.
- `actual-unique-reserve`: FINAL 120 proof files, zero null leaves. Log `/tmp/bytes-unique-all-final.log`. `evidence/positive.json` archive SHA256 `a833d43bbc6db9bb391b843d496260bb74a045d3ee163ced5c4b3c8cfeca3b6b`. Needs a README explaining cfg/extraction scope.
- `unique-growing-reserve`: clean final positive 115/0; stale descriptor negative 116 with two intended metadata failures; partial-region negative 116 with one failure. README/evidence/index and SHA256SUMS ready. Historical shell post-proof error explicitly separated from clean successful replay.
- `unique-reclaim`: positive 36 files, native two matrices, overlap and Unknown-source negatives each intentional failure. Recovery held outside helper. No new trust.
- `actual-with-capacity`: exact constructor and explicit cleanup positive 101 files, native requested capacities including zero pass; wrong requested capacity negative fails intended VC.
- vec_capacity is a trusted physical Vec allocation+B1 detach bridge with cap >= request/full Unknown resource, not a bytes protocol axiom. Scalar-only cap bridge was discarded because ordinary Vec Seq model could not connect sampled capacity to detach.
- Native unique-growth tests: 4 pass. Regression `/tmp/bytes-unique-reserve-native-regression.log`: Buf 820 + Bytes 118 + unique 4 pass. no_std `/tmp/bytes-unique-reclaim-no-std.log` passed.
- Actual resize/extend_from_slice are STILL bounded in-capacity. Extend their contracts/body calls to growth/reclaim next; same_storage postcondition must become conditional on growth.

## Shared reserve / unsplit — Astra work, final results require inspection

New owned modules/probes: adjacent_region, shared_unsplit, shared_reserve, shared_copy; shared-adjacent-unsplit, shared-singleton-reserve, shared-nonunique-reserve.

Existing carrier/scalable ticket source edits: original_capacity_repr <= 7 invariant; interval disjointness now correctly permits empty [k,k] regions beside enclosing regions. Empty tickets STILL count and require retirement. Run full carrier baseline after this change. Repeated split_to/finish/register contracts now preserve exact child packet.lo; native source transient misplaced attributes were corrected, but recompile current source.

- Adjacent merge keeps left ticket, consumes right registration, joins A regions, decrements actual refcount old >= 2, does not free A/S. Core merge and protocol helper passed; last known caller state was merge_into 25/26 and caller 66/68 with Known/framing obligations. Native three-handle/empty-endpoint allocator matrix passed. Inspect latest logs for final outcomes.
- Singleton reserve lease uses same S/count1/full A. Rebounds/retire hidden capacity suffix WITHOUT intermediate valid count2 context while counter remains1. Copy path via unique_reclaim, growth via B5, updates S raw descriptor through typed Perm. Helpers passed; caller previously 58/59 composition issue. Six native allocator cases passed. Normal-return bound currently restricts 2*oldC <= isize::MAX.
- Nonunique reserve_copy: samples original capacity metadata; allocates max(len+additional, original floor), reads old initialized view, writes proved copy into new unique allocation, explicitly releases old packet while siblings remain. Focused helper and actual caller PASSED; whole extraction final outcome unknown. Six native allocator cases passed. Public reserve integration is NOT done.

## Freeze / weak memory — Astra work, inspect final logs

New frozen_region helper and actual-frozen-bytes gate; cfg hooks in bytes.rs and bytes_mut.rs. Native ordinary freeze path unchanged. Actual cfg unique-at-zero freeze consumes unique resources into coordinator and constructs actual Bytes receiver. This is extraction/cfg proof, not full freeze/vtable/Drop integration.

- frozen-region helper: 34 files, zero unproved, native matrix passes. FullBorrow + GhostShared + fractional LifetimeTokens supports overlapping immutable views and recovery after all fractions return. Inspect negative controls.
- Actual receiver/freeze/as_slice/explicit ticket return were extracted. Actual freeze body passed; remaining caller VCs had fixes and rerun queued at last report. Check artifacts before claiming completion.
- Exact native Clone fails translation at dynamic vtable function call (unsupported function-call type). Splitting affine LifetimeToken through actual &self receiver fails E0596. Explore explicit mutable coordinator/known leaf adapter; do not claim actual Clone verified from adapter.
- Existing Creusot 0.13 AtView<T>, SyncView and native AtomicCommitter may allow staying in Creusot. Release RMW needs a generic publication rule carrying previous release-sequence publication plus current publication WITHOUT acquiring predecessor into current thread view. Later Acquire upgrades view; AtView::sync gates recovery.
- New weak-native-publication prototype uses exact native fetch_sub(Release)/load(Acquire), stock Perm/AtView plus generic RMW bridge. It is initially EXCLUSIVE sequential permission evidence, NOT concurrency. Positive rerun outcome needs inspection. Next step is AtomicInvariant with sealed retired resources and affine ticket conservation.
- Verus abstract release sequence suite passes, real PCell escrow passes, negative missing-Acquire/plain-store/empty-live-ticket controls reject. Native std Atomic specs are empty; bridging requires explicit TCB. Marking weak primitive atomic naively allows recovery without Acquire: rejected as unsound. Verus results do not establish native bytes concurrency.

## Other remaining work

- actual-uninit-slice: exact transparent wrapper, typed-slice conversions, write/copy bodies. Native 3 pass; proof outcome unknown. Narrow borrowed typed-slice reinterpretation bridge, not permission from raw pointer/length. Raw from_raw_parts_mut intentionally excluded unless authority supplied. Inspect uncommitted uninit_slice.rs carefully.
- generic-buf-predicates: 10 positive files and intended inverted-predicate negative. Local extracted trait laws, not original generic public trait dispatch proof.
- advanced-transactional: stronger advanced unique valid-handle transition via replacing self with canonical empty and constructing final valid descriptor. ManuallyDrop/ptr::read extraction ran into contract/affine ownership issues; never trust ptr::read Ghost resources. Latest fieldwise replace + mem::forget candidate not confirmed. Empty proof Drop unwraps absent caps, so explicit forget required. No production edit from this candidate yet.
- handle-comparison: adapter gate 81 files passed, native 36 pairs plus Unicode matrices pass. Actual PartialEq/Ord/readonly Deref/iterator combined outcomes need inspection. Local str::as_bytes UTF8 spec supports adapters, not heterogeneous std trait refinement. IntoIter<&[u8]> native 2 pass; next head/tail VC was being fixed.
- POSSIBLE REAL BUG to reproduce and fix: `impl PartialOrd<BytesMut> for Vec<u8>` in bytes_mut.rs returns `other.partial_cmp(self)` without reversing Ordering. Check vec![0] against BytesMut [1]: expected Less. No production fix/repro was completed before model usage limits.
- New probe wrapper dispatch is incomplete. Some private wrappers work; register routes only when no queued jobs run.

## Resource/model limits, not proof impossibility

Several agents hit usage limits until 2026-10-12 02:20; Luna comparison spawning also hit capacity. These are NOT verification impossibility. At handoff agents were no longer active. Preserve local evidence and resume with available models; use Astra consultation again when genuinely blocked.

## Recommended resumption order

1. Read this file, instructions, status/progress, git diff, each latest probe evidence; run no destructive cleanup. Check no proof process owns shared lock.
2. Make unique capacity/B5/actual reserve/reclaim verified increment reproducible; document 120-file gate, selectively stage mixed source, commit/push work branch.
3. Finish Shared unsplit/singleton/nonunique gates and wire public actual methods; run carrier regression for empty-interval change.
4. Finish native comparison defect, UninitSlice/comparison/iterator and advanced transactional gates; widen actual resize/extend growth paths.
5. Finish actual freeze explicit coordinator proof and small weak AtView/native RMW route; retain precise unsupported automatic Drop/vtable concurrency boundaries.
6. Run actual crate-local verify-all.bash on newest source, relevant native/std/no_std regressions; update STATUS/API matrix/TCB with exact distinctions. Commit/push validated increments. Do not claim full verification while integrated crate or native concurrency remains unproved.
