# Tokio async Mutex S11 closure checklist

Scope: Tokio 1.52.3 `sync::Mutex<T>` and its four guard representations under
the standard `sync` feature.

| Obligation | Evidence | Status |
|---|---|---|
| one-permit exclusion | `MutexPermitModel::{new,try_lock,try_lock_owned,drop_guard}` composed with closed S06 batch semaphore | body-proved / loom-tested |
| `lock` and `lock_owned` FIFO | `MutexWaitQueue::{enqueue,grant_oldest}` and inherited production intrusive-list refinement | body-proved / tested |
| pending-future cancellation | exact indexed unlink with surviving-order preservation | body-proved / tested / loom-tested |
| `try_lock` and `try_lock_owned` | exact immediate success and mutation-free failure | body-proved / tested |
| borrowed and owned lifetimes | same lock/owner capability; owned bit preserves Arc-backed identity | body-proved above Arc adapter / tested |
| `map` and nested map | original guard consumed without release or duplication; projection alone changes | body-proved / tested |
| every `try_map` success/failure | success returns one mapped capability; failure returns the original live guard unchanged | body-proved / tested |
| four guard destructors | unique capability consumption restores exactly the sole permit | body-proved / tested / loom-tested |
| deref/value mutation | exclusive capability conserves arbitrary `T`; raw UnsafeCell/pointer validity remains frozen | body-proved transfer / tested |
| `get_mut` and `into_inner` | exact value replacement/extraction under exclusive Rust ownership | body-proved / tested |
| blocking wrappers | exact borrowed/owned lock result outside an async runtime | tested; block-on adapter inherited |
| public traits and formatting | Mutex/guard Debug/Display, TryLockError Display/Error, From/Default and existing Send/Sync/Unpin checks | body-proved where state-selecting / compile/runtime-tested |
| mutation sensitivity | premature map release, double Drop release, and wrong-node cancellation witnesses | body-proved |
| cfg integration | ordinary `full,test-util`, public tests, bounded exact loom cases, Verus crate | integrated |

The proof reuses S06's already-closed one-permit semaphore representation and
FIFO intrusive queue. Rust weak-memory atomics, Arc lifetime mechanics,
UnsafeCell and raw-pointer validity, Pin/Poll/Context/Waker mechanics,
arbitrary Drop execution, and scheduler liveness remain the frozen project
foundation. The proof establishes Tokio's protocol above those adapters; it
does not claim to verify their implementations.

No production source change, temporary trusted function, `assume`, `admit`, or
new axiom is introduced. Taskdump-gated tracing remains owned by R08.
