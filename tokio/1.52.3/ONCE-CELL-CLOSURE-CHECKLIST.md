# Tokio OnceCell S10 closure checklist

Scope: Tokio 1.52.3 `sync::OnceCell<T>` under the standard `sync` feature.

| Obligation | Evidence | Status |
|---|---|---|
| empty/full construction | `OnceCellModel::{new,new_with}`, production constructor tests | body-proved / tested |
| state representation | Empty = flag false + permit available; Initializing = flag false + permit held; Published = flag true + semaphore closed | composed with S06 and atomic/UnsafeCell adapters |
| `initialized`, `get`, `get_mut` | Acquire observation plus exclusive published-value replacement | body-proved / tested |
| nonblocking `set` | exact empty, initializing, and already-initialized results with rejected payload preservation | body-proved / tested |
| `get_or_init` | immediate read, unique initializer, waiter observation, and exact publication | body-proved / loom-tested |
| cancellation and panic | initializer permit drop restores Empty; a later attempt may initialize | body-proved / tested / loom-tested |
| `get_or_try_init` error | error returns without publication and releases the initializer permit | body-proved / tested / loom-tested |
| recursive initialization | second acquisition waits; cancelling the outer attempt restores Empty | body-proved / deterministic timeout regression |
| `take` / `into_inner` / Drop | exactly one payload transfer or destruction; `take` installs a fresh empty cell | body-proved / tested |
| public traits and errors | Default/From/Clone/Eq/Debug, including initializing and generic-trait panic preservation; Send/Sync/Unpin; SetError variants, classifiers, Display/Debug/Error source | body-proved where OnceCell-owned; compile/runtime-tested |
| cfg integration | ordinary `full,test-util`, standard `sync`, loom AtomicBool/UnsafeCell path | integrated |

`once_cell_refinement` is the thin production-shaped state machine above the
already-closed S06 semaphore contract. The semaphore supplies exclusive
initializer-permit ownership and wake/close behavior. Raw atomic semantics,
UnsafeCell validity, Pin/Poll/Context/Waker execution, arbitrary initializer
Future behavior, and generic Clone/Debug/Drop execution remain the frozen
foundation selected for the Tokio project.

The production AtomicBool now uses Tokio's loom abstraction, matching the
existing loom UnsafeCell. Outside `cfg(loom)` that abstraction is the standard
AtomicBool, so ordinary runtime behavior is unchanged. The exclusive mutable
helpers retain their original non-loom `get_mut` operations; loom uses relaxed
instrumented accesses solely to make that cfg observable to the model checker.

No temporary trusted function, `assume`, `admit`, or new axiom is introduced.
Taskdump-gated trace projection remains owned by R08, and scheduler liveness is
not claimed by this safety/refinement closure.
