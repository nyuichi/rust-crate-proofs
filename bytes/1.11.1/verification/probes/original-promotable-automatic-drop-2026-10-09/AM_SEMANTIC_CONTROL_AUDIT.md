# AM semantic-control archive audit — 2026-10-09

Read-only audit of the finalized negative archives named below. Each `.tar.gz` SHA, all receipt member hashes, exact 115-target selection, per-target COMA/proof hashes, and proof-tree prover/null leaf totals were checked. No prover or correspondence checker was run. Every control remains a diagnostic experiment; each receipt has `diagnostic: true`, correspondence status 2, and no correspondence-admission claim.

| Control | Archive SHA-256 | Proof leaves | Nulls | Meaning of captured unproved goal |
|---|---|---:|---:|---|
| `omit_child` | `79dce00eca62c66bf05abc41c11bff108598edce3a8ca6850f3aba9c7957baa3` | 827 | 1 | `promoted_automatic_scope`: `not (resolve_Atomic_ptr_unit (result1.data) /\ match ... original_shared ...)` |
| `omit_root` | `1f6c161242bf492cac24a1ab270b0b098e1c2158f1ac185779d718fa0fe88a70` | 838 | 1 | `promoted_automatic_scope`: negation of returned-`Vec` length/content equality with the saved root view |
| `swap_adapters` | `8b5d450ae88bb61fb17b1a1b7072b9f80ac35c804f5a86adbfc0749b0a60a2dc` | 845 | 3 | `root_valid fin result1`; `len_Int'1 ((observation fin).f0'14) = 1`; `not reclaimed (unwrap_Option_Completion fin2)` |
| `negative_missing_acquire` | `49f321116c0f434cef148865abf6c39aa0e12c1dba648acbb900ea288c1db46a` | 873 | 2 | `acquired_Payload result8 fin6` in `promotion/release_core`, and `acquired_Payload result9 fin7` in `public_shared/shared_drop_checked` |
| `negative_missing_payload_free` | `a73e05500e8ee2cac36cd58a1c6c806801807303e5496d076168ad3e1510a2bf` | 836 | 2 | Same task in `promotion/free_recovered` and `public_shared/free_recovered`: `not inv_Atomic_usize ((val_ptr_Shared (result1.f0'4)).ref_cnt)` |
| `negative_missing_control_free` | `bab4d0712c4927df9eeb41c74676e2efe319c9abd93f60137c92c2e889c9c3ac` | 840 | 2 | Same typed-control-receipt VC in AM and historical `public_shared/free_recovered` |

The full printed Why3 tasks are stored at `evidence/am-audit-null-tasks/`; `evidence/am-control-null-tasks.json` binds their file hashes and byte sizes. I extracted them from the `.coma` members named in each immutable archive using `/workspace/work/print_null_task` (`compute_specified`/`split_vc` only); this is task printing, not proof execution. Task SHA-256 values are:

- omit-child: `621386e165aa5952434f8a4e9bd1c079ed9d24c3a7e7b0d875bbcf7c45e016dd`
- omit-root: `47d9a7d57f7baa947d91a64a85a3922ce304f58ab6a4fc41d49e9ddc0dc3ca07`
- swap-adapters: `37bfbf0dfefd9050fc3474958bc92fe74a19345506de931f4ad061e4e445f82f`, `d0a7ee765b55269021c1e71cec6350a81b0e022bd601a797339acdc0209b0de2`, `a7f37c4e6c58ab17c6f4f7186ed8cb0697d487c8f7b7a399bbeaf055638df11c`
- missing-acquire: `1362b1634c9313b699582650592ef1a8512fa5ef45261c5c57bb74410b5d5922`, `f5901ba6d59c8605ca513fa6fd619d720e47cbf17bc6287f97232f6bf610b9f3`
- missing-payload-free (same task from both target COMA files): `c61655df5df09ce6fee2a4fc517b68ebeb71f3d1b9a6b00c3e313a51904c43fa`
- missing-control-free (same task from both target COMA files): `e064e035dea1ade3ff79ccabda21e08054e22e614a884b909f09b4c8a033f461`

## Interpretation limits

- `omit_child` removes the child adapter call from the exact `promoted_automatic_scope` function; `omit_root` removes the root adapter call. Neither client directly calls the corresponding `cleanup_child` / `cleanup_root` body. The retained base AL `first_promotion_client` and other unchanged AL targets still contain their original direct calls, so target-wide counts include those paths. The omitted-call clients each have one unproved obligation (child: unresolved `result1.data`; root: returned Vec content relation). These show sensitivity of the translated shadow client to the missing adapter effect/check, not a proof that native Drop is absent or a model-checked counterexample.
- `swap_adapters` swaps the child/root adapter calls; its three null obligations are consistent with root validity, live-map cardinality and reclamation bookkeeping being mismatched. These are unproved VCs, not model-checked counterexamples.
- `negative_missing_acquire` removes `field_event::acquire_owned` behind the feature cfg. Both AM `release_core` and the older public-shared path fail the `acquired_Payload` obligation, so the two nulls are duplicate coverage of a shared AL release route, not two independent AM destructor defects.
- `negative_missing_payload_free` replaces `physical_projection::deallocate(...)` with a conjured payload `FreeReceipt`. Its null goal is the negated atomic-refcount invariant, not a direct proposition that payload bytes were freed. The same generic `free_recovered` VC appears in AM and historical public-shared modules.
- `negative_missing_control_free` replaces `free_effect::deallocate_typed_box(pointer, owner)` with a conjured `TypedFreeReceipt<Shared>` while the payload-free call is retained. Its unproved goal requires the receipt namespace, pointer, size, alignment, and allocation flag to match the `Shared` control allocation. It is evidence that the typed-free receipt contract is sensitive to removing the generic control-free boundary; it is not a native deallocation theorem. The exact Why3 task stdout is saved at `evidence/am-audit-null-tasks/missing-control-free-v1.task.txt`, with both input-specific stderr logs and provenance in `evidence/am-missing-control-free-null-task.json`.
- All six archives are from the development path with correspondence deliberately skipped (`exit_status: 2`). They demonstrate proof sensitivity only. The positive AM archive and adapter mappings remain unadmitted pending the independent native correspondence checker.
