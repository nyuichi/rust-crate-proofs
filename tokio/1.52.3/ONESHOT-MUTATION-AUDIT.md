# Tokio oneshot mutation audit

Date: 2026-08-01

These mutations were applied one at a time to a disposable copy of Tokio
1.52.3. None is present in the verification target. Each command selected the
smallest existing loom test exercising the damaged production path.

| Mutation | Expected defect | Detecting test | Result |
|---|---|---|---|
| weaken `State::set_complete` CAS success ordering from `AcqRel` to `Relaxed` | the receiver can observe `VALUE_SENT` without the payload publication edge | `sync::tests::loom_oneshot::smoke` | rejected: loom reported concurrent read/write causality violation |
| remove the completion recheck after installing `RX_TASK_SET` | send can occur between the initial state read and Waker registration, losing the wake-up | `sync::tests::loom_oneshot::smoke` | rejected: loom reported deadlock |
| retain an obsolete receiver Waker instead of replacing it when `will_wake` is false | the sender can wake the old task and leave the current receiver pending | `sync::tests::loom_oneshot::changing_rx_task` | rejected: loom reported deadlock |

The audit demonstrates sensitivity to the three concurrency edges represented
in the Verus models: publication ordering, register-then-recheck, and Waker
identity replacement. It does not turn loom into a proof, and it does not
validate executions outside loom's explored model.
