# Tokio Notify mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Increment generation by 1 instead of 4 | `verify_notify_mutants_rejected` | generation update corrupts the low state bits |
| Consume NOTIFIED before checking the captured broadcast generation | `verify_broadcast_wins_without_consuming_permit` | broadcast steals a stored notify-one permit |
| Leave WAITING after removing the final waiter | `notify_one_locked` contract and Notify tests | state says a nonexistent waiter remains |
| Remove the locked generation recheck | `NotifiedCore` registration proof and integrated loom broadcast race | a concurrent broadcast may be lost |
| Wrap the terminal broadcast generation | `verify_notify_word_terminal_rejection_preserves_permit`, terminal production regressions, and the exact loom race | the all-build terminal gate rejects before atomic or locked waiter-list mutation and preserves a stored permit |
| Notify WAITING waiters before committing the terminal check | WAITING terminal regression with a retained Waker and `notify_terminal_epoch` | a terminal panic must preserve state, call count, waiter ownership, and safe cancellation |

Atomic ordering and Waker execution remain the frozen common foundation; the
Tokio-specific word formulas, poll/notify branch ordering, and the all-build
terminal generation policy are proved. The check occurs before mutation on the
EMPTY, NOTIFIED, and locked WAITING paths, so generation zero is never reused.
