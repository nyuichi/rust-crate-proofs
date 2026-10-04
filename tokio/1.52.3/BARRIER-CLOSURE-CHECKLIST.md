# Tokio Barrier S08 closure checklist

This file is the specification map and closure checklist for promoting S08
from legacy `L / R / I` evidence to current `C / R / I`. It applies only to
Tokio 1.52.3's standard `sync` Barrier implementation.

## Contract and composition map

| Obligation | Production / proof connection | Status |
|---|---|---|
| channel construction | `watch::channel(0)`, one retained Receiver, `generation = 1`, `arrived = 0` | reviewed; leaf proved |
| `n = 0` and `n = 1` | zero normalizes to one; every wait is the leader | reviewed; leaf proved and tested |
| arrival count | the synchronous Mutex serializes `arrived += 1`; between successful releases `arrived < n` | reviewed; leaf proved |
| exactly one leader | exactly the nth arrival takes the leader branch for each successful cohort | reviewed; protocol proved and tested |
| generation reset | initial Barrier generation is one and remains exactly one above the watch update count; leader publishes, resets, and advances both | reviewed; leaf and orchestration proved |
| follower wait/recheck | clone Receiver, register through `changed`, then accept the exact published token with `>=` | reviewed; refinement proved |
| cancellation | dropping a follower future does not roll back its counted arrival; the next nth arrival still opens the cohort | reviewed; protocol proved and production-tested |
| repeated cohorts | successful release clears the count and one leader is selected in every later cohort | reviewed; protocol proved and ten-cohort test integrated |
| watch terminal behavior | first arrival reserves one publication and `n - 1` Receiver credits; insufficient capacity panics mutation-free | reviewed; orchestration proved and terminal-tested |
| Mutex/watch composition | the Barrier critical section owns the sole Sender update and serializes reserve/admit/publish/reset | reviewed; orchestration proved |
| `BarrierWaitResult` surface | leader-bool construction, `is_leader`, and clone observation preserve the branch result; Debug is observational only | behavioral map body-proved; Clone/Debug runtime-tested |
| `Send` / `Sync` / `Unpin` | Barrier and result traits plus `wait` future traits under `sync` | compile assertions integrated |
| cfg and tests | standard `sync`, `full,test-util`, tracing wrapper, and exact Barrier test selection | standard suite integrated; tracing wrapper is an R08 projection |

## Proof boundaries

1. Keep `barrier_protocol` as the unbounded logical cohort/leader model.
2. Keep `barrier_refinement` as leaf arithmetic for one successful critical
   section and the follower comparison.
3. Add a thin production orchestration that composes construction, one
   successful leader publication, follower recheck, cancellation, repeated
   cohorts, and public result mapping without reopening the leaf proofs.
4. Model the selected watch-terminal policy as a separate leaf before claiming
   the orchestration or ledger row current-closed.

No temporary trusted component is planned. Mutex execution, the body-proved
S03 watch contract above its frozen storage/wake adapters, and scheduler
liveness remain the recorded foundation. The taskdump-gated trace projection
belongs to R08.

## Terminal composition policy

At a clean cohort boundary, the first arrival preflights the sole Sender's
remaining update capacity and cumulatively reserves `n - 1` Receiver creation
credits. Failure releases the Mutex and panics before changing `arrived`, the
Barrier generation, or its reservation count. Every follower constructs its
private Receiver from one reserved credit before committing its arrival. The
nth arrival therefore cannot encounter a watch-resource panic midway through
an admitted cohort; it publishes, resets, and consumes the single update.

The last admitted three-party cohort and its follower wake-up, next-cohort
mutation-free rejection, insufficient multi-follower credit rejection, and
non-cancel-safe follower drop are runtime regressions. `barrier_orchestration`
body-proves the same finite-resource composition. Because the S03 terminal
gate occurs before a full Barrier generation cycle, the wrapping-generation
leaf remains valid arithmetic but is unreachable in the integrated policy;
no full-cycle follower liveness exception is needed. S08 is current-closed for
standard `sync` above the recorded foundation.

The orchestration uses the mathematical relation `barrier_generation =
watch_update_generation + 1`, initialized by `watch::channel(0)` and
`BarrierState::generation = 1`. Terminal fixtures construct that exact paired
state, the last admitted cohort advances both sides once, and the subsequent
mutation-free rejection preserves the positive, never-reused Barrier token.

## Stop conditions

- Do not add certificates to the leader body to hide the terminal mismatch.
- Revisit the production interface after two same-shaped composition failures.
- Stop after three failures in one area or 30--45 minutes without structural
  progress.
- Validate leaf milestones in the verification crate; reserve
  `verify-all.bash` for an integrated closure checkpoint.
