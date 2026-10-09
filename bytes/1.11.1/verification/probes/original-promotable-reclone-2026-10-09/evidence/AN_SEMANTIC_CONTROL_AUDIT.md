# AN semantic control archive audit

Read-only audits of archived AN semantic controls. The machine-readable record is [AN_SEMANTIC_CONTROL_AUDIT.json](AN_SEMANTIC_CONTROL_AUDIT.json). Exact task text and stderr are under `an-semantic-null-tasks/`. All printed tasks were regenerated from their archived Coma inputs. No prover/build or Rust source edits were performed.

## Raw branch inversion

`an-negative-raw-branch-v1.tar.gz` (`0d7fa3ea…08a06a3e`) has 7,298 verified members and 120 included targets. The mutation flips `kind == 0` to `kind != 0` in both Shared-phase reclone dispatchers. Results are 120 files, 986 prover leaves, and two null leaves. Even and odd goals each report 37/38; both nulls are at `[0,11]` and print `not fin.final = fin.current`. This is bounded cursor/borrow progress sensitivity, not a standalone functional counterexample or global bytes-law refutation.

## Stale view

`an-negative-stale-view-v1.tar.gz` (`51d5ec1e…3582bcf2`) has 7,454 verified members and 120 included targets. Results are 120 files, 1,002 prover leaves, and four null leaves. The mutant replaces `phase.current` with a fresh `SyncView` before each snapshot. For even and odd, one null is a pointer `visible_is` obligation at the reset view timestamp (`[0,11,2]`); the other is `reclone_result` (`[0,18,1]`). These show visibility/history and dependent result-contract sensitivity, not independent memory-safety or functional counterexamples.

## Missing registration

`an-negative-missing-registration-v1.tar.gz` (`63c9911e…adeba708c`) has 7,543 verified members and 120 included targets. Results are 120 files, 937 prover leaves, and two null leaves. Removing `lifecycle::State::on_register(...)` from the `shallow_clone_arc_checked` callback leaves `shot_store_AtomicUsize (c.final)` open (`[0,12,7,4]`) and fails the compound returned `ChildProof` invariant (`[0,20]`). The helper goal reports 30/32. These are store-history and child-proof obligations, not full-client behavioral refutation.

## Omitted terminal effects

- `an-negative-omit-second-v1.tar.gz` (`4d36623b…871e0f12`): 7,578 members, 120/921/1. `promoted_reclone_scope` reports 8/9; at `[0,5]` the unproved formula is the negation of resolution for `result2.data` and its Root/Child proof.
- `an-negative-omit-first-v1.tar.gz` (`eeb9ffbf…2f2c64ca0`): 7,618 members, 120/927/1. `promoted_reclone_scope` reports 14/15; `[0,10]` negates the child removal, same-root/pointer-owner, and root-reclaimed ledger condition.
- `an-negative-omit-root-v1.tar.gz` (`ffd2bc72…764b6f3a3`): 7,672 members, 120/943/1. `promoted_reclone_scope` reports 31/32; `[0,25]` negates the returned Vec length/content equality condition.

These are proof/ledger and postcondition sensitivities to omitted helpers. They do not show that a native destructor failed to run or provide a concrete output counterexample.

Each archive includes all 120 targets, no exclusions, and remains diagnostic with correspondence status 2. Plugin warnings appeared on printer stderr but all print commands exited successfully. A null reports only that the particular archived VC leaf was not discharged.

## Swapped child places

`an-negative-swap-child-places-v1.tar.gz` (`45f8cf36…e46d493a4`) has 7,727 verified members and 120 included targets. Results are 120 files, 947 prover leaves, and one null leaf. The generated proof client pairs each child handle with the other child’s terminal receipt. `promoted_reclone_scope` reports 34/35; the null at `[0,11]` is the observation equation removing the expected child’s `child_id`. This is receipt/owner identity correspondence sensitivity, not a byte-value or runtime counterexample. The task was printed from the archived Coma file and is saved with its stderr.

## Omitted acquire transition

`an-negative-negative-missing-acquire-v1.tar.gz` (`75e50022…b1a0250`) has 7,867 verified members and 120 included targets. Results are 120 files, 963 prover leaves, and two null leaves. Its proof-model feature omits the `field_event::acquire_owned` transition when the last owner is released. It leaves `acquired_Payload` obligations open in `release_core` (30/31, path `[0,26,1,1]`) and inherited `shared_drop_checked` (35/36, `[0,29,1,1]`). Both task texts were replayed from archived Coma files. These are the same acquire-model omission affecting two proof paths; the feature does not alter native production Rust or run a weak-ordering implementation.

## Missing payload free proof receipt

`an-negative-negative-missing-payload-free-v1.tar.gz` (`387d6a98…7e3237cb`) has 7,909 verified members, 120 targets, and 120/926/2 results. The proof-model mutation substitutes `Ghost::<FreeReceipt>::conjure()` for the payload deallocation witness in both the AN and inherited shared `free_recovered` targets. Each reports 12/13 with a null at `[0,11]`; the printed formula is `not inv_Atomic_usize (Shared.ref_cnt)`. This is an Atomic invariant obligation in the substituted model, not a direct counterfeit-receipt theorem. The paired targets show one feature across two scopes, not two independent defects.

## Missing control-block free proof receipt

`an-negative-negative-missing-control-free-v1.tar.gz` (`4c7cdd80…81a93aff`) has 7,942 verified members, 120 targets, and 120/930/2 results. It substitutes a conjured `TypedFreeReceipt<Shared>` for `deallocate_typed_box` in the paired free paths. Each reports 13/14 with a null at `[0,12]`: the task requires the control block's namespace, pointer, size, alignment, and allocated condition. This is sensitivity of the typed control-free receipt obligation; it does not establish a concrete native leak.

For both controls, the two free targets are paired uses of the same feature. The captures are diagnostic, and the nulls are confined to their recorded proof obligations.
