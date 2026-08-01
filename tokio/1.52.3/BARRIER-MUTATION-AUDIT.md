# Tokio Barrier mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Use `>` instead of `>=` in the follower watch check | `verify_barrier_mutants_rejected` and `verify_two_party_publication` | the leader publishes exactly the captured generation, so the follower never completes |
| Increment the final generation with debug-overflowing `+= 1` | `verify_generation_wrap_is_non_panicking` and `generation_wraps_without_panicking` | debug and release builds disagree and the final-generation leader unwinds after publication |
| Reset `arrived` to 1 after release | `BarrierMachine::arrive` contract and protocol conservation | the next cohort starts with a phantom arrival |
| Publish the incremented generation | exact `published_generation` postcondition | the watch token no longer equals the cohort captured by followers |

Mutex exclusion, watch storage/wake execution, and scheduler liveness remain the
frozen common foundation. A follower delayed for at least one complete
machine-word generation cycle is also a finite-counter liveness boundary; no
memory-safety or unique-leader claim depends on excluding that delay.
