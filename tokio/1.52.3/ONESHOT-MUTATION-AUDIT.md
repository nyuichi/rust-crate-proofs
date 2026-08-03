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

## Formal refinement witnesses

The following are proof-level counterfactuals, not additional disposable-copy
production mutations. Each is rejected by an executable caller or invariant in
`verification/src/oneshot_refinement.rs`:

| Counterfactual | Rejected property |
|---|---|
| introduce a fifth state bit | exact snapshots can encode only RX task, VALUE_SENT, CLOSED, and TX task |
| clear CLOSED or VALUE_SENT between a failed weak CAS observation and the next actual state | lifecycle bits are monotonic across CAS retries; task bits alone may be unregistered |
| enter Sender completion with VALUE_SENT already set, or let interference set it | the unique active-Sender CAS contract requires VALUE_SENT clear and unchanged across retry interference |
| treat a weak-CAS failure as completion | failed and spurious CAS outcomes return Retry without publication |
| set a task bit without an initialized Waker, or retain an RX Waker after close wins | raw task bits are derived exactly from Waker ownership and close clears the losing RX task |
| let `try_recv` add CLOSED | successful/closed receive preserves the pre-existing CLOSED bit exactly |
| let close win after Sender stores but before completion without taking the payload back | the Staged race witness returns the exact `T`, leaves the slot empty, and never sets VALUE_SENT |
| let Receiver drop after the staged store release its `Inner` early or lose the staged value | the drop-interference witness retains the local Arc, returns the exact `T` to Sender, then performs final cleanup |
| clear the outer Sender option without retaining its local Arc during the staged interval | strong-count conservation and the unique Staged capability fail |
| discard TX_TASK_SET/Waker when Sender is consumed, or retain it past terminal Receiver cleanup | the TX-Waker witness preserves it across send and terminal close, then releases it with the final Receiver Arc |
| make terminal `try_recv` or `close` consume resources again | repeated terminal calls are state-preserving; `try_recv` remains Closed and `close` remains a no-op |
| release `Inner` before both endpoints and the local send Arc are gone, or clean it twice | final-drop equivalence and Released capability uniqueness fail |
