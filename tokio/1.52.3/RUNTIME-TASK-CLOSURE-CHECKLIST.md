# Tokio R02 runtime task-core closure checklist

Scope: Tokio 1.52.3 `runtime/task/{state,core,raw,harness,waker}.rs` and the
join/abort task-core connection.

| Phase | Obligation | Status |
|---|---|---|
| R02-1 | packed state constants, lifecycle flags, notification, cancellation, join-interest/Waker flags, and reference transitions | body-proved / integrated |
| R02-2 | spawn → schedule → poll → pending/reschedule/complete normal orchestration | body-proved / integrated on current- and multi-thread schedulers |
| R02-3 | abort/cancel and exact JoinHandle result ownership | pending |
| R02-4 | raw vtable/layout, final deallocation, future/output/waker panic cleanup | pending |

The R02-1 proof uses the exact six low state bits and `REF_ONE == 64`; the
initial state is exactly 204 with three references. Atomic modification order
and raw allocation remain frozen foundations. Reference increment is bounded
by finite simultaneously live references, matching production's abort-on-
overflow policy rather than introducing an unbounded mathematical counter.

No production logic was changed. Production-source changes are test-only.
