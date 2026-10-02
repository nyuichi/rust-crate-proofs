# Tokio AtomicWaker mutation audit

| Mutation | Detecting proof/check | Rejected behavior |
|---|---|---|
| Replace `fetch_or(WAKING)` with a store | `verify_fetch_or_waking_table` | a concurrent REGISTERING bit is erased instead of becoming REGISTERING\|WAKING |
| Clear WAKING before the register owner observes it | `take_waker` and `verify_wake_racing_successful_register` | the racing wake is lost |
| Take the slot when `fetch_or` returns REGISTERING, WAKING, or REGISTERING\|WAKING | `take_waker` postconditions | a caller without the slot lock accesses `UnsafeCell` |
| On successful register+wake race, wake only the replacement | `verify_wake_racing_successful_register` | the displaced old Waker callback is skipped |
| On clone panic, leave state REGISTERING or discard the old slot | both panicking-clone callers | later calls stall or the prior registration is lost |

Rust atomic ordering, UnsafeCell validity, arbitrary Waker clone/drop/wake
execution, and scheduler liveness remain the frozen common foundation. The
Tokio-specific state values, atomic return-value branches, and logical callback
ownership above those adapters are proved.
