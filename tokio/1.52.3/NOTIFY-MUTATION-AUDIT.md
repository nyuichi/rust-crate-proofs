# Tokio Notify mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Increment generation by 1 instead of 4 | `verify_notify_mutants_rejected` | generation update corrupts the low state bits |
| Consume NOTIFIED before checking the captured broadcast generation | `verify_broadcast_wins_without_consuming_permit` | broadcast steals a stored notify-one permit |
| Leave WAITING after removing the final waiter | `notify_one_locked` contract and Notify tests | state says a nonexistent waiter remains |
| Remove the locked generation recheck | `NotifiedCore` registration proof and integrated loom broadcast race | a concurrent broadcast may be lost |

Atomic ordering and Waker execution remain the frozen common foundation; the
Tokio-specific word formulas and poll/notify branch ordering are proved.
