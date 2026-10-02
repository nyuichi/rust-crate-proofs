# Tokio oneshot S02 closure checklist

Scope: Tokio 1.52.3 `sync::oneshot`, including its private use under `rt` and
Windows `process`.

| Obligation | Evidence | Status |
|---|---|---|
| exact four-bit state and weak-CAS outcomes | `oneshot_state`, `oneshot_atomic`, `oneshot_refinement` | body-proved / loom-tested |
| shared payload cell permission | generic `vstd_ext::shared_pcell` separates `PCell` from its linear permission without a new axiom | body-proved |
| Sender → staged → Receiver handoff | `oneshot_shared_refinement` preserves exact `T`, including close-win take-back | body-proved / loom-tested |
| RX/TX Waker slot transitions | Vacant/Writing/Published/Detached/Released capability excludes partial reads and proves replacement/terminal cleanup | body-proved / loom-tested |
| receiver poll/recheck | initial load, registration, publication recheck, stale-Waker replacement, Pending/Ready | body-proved / loom-tested |
| sender `poll_closed` | registration/recheck, replacement, close wake and final cleanup | body-proved / loom-tested |
| cooperative and trace gates | trace precedes budget; exhausted/trace Pending cannot poll core; Pending restores budget; Ready commits progress | body-proved / cfg-tested |
| Future terminal surface | Ready removes the outer inner handle; every later poll takes the exact panic branch | body-proved / runtime-tested |
| `blocking_recv` | instantiates the proved runtime BlockingRegion/TLS/CachedParkThread path for Value and Closed after arbitrary finite Pending polls | body-proved / both cfgs tested |
| public errors and traits | RecvError/TryRecvError Clone/Eq/Debug/Display/Error source plus endpoint Send/Sync/Unpin | body-proved where Tokio selects state / runtime and compile-tested |
| payload and Waker unwind | payload Drop once on receiver cleanup/rejected send; tracing construction/drop path | body-proved ownership / runtime-tested |
| endpoint/Arc/final Inner cleanup | local send Arc, two endpoints, task slots, and value are released exactly once | body-proved / loom-tested |
| cfg integration | `full`, `sync`-only, `rt`-only, unstable tracing, and Windows process-only cross-check | integrated |
| mutation sensitivity | weakened publication, removed recheck, and stale Waker retention are rejected | three mutations recorded |

The remaining boundaries are only the repository-wide frozen foundation:
Rust weak-memory atomic execution; Arc/UnsafeCell/raw pointer and allocation
mechanics; Pin/Poll/Context/Waker mechanics; arbitrary user Clone/Drop and
formatting execution; and scheduler/parking liveness. The generic
`SharedPCell` extension is proved from vstd `PCell` and introduces no
`external_body`, `assume`, `admit`, or axiom.

No production source was changed for this closure. Taskdump callback mechanics
remain owned by R08; S02 proves that a trace Pending or unwind occurs before
oneshot state, payload, and cooperative-budget mutation.
