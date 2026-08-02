# T01 cooperative budget and yield mutation audit

## Scope

This audit covers the selected T01 slice in `src/task/yield_now.rs` and
`src/task/coop/*`: the exact `Budget::decrement` branches, the accessible-TLS
branch of `poll_proceed`, `RestoreOnPending`, scoped budget restoration,
unconstrained nesting, and the register/recheck ordering of `yield_now`.

The Verus projection is a logical, production-shaped state refinement in
`verification/src/{coop_refinement,coop_tls_refinement,defer_refinement}.rs`.
Production correspondence is checked by
source comparison, the crate-local `task::coop::test` regressions, the public
`task_yield_now` integration test, and the existing current-thread and
multi-thread runtime loom yield cases. Pin/Poll/Context/Waker mechanics and
scheduler liveness remain frozen foundation boundaries. Narrow
generic TLS/Cell semantics are represented by `vstd_ext::thread_local`.
`ThreadLocalBudgetCell` and `SchedulerContextAccess` are body-proved Tokio
mappings over that common primitive; they connect compiled TLS access and the
T01 `context::defer` route without trusting queue ownership.

## Rejected counterexamples

| Mutation | Counterexample and rejecting evidence |
|---|---|
| Restore the post-decrement budget when the downstream operation returns Pending | Starting at nine, `poll_proceed` installs eight but the uncommitted ticket must restore nine. `verify_pending_work_restores_exact_budget` rejects eight; production `budgeting` checks the same rollback. |
| Return Pending on the decrement that changes one to zero | Production permits that operation and records the forced-yield metric; only the next `poll_proceed` is Pending. `verify_budget_conservation_and_forced_pending` checks Ready-at-zero followed by Pending. |
| Return forced Pending without registering the current Waker | `RegisteredBroadcastPoll` consumes `NeedsRegistration`, derives immediate versus scheduler route from context, and exposes the exact wake call or adjacent-deduplicated/queued token before Pending. Production `budgeting` asserts that the pending task is woken. |
| Let `made_progress` restore the old credit | The ticket is changed to the unconstrained sentinel, so its drop preserves the decremented current value. The conservation witness consumes two credits exactly before the forced Pending. |
| Leak the unconstrained sentinel or an inner initial budget across a scope exit | `verify_unconstrained_nesting_and_unwind_restoration` proves LIFO restoration. `nested_unconstrained_restores_exact_budget` checks `127 -> unconstrained -> 128 -> unconstrained -> 127`; `scoped_budget_restores_after_unwind` checks the same ResetGuard path during unwind. |
| Complete the first trace-ready `yield_now` poll, defer twice, or inspect `yielded` before `trace_leaf` | `verify_yield_register_recheck_order` includes a trace-Pending first poll and a trace-Pending re-poll after yielding, proving no state change/defer in either case; the first trace-ready poll alone defers and returns Pending, and the next trace-ready poll alone returns Ready. The public outside-runtime regression observes Pending+woken then Ready. |

## Result and residual

All listed mutations are rejected without adding a trusted premise. The
selected budget/TLS/defer state refinement is body-proved. TLS success,
teardown fallback, exact ResetGuard LIFO restoration, adjacent-only Defer
deduplication, clone/allocation failure preservation, pop-before-Waker call,
and remaining-token release are connected. T01 as a whole remains partial on
`poll_fn`/pin projection, `consume_budget` capture mechanics,
`Unconstrained<F>::poll`, and metric increment. Taskdump-gated consumer
projections, including Pending and panic ordering, are owned by R08 rather than
T01.
Scheduler liveness is not claimed by these safety proofs.
