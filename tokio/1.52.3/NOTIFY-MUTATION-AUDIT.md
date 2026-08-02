# Tokio Notify mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Increment generation by 1 instead of 4 | `verify_notify_mutants_rejected` | generation update corrupts the low state bits |
| Consume NOTIFIED before checking the captured broadcast generation | `verify_broadcast_wins_without_consuming_permit` | broadcast steals a stored notify-one permit |
| Leave WAITING after removing the final waiter | `notify_one_locked` contract and Notify tests | state says a nonexistent waiter remains |
| Remove the locked generation recheck | `NotifiedCore` registration proof and integrated loom broadcast race | a concurrent broadcast may be lost |
| Treat locked `data + 4` as total wrapping arithmetic | `verify_locked_waiting_increment_boundary` and production `inc_num_notify_waiters_calls` inspection | the WAITING path panics at call count `usize::MAX / 4` with overflow checks enabled |
| Assume generation inequality across an unbounded observation interval | `verify_notify_word_wrap_preserves_permit` and the full-period snapshot witness | after `usize::MAX / 4 + 1` broadcasts the stored snapshot and current generation are equal again |

Atomic ordering and Waker execution remain the frozen common foundation; the
Tokio-specific word formulas and poll/notify branch ordering are proved below
the terminal generation. Notify remains partially production-refined until a
finite observation window is accepted or the production overflow/ABA policy is
changed.
