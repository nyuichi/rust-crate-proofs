# Tokio U01 utility-collection closure checklist

Scope: Tokio 1.52.3 `util::linked_list`, `util::wake_list`, and
`util::idle_notified_set` under their shipped internal feature configurations.

| Obligation | Evidence | Status |
|---|---|---|
| intrusive membership and ownership | detached/list transfer for push, front/back pop, exact indexed remove | body-proved / tested |
| intrusive order | push-front, FIFO pop-back, guarded-list pop-back, and last-element behavior | body-proved / tested |
| list-to-list transfer | repeated pop-back/push-front preserves the source order and prepends it to the destination | body-proved / connected |
| drain filter | keep/remove advance exactly once; predicate panic leaves the current and remaining nodes linked | body-proved / tested |
| traversal snapshot | callbacks receive a fixed snapshot and cannot remove values while the collection is mutably borrowed | body-proved / tested |
| WakeList capacity | exactly 32 initialized prefix slots; push is admitted only below capacity | body-proved / tested |
| WakeList wake/drop | ownership transfers before wake; panic drops the exact remaining suffix; later Drop cannot repeat it | body-proved / tested |
| idle/notified transitions | unique insertion, idempotent wake, oldest-notified pop, and exact list movement | body-proved / tested |
| registered waker | `pop_notified` replaces registration even on empty notified list; `try_pop_notified` preserves it; successful wake consumes it | body-proved / tested |
| value and length ownership | insert/remove transfer the exact value and change length by one | body-proved / tested |
| for_each mutation | idle and notified values are each visited once from the captured snapshot | body-proved / tested |
| drain/drop unwind | both lists empty and all values are processed after the first callback panic | body-proved / tested |
| finite length arithmetic | successful simultaneous allocation keeps the live entry count strictly below `usize::MAX` | reviewed frozen allocation premise |

Raw intrusive-pointer validity, `Arc`/`Mutex`/`UnsafeCell` mechanics, allocation,
and arbitrary Waker/Drop/callback execution remain the frozen foundation.
Tokio-owned membership, ordering, extraction, cleanup, and mutation effects are
closed above it.

No production logic, temporary trusted function, `external_body`, `assume`,
`admit`, or new axiom was introduced. Changes in production modules are
test-only.
