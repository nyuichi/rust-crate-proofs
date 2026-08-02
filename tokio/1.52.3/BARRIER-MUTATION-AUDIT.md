# Tokio Barrier mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Use `>` instead of `>=` in the follower watch check | `verify_barrier_mutants_rejected` and `verify_two_party_publication` | the leader publishes exactly the captured generation, so the follower never completes |
| Replace the defensive generation advance with build-mode-dependent arithmetic | `verify_generation_advance_arithmetic_is_total` and `generation_advance_arithmetic_is_build_mode_independent` | the arithmetic helper differs between debug and release; S03's earlier terminal gate makes rollover unreachable in the integrated Barrier policy |
| Reset `arrived` to 1 after release | `BarrierMachine::arrive` contract and protocol conservation | the next cohort starts with a phantom arrival |
| Publish the incremented generation | exact `published_generation` postcondition | the watch token no longer equals the cohort captured by followers |
| Reserve only the update, not all `n - 1` Receiver credits | `verify_three_party_clone_credits_and_publication`, terminal three-party regression | a later follower can panic after earlier arrivals committed, stranding the cohort |
| Commit an arrival before its reserved Receiver clone | orchestration invariant and insufficient-credit regression | clone failure can leave an admitted arrival without a waiter |
| Reject terminal capacity after changing `arrived` | `verify_last_three_party_cohort_and_terminal_rejection`, mutation-free panic regression | a caught terminal panic corrupts the next cohort boundary |
| Advance Barrier generation independently of the watch update count | orchestration invariant `generation = update_generation + 1` and terminal cross-state fixtures | followers compare tokens from unrelated logical generations, or an unreachable terminal state is treated as evidence |

Mutex exclusion, watch storage/wake execution, and scheduler liveness remain the
frozen common foundation. S03's finite terminal gate occurs before a complete
machine-word Barrier generation cycle, so admitted generations are never reused.
