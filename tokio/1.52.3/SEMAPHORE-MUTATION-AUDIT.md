# Tokio Semaphore mutation audit

The exact representation proof rejects three independent defect classes:

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Remove `PERMIT_SHIFT` when encoding permits | `verify_semaphore_mutants_rejected` | one permit decodes as zero |
| Omit `curr_bits & CLOSED` in `forget_permits` | `verify_semaphore_mutants_rejected` | a closed semaphore reopens |
| Change waiter selection from `last/pop_back` to `first/pop_front` | `verify_push_front_pop_back_is_fifo` and semaphore FIFO tests | newest waiter bypasses oldest |

The integrated ordinary and loom Semaphore suites additionally exercise exact
acquire, release, close, cancellation, overflow, and multi-permit behavior.
