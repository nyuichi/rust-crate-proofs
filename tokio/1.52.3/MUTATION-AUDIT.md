# SetOnce mutation audit

Date: 2026-07-30

Each mutation was applied only to an independent copy under `/tmp`. The
production worktree was not modified. Commands were restricted to tokio 1.52.3
and the exact SetOnce test that should reject the mutation.

| Mutation | Detecting check | Observed failure |
|---|---|---|
| Acquire load changed to Relaxed | loom `set_once_get_publication_test` | causality violation: concurrent read/write |
| Release store changed to Relaxed | loom `set_once_get_publication_test` | causality violation: concurrent read/write |
| second writer-side initialized check removed | loom `set_once_three_writers_test` | three successful writers instead of one |
| flag clear before owned take removed | `drop_into_inner` integration tests | drop count mismatch followed by invalid double access |
| relaxed readiness recheck removed from `poll_waiter` | loom `set_once_wait_test` | deadlock in the get-to-registration interval |
| both pre-registration `notify_waiters` generation checks removed | loom `set_once_notify_generation_before_registration_test` | first poll returned `Pending` instead of `Ready` |

The production wait loop contains a `cfg(all(loom, test))` yield immediately
after an unsuccessful `get`. It forces loom to explore publication between the
Acquire check and notification-future creation. This instrumentation has no
effect in a non-loom production build.

These results show that the regression suite is sensitive to the principal
safety and lost-wakeup assumptions used by the proof. They are mutation tests,
not a replacement for the Verus contracts or the documented foundational
trusted boundaries.
