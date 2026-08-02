# Tokio 1.52.3 verification ledger

This is the coverage manifest for the effort to verify the whole Tokio 1.52.3
crate. It is deliberately not a line-of-code counter. The denominator is the
53 semantic API/implementation paths in the manifest below. A path is a unit
only when it has its own state, ownership, ordering, cancellation, arithmetic,
or platform contract; nearby forwarding functions are grouped with that path.

The manifest covers the modules exposed by `src/lib.rs` under the published
feature set, plus internal components whose correctness is required by those
public modules. Code supplied by dependencies, including the implementation of
`tokio-macros`, is not part of the denominator. Platform variants in one API
family share a row, but the row cannot close until every supported variant has
either been verified or declared as a reviewed environment adapter.

## Status and counting rules

- **P (protocol)**: Tokio-specific abstract transitions have executable body
  proofs. This does not imply a connection to the production representation.
- **R (refined)**: production constants, representation arithmetic, and branch
  structure have been related to the protocol. `R(partial)` names the missing
  connection explicitly and does not count as fully refined.
- **I (integrated)**: the target is exercised by the Tokio-1.52.3-scoped
  `verify-all.bash` run. Tests or loom alone never imply P or R.
- **L (legacy scoped)**: the row met an earlier, narrower protocol-completion
  definition. L records real proof evidence but does not satisfy the current
  production-refinement closure condition and does not count as currently
  closed.
- **C (current closed)**: every Tokio-specific obligation in the row's recorded
  closure condition, including its production refinement, is proved above the
  frozen trusted foundation. This is not a proof of that foundation.
- **U**: no component-specific Verus proof is currently claimed.

A row counts as current progress only at C. The two scores are intentionally
separate:

- **legacy protocol-scope score: 9/53 (17%) L-or-C**, all integrated;
- **current full-closure score: 8/53 (15%) C**, all fully production-refined
  and integrated.

Oneshot is recorded separately as partial refinement. The legacy numerator is
historical evidence, not an alternative whole-Tokio completion percentage.
These percentages describe this manifest, not effort, risk, source coverage,
or proof-strength-weighted progress:
the runtime, scheduler, I/O, and OS rows are substantially harder than most
closed sync rows.

The frozen trusted foundation is: Rust weak-memory atomic semantics;
Mutex/RwLock/Arc/UnsafeCell implementations; Pin/Poll/Context/Waker mechanics;
raw-pointer validity and allocation; arbitrary user Clone/Drop execution; and
scheduler liveness. A row may use only the relevant subset listed in its
adapter column. OS, clock, and selector contracts are not frozen yet; rows that
need them remain U until those contracts are reviewed. Arbitrary user code is
not a blanket trusted boundary: only the language/library execution semantics
of Clone and Drop are frozen. Predicate, initializer, and update-closure
outcomes are specification inputs whose effects on Tokio state still have to
be proved.

The repository-local `vstd_ext::thread_local` adds one narrow generic
TLS/Cell correspondence to that foundation. Tokio-specific budget,
scheduler-context, blocking-region, scoped binding, and current-parker mappings
are body-proved above it; they are not separate trusted adapters. Numeric key
tags identify owned instances/fields inside one composed model and do not claim
global TLS registry freshness.

## Component/API-path manifest

| ID | Area and production paths | Public or downstream surface | Feature / cfg | Status and evidence | Trusted adapters used | Residual gap / next closure condition |
|---|---|---|---|---|---|---|
| S01 | SetOnce: `src/sync/set_once*.rs` | `SetOnce`, set/get/wait/take/traits | `sync` | **C / R / I**; `verification/src/{lib,published_cell,release_acquire,writer_lease,wait_protocol,trait_views,...}.rs`, `PROVENANCE.md`; exact publication, wait/cancel, take/drop/unwind, generic Clone/PartialEq, Debug state selection, SetOnceError Display/Debug/source orchestration, and public trait bounds are proved or compile/runtime-tested as recorded | atomics, loom Mutex/UnsafeCell, poll/Waker, raw links, user trait/Drop and generic formatting execution | Closed under the frozen scoped-completion definition. Generic formatter behavior remains its standard trait boundary, while SetOnce-owned state selection and preservation are body-proved. |
| S02 | oneshot: `src/sync/oneshot.rs`, `oneshot_value.rs` | channel, Sender/Receiver, poll/close/drop | `sync`; internal under `rt` and Windows `process` | **L / R(partial) / I**; `oneshot_{state,atomic,value,poll,closed,drop,refinement}.rs`, `ONESHOT-MUTATION-AUDIT.md`; exact reachable four-bit/CAS, staged store-close/drop-complete races, outer endpoint/local-Arc and physical proof-slot capability refinement, repeated terminal try/close, TX-Waker terminal cleanup; `rt`-only and `sync`-only compile gates | raw atomic/UnsafeCell semantics, Arc, Pin/Poll/Context/Waker, user Drop execution | Connecting the state capability to the PCell permission inside Tokio's shared-`&self` UnsafeCell wrapper is an unclosed Tokio-specific representation refinement, not a frozen adapter. Prove task-bit/Waker transition-in-progress states with a concurrent atomic invariant, `coop::poll_proceed`/`trace_leaf`, `blocking_recv` through the `rt` runtime-context/BlockingRegion and non-`rt` CachedParkThread runtime/park paths, Future Ready-repoll panic, unstable tracing-panic preservation, and public error/Debug/Display/Error/Clone paths; run the Windows process-only cross-build before C/R. |
| S03 | watch: `src/sync/watch.rs` | channel, send/update, borrow/changed/wait_for, endpoint lifecycle | `sync` | **C / R / I**; `watch_{protocol,completion,refinement,orchestration,endpoint_refinement,selector_refinement,surface,terminal_refinement}.rs`, `WATCH-{MUTATION,CLOSURE-CHECKLIST}.md`; exact terminal update/Receiver-credit policies, Arc-first Sender clone, predicate panic preservation, closed registration/reopen races, circular/RNG eight-shard selection, cooperative/public surface, and boundary Loom races proved | atomics, RwLock, Arc, Waker, Clone/Drop execution; T01 cooperative contract | Closed for standard `sync` cfg above the frozen foundation. Update-closure/predicate outcomes remain specification inputs. Taskdump-gated trace projection is owned by R08. Notify terminal behavior is supplied by S07. |
| S04 | broadcast: `src/sync/broadcast.rs` | channel, send/recv/try_recv/blocking_recv, lag, subscribe, weak/endpoints | `sync`; blocking path with and without `rt` | **C / R / I**; `broadcast_{protocol,completion,refinement,physical_refinement,endpoint_refinement,orchestration,surface}.rs`, `coop_tls_refinement.rs`, `defer_refinement.rs`, `blocking_recv_refinement.rs`, and `BROADCAST-{MUTATION,WRAP}-AUDIT.md`; terminal exhaustion, generic ring generations/slot-rem conservation, waiter/lock ordering, endpoint lifecycle, public error/trait mapping, unwind cleanup, cooperative recv/defer, shared-CONTEXT blocking rejection/fallback, current-parker routing, arbitrary finite Pending/park repetition, and terminal blocking results are proved and regression-tested | atomics, Mutex, raw waiter links, Waker/condvar mechanics, Clone/Drop execution, scheduler liveness; generic TLS/Cell correspondence; narrow `FiniteExecutionResources` | Closed under the recorded S04 scope. The taskdump-gated `trace_leaf` projection is owned by R08, not an S04 residual. Terminal exhaustion is not channel close: an empty terminal Receiver may remain Pending until Sender drop. |
| S05 | mpsc: `src/sync/mpsc/{chan,list,block,bounded,unbounded}.rs` | bounded/unbounded send/reserve/recv/close, permits, weak/endpoints | `sync`; timeout forms also `time` | **L / R(partial) / I**; `mpsc_{protocol,completion,refinement,endpoint_refinement,try_recv_refinement,queue_refinement,recv_compiled_refinement,try_recv_raw_refinement,orchestration,surface}.rs`, `MPSC-{MUTATION,WRAP,RECV-MANY-UNWIND,CLOSURE-CHECKLIST}.md`; logical raw-list classification from claims/ready bits/close, linear payload-slot ownership, traversal/release leases, exact recv and recv-many composition without a spare-capacity premise, finite-resource count-before-Arc gaps, last-strong raw close/wake composition, arbitrary finite Busy-prefix rank, full gate snapshots, and cooperative/blocking/public result mappings are body-proved and integrated; `Block::grow` uses explicit wrapping and has a final-generation regression | S06 semaphore and S09 AtomicWaker contracts; atomics, Arc, UnsafeCell, Pin/Poll/Context/Waker, raw allocation/pointer validity, Vec allocation, arbitrary Drop execution, park/Waker mechanics, scheduler liveness; mpsc-scoped `MpscFiniteExecutionResources` | Two production index-boundary decisions remain. `Block::has_value` still uses `start_index + BLOCK_CAP`, which can panic in debug and misclassify the final aligned block in release. `Rx::reclaim_blocks` still uses ordinary `required_index > self.index`, which does not implement the proved logical traversal lease across wrap. Exact non-mutating fixtures record both behaviors. The logical queue keeps an absolute generation while production stores erased `usize`; therefore S05 remains partial until both comparisons receive reviewed modular policies and connected regressions. |
| S06 | semaphore: `src/sync/{batch_semaphore,semaphore}.rs` | Semaphore, borrowed/owned permits, acquire/close/add/forget | `sync`; internal Mutex support under `fs` | **C / R / I**; `semaphore_{protocol,refinement}.rs` | atomics, Mutex, Arc, raw waiter links, Waker | Closed under frozen scope. |
| S07 | Notify: `src/sync/notify.rs` | notify_one/last/waiters, Notified/OwnedNotified enable/poll/drop | `sync`; internal under `rt`, `signal`, `process` | **C / R / I**; `notified_core.rs`, `notify_{protocol,refinement}.rs`, `NOTIFY-MUTATION-AUDIT.md`; all-build terminal generation rejection occurs before EMPTY/NOTIFIED/WAITING mutation and preserves waiter ownership | atomics, Mutex, raw links, Pin/Poll/Waker | Closed under frozen scope. The terminal call panics rather than reusing generation zero. |
| S08 | Barrier: `src/sync/barrier.rs` | new/wait, reusable cohorts and leader result | `sync` | **C / R / I**; `barrier_{protocol,refinement,orchestration}.rs`, `BARRIER-{MUTATION-AUDIT,CLOSURE-CHECKLIST}.md`; cohort-wide update/Receiver credit admission before arrival commit, exact Barrier/watch generation relation, mutation-free terminal rejection, cancellation, repeated cohorts, and leader-result behavioral mapping are body-proved; Debug/Clone runtime behavior and public cfg/auto traits are tested | Mutex, S03 watch storage/wake, scheduler liveness | Closed for standard `sync` above the frozen foundation. The S03 terminal gate prevents generation reuse; taskdump-gated trace projection is owned by R08. |
| S09 | AtomicWaker: `src/sync/task/atomic_waker.rs` | internal wake/register primitive used by channels/runtime | `sync` and internal consumers | **C / R / I**; `atomic_waker_{protocol,refinement}.rs` | atomics, UnsafeCell, Waker execution | Closed under frozen scope. |
| S10 | OnceCell: `src/sync/once_cell.rs` | new/get/set/get_or_init/get_or_try_init/take/traits | `sync` | **C / R / I**; `once_cell_refinement.rs`, `ONCE-CELL-CLOSURE-CHECKLIST.md`; Empty/Initializing/Published composition, unique initializer, waiter observation, cancellation/panic/error retry, recursive wait, set errors, mutable/take/drop, and public trait/error surfaces proved and tested | S06 semaphore; atomics, UnsafeCell, Pin/Poll/Context/Waker, arbitrary Future/Clone/Debug/Drop execution, scheduler liveness | Closed for standard `sync` above the frozen foundation. Tokio's loom AtomicBool path now matches its loom UnsafeCell; ordinary builds still use the standard AtomicBool. Taskdump-gated trace projection is owned by R08. |
| S11 | async Mutex: `src/sync/mutex.rs` | lock/try_lock, owned and mapped guards, get_mut/into_inner | `sync`; internal under `fs` | **U** | batch semaphore, Arc, UnsafeCell, Drop | Refine one-permit exclusion to guard mapping/owned lifetimes, cancellation FIFO, unlock-on-drop, and value ownership. |
| S12 | async RwLock: `src/sync/rwlock.rs`, `src/sync/rwlock/*.rs` | read/write/try/owned guards, map/downgrade/get_mut/into_inner | `sync` | **U** | batch semaphore, Arc, UnsafeCell, Drop | Prove reader/writer permit accounting, writer exclusion/fairness, downgrade atomicity, all mapped/owned guard Drop paths. |
| F01 | future combinators: `src/future/{maybe_done,try_join}.rs`, `src/macros/try_join.rs` | `try_join!` orchestration and internal MaybeDone | `macros` for macro; core future internals | **U** | Pin/Poll/Context, arbitrary Future/Drop | Prove state transitions, no repoll after completion, early-error cancellation/drop, output ordering, and macro arities. |
| T01 | cooperative yield/budget: `src/task/{yield_now,coop/*}` | yield_now, poll_proceed/consume_budget/unconstrained | `rt`; `test-util` controls | **P / R(partial) / I**; `verification/src/{coop_refinement,coop_tls_refinement,defer_refinement,vstd_ext/thread_local}.rs`, `COOP-MUTATION-AUDIT.md`; exact budget/TLS success and teardown paths, callback cardinality, restore/reset guards, forced-Pending registration routing, adjacent Defer deduplication, Waker-token ownership and panic cleanup, and trace-before-yielded ordering are proved | Pin/Poll/Context/Waker mechanics, arbitrary Drop execution, scheduler liveness; generic TLS/Cell correspondence | The budget/TLS/defer subset used by standard and blocking broadcast recv is connected through body-proved Tokio mappings. T01 remains partial on `poll_fn` capture/pinning for `consume_budget` and `yield_now`, `Unconstrained<F>::poll`, and metric increment. Taskdump-gated trace projections at T01 consumers are owned by R08. Liveness remains a frozen adapter, not a theorem of this safety slice. |
| T02 | spawn/join/abort surface: `src/task/{spawn,builder}.rs`, `src/runtime/task/{join,abort,error,id}.rs` | spawn, JoinHandle, AbortHandle, JoinError, ids | `rt`; builder unstable | **U** | runtime task core, panic payload/Drop, Waker | Close after R02 task-state refinement plus exact join/abort/panic/cancel result ownership. |
| T03 | JoinSet: `src/task/join_set.rs` | spawn/join_next/abort/shutdown/detach_all | `rt` | **U** | task core, owned linked list, Waker | Prove membership and output ownership, wake/recheck, cancellation/drop, and task-id consistency. |
| T04 | LocalSet/local spawn: `src/task/local.rs` | LocalSet, spawn_local, run_until, enter | `rt` | **U** | thread identity/TLS, scheduler, Waker | Prove owner-thread confinement, queue ownership, wake routing, enter nesting, and shutdown/drop. |
| T05 | task-local storage: `src/task/task_local.rs` | task_local!, LocalKey::scope/sync_scope/get/try_with | `rt`; macro surface core | **U** | TLS/context, Future/Drop/unwind | Prove scoped replacement/restoration including panic/cancel and nested values. |
| R01 | runtime construction/lifecycle: `src/runtime/{builder,runtime,handle,context,config}.rs`, `local_runtime/{runtime,options}.rs` | Builder, Runtime, LocalRuntime, Handle, enter/block_on/shutdown | `rt`; multi-thread options `rt-multi-thread`; local runtime unstable | **U** | OS threads/parking/time/I/O contracts | Define lifecycle state and prove construction rollback, enter context, shutdown ownership, and driver/scheduler composition. |
| R02 | task core/state: `src/runtime/task/{state,core,raw,harness,waker}.rs` | downstream basis of spawned tasks | `rt` | **U** | atomics, raw allocation/vtable, Pin/Poll/Waker, panic/Drop | Prove exact state bits/refcount, schedule/poll/cancel/complete/join linearization, deallocation, and unwind paths. |
| R03 | current-thread scheduler: `src/runtime/scheduler/current_thread/*` | current-thread Runtime execution | `rt` | **U** | R02, parking/clock/I/O, liveness | Prove local/remote queue ownership, tick/poll ordering, wake injection, block_on and shutdown drain. |
| R04 | multi-thread local queues: `src/runtime/scheduler/multi_thread/{queue,overflow}.rs` | internal work queue / stealing | `rt-multi-thread` | **U** | atomics, raw task ownership, liveness | Prove ring indices, push/pop/steal/overflow races, wrap, and exact task conservation. |
| R05 | multi-thread workers: `src/runtime/scheduler/multi_thread/{worker,idle,park,handle}.rs` | multi-thread Runtime scheduling | `rt-multi-thread` | **U** | R02/R04, threads, parking, randomness, liveness | Compose task ownership across workers/inject/steal, search/park transitions, shutdown, and blocking handoff. |
| R06 | inject/defer support: `src/runtime/scheduler/{inject,defer,lock}.rs` | remote scheduling and deferred wake/free | `rt` / `rt-multi-thread` variants | **U**; the T01-used `defer.rs` subset is **P / R / I** via `defer_refinement.rs` and focused tests | Mutex/atomics/raw task links; Waker execution and scheduler liveness for the selected Defer subset | Defer adjacent dedup, exact push, pop-before-wake, panic preservation, and Drop release are proved. Inject/lock closed-queue semantics, batch pop, and remote raw-task ownership remain U. |
| R07 | blocking pool: `src/runtime/blocking/*`, `src/blocking.rs`, `src/task/blocking.rs` | spawn_blocking, block_in_place | `rt`; block_in_place `rt-multi-thread` | **U** | OS threads/condvars, task core, liveness | Prove queue/task ownership, worker creation/retirement, cancel/shutdown, and scheduler handoff. |
| R08 | metrics/hooks/dump: `src/runtime/{metrics,task_hooks,dump}.rs`, scheduler metrics, and taskdump-gated consumer projections | RuntimeMetrics, unstable hooks/taskdump | `rt`; `tokio_unstable`; `taskdump` | **U** | counters, backtrace/platform stack inspection | Prove metric invariants and hook ordering; isolate taskdump platform contracts and verify snapshot ownership; connect `trace_leaf` callbacks at consumers including broadcast and T01, preserving Pending and panic ordering. |
| R09 | driver orchestration: `src/runtime/driver.rs`, `src/runtime/driver/*`, `src/runtime/park.rs` | runtime integration of I/O, time, signal, park | `rt` plus enabled drivers | **U** | selector/clock/OS parking contracts | Specify driver enablement and prove park/unpark, turn ordering, shutdown, and error propagation across subdrivers. |
| TM01 | Instant/clock control: `src/time/{instant,clock}.rs` | Instant, pause/resume/advance | `time`; controls `test-util` | **U** | monotonic clock and TLS runtime context | Review clock contract; prove checked arithmetic, paused-time state, advance wake ordering, and context errors. |
| TM02 | Sleep/Timeout: `src/time/{sleep,timeout}.rs` | sleep(_until), timeout(_at), reset/deadline | `time` | **U** | timer driver, Pin/Poll/Waker, arbitrary Future | Prove registration/reset/cancel, deadline races, biased timeout result ownership, and no stale wake. |
| TM03 | Interval: `src/time/interval.rs` | interval(_at), tick/reset, missed-tick strategies | `time` | **U** | TM02, clock arithmetic | Prove burst/delay/skip arithmetic including overflow, reset semantics, and cancellation. |
| TM04 | timer core: `src/runtime/time/{entry,wheel,source}.rs`, `time_alt/*` | internal timer registration/wheel/cancellation | `time`; alternate cfg paths | **U** | clock, atomics/locks, raw links, Waker | Prove wheel level/slot arithmetic, entry ownership, elapsed/cancel races, wake queues, rollover, and shutdown for each compiled implementation. |
| I01 | async traits and ReadBuf: `src/io/{async_read,async_write,async_buf_read,async_seek,read_buf}.rs` | AsyncRead/Write/BufRead/Seek, ReadBuf | core | **U** | Pin/Poll, initialized-memory/raw-pointer validity | Prove ReadBuf initialized/filled bounds and trait helper contracts; unsafe initialization remains a reviewed memory adapter. |
| I02 | readiness values: `src/io/{ready,interest}.rs` | Ready, Interest | `net`/runtime I/O consumers | **U** | none beyond integer semantics | Prove bit-set algebra, platform mapping, tick masking, and all combination/iterator operations. |
| I03 | I/O utility futures: `src/io/util/*`, `split.rs`, `join.rs` | extension traits, copy/read/write/lines/buffers/split | `io-util` | **U** | I01 traits, Pin/Poll, user I/O behavior, allocation | Prove each future state machine, buffer bounds/ownership, partial progress, errors, cancellation, and split lock/wake protocol. |
| I04 | standard I/O wrappers: `src/io/{stdin,stdout,stderr,stdio_common,blocking}.rs` | stdin/stdout/stderr async wrappers | `io-std` | **U** | blocking threads and OS stdio semantics | Prove request/buffer ownership, cancellation, flush/shutdown, and shared-handle serialization above OS contracts. |
| I05 | readiness registration: `src/io/{async_fd,poll_evented}.rs`, `src/runtime/io/*` | AsyncFd and downstream network registration | `net`/`io-uring` relevant cfgs | **U** | mio selector/OS handle semantics, atomics, Waker | Define selector contract; prove ScheduledIo readiness/ticks, registration lifecycle, clear/try_io races, deregistration and shutdown. |
| I06 | io-uring path: `src/io/uring/*`, runtime io driver uring | internal async open/read/write operations | `io-uring`; Linux; unstable gate | **U** | io_uring/kernel, DMA/buffer/raw-pointer validity | Review kernel contract; prove operation/buffer ownership, submit/complete/cancel/drop and fallback/error paths. |
| N01 | address resolution: `src/net/{addr,lookup_host}.rs` | ToSocketAddrs, lookup_host | `net` | **U** | DNS/OS resolver and blocking pool | Prove iterator/result ownership and async blocking orchestration; declare resolver semantics environmental. |
| N02 | TCP: `src/net/tcp/*` | TcpListener/Stream/Socket, split halves | `net`; platform cfgs | **U** | socket syscalls + I05 selector contract | Prove socket/registration ownership, accept/connect/read/write readiness loops, split reunite/drop, and platform options. |
| N03 | UDP: `src/net/udp.rs` | UdpSocket send/recv/connect/readiness | `net`; platform cfgs | **U** | socket syscalls + I05 selector contract | Prove datagram buffer/address ownership, readiness retry, connected/unconnected states, options and errors. |
| N04 | Unix and Windows IPC: `src/net/{unix,windows}/*` | Unix streams/listeners/datagrams/pipes; named pipes | `net`; Unix/Windows cfgs | **U** | platform socket/pipe/credential semantics | Specify both platform families and prove creation/connect/accept/split/read/write/readiness/drop paths. |
| FS01 | filesystem one-shot operations: `src/fs/{read,write,copy,metadata,...}.rs` | path-level async fs functions | `fs`; platform cfgs | **U** | std/OS filesystem and blocking-pool contracts | Prove closure/result/path ownership and cancellation semantics; declare filesystem effects environmental. |
| FS02 | stateful filesystem objects: `src/fs/{file,open_options,read_dir,dir_builder}.rs` | File/OpenOptions/ReadDir/DirBuilder | `fs`; `io-uring` variants | **U** | filesystem, blocking pool/io-uring, Mutex/semaphore | Prove cursor/buffer ownership, in-flight operation serialization, seek/read/write/flush/drop, directory iteration, and platform branches. |
| P01 | child process API: `src/process/{mod,kill}.rs` | Command, Child, stdio, wait/kill | `process`; platform cfgs | **U** | process/syscall, pipes/signals, I/O registration | Prove handle/stdio ownership, spawn rollback, wait/kill races, kill-on-drop, and output collection above OS contracts. |
| P02 | process reaping: `src/process/{unix,windows}.rs`, `unix/{reap,orphan,pidfd_reaper}.rs` | internal child completion/reaping | `process`; Unix/Windows variants | **U** | OS process/signal/pidfd contracts, Waker | Prove registration, orphan transfer, exactly-once reap, PID reuse boundary, cancellation and shutdown for each backend. |
| SG01 | signal registry: `src/signal/{registry,reusable_box}.rs` | downstream signal subscriptions | `signal`; internal process/runtime use | **U** | atomics/Mutex, Waker, allocation | Prove listener-slot ownership, generation/notification, registration/drop, reusable future replacement, and no lost event. |
| SG02 | platform signals: `src/signal/{ctrl_c,unix,windows}.rs` | ctrl_c and Unix signal streams | `signal`; OS cfgs | **U** | OS signal handler/console semantics | Specify async-signal-safe bridge; prove initialization, event fanout/coalescing, stream poll/drop, and platform errors. |
| M01 | join macro: `src/macros/join.rs`, `src/future/maybe_done.rs` | `join!` | `macros` | **U** | Pin/Poll, arbitrary futures | Prove output order, fair polling rotation, completion/drop and supported arities; share F01 kernel. |
| M02 | select macro runtime support: `src/macros/select.rs` | `select!` | `macros` | **U** | macro expansion, Pin/Poll, user guards/patterns/futures | Verify branch enablement, fair/random and biased order, precondition evaluation, disabled patterns, cancellation/drop and else/panic behavior. |
| M03 | attribute macro integration: reexports in `src/lib.rs`; implementation dependency `tokio-macros` | `#[tokio::main]`, `#[tokio::test]` | `macros`; runtime flavor cfgs | **U** | external proc-macro implementation, Runtime Builder | Denominator covers Tokio-side expansion contract only: validate generated Runtime configuration, argument/result/panic behavior, and feature diagnostics; dependency implementation remains out of scope. |
| U01 | intrusive/wake collections: `src/util/{linked_list,wake_list,idle_notified_set}.rs` | scheduler/sync/time downstream foundation | internal cfgs | **U** | raw pointer validity, Waker | Prove membership/ownership, remove/drain/drop, capacity boundaries, wake extraction and mutation sensitivity; then reuse in dependent refinements. |
| U02 | atomic/cell/random helpers: `src/util/{atomic_cell,rc_cell,bit,rand}.rs`, `src/loom/*` | runtime/sync downstream foundation | internal platform/loom cfgs | **U** | Rust atomics/UnsafeCell; entropy source | Prove Tokio-specific encodings and ownership APIs; retain raw atomic/cell and entropy semantics as adapters. |
| U03 | sharded/auxiliary ownership: `src/util/{sharded_list,try_lock,wake,sync_wrapper}.rs` | task lists, scheduler and utility downstream | internal cfgs | **U** | Mutex/atomics/raw pointers/Waker | Prove shard selection, list ownership, lock-state protocol, wake-by-ref ownership, Send/Sync wrapper justification and all drop paths. |

## Source coverage index

This index makes the manifest's grouping claim auditable for source paths that
are not all spelled out in the main table. It does not add a second coverage
denominator.

| Source family or otherwise implicit path | Owning manifest IDs |
|---|---|
| `src/sync/*` including guard submodules | S01--S12; taskdump-gated `trace_leaf` projections, including the one in `src/sync/broadcast.rs`, belong to R08; `src/sync/tests/*` supplies I evidence and is not a proof unit |
| `src/future/maybe_done.rs`, `try_join.rs` | F01 |
| `src/future/block_on.rs` | R01 |
| `src/future/trace.rs` | R08 |
| `src/task/yield_now.rs`, `coop/*` | T01 |
| `src/task/spawn.rs`, `builder.rs`, `blocking.rs`, `join_set.rs`, `local.rs`, `task_local.rs` | T02, R07, T03, T04, T05 respectively |
| `src/runtime/builder.rs`, `config.rs`, `context.rs`, `handle.rs`, `runtime.rs`, `id.rs`, `thread_id.rs`, `local_runtime/*` | R01 |
| `src/runtime/task/*` | R02 for core/raw/harness/state/waker/list/trace; T02 for join/abort/error/id public projection |
| `src/runtime/scheduler/current_thread/*` | R03 |
| `src/runtime/scheduler/multi_thread/queue.rs`, `overflow.rs` | R04 |
| remaining `src/runtime/scheduler/multi_thread/*` | R05, except metric/taskdump/trace projections also reviewed in R08 |
| `src/runtime/scheduler/inject/*`, `defer.rs`, `lock.rs` | R06 |
| `src/runtime/blocking/*`, root `src/blocking.rs` | R07 |
| `src/runtime/metrics/*`, `task_hooks.rs`, `dump.rs`, task/scheduler trace and metrics paths | R08 |
| `src/runtime/driver.rs`, `driver/*`, `park.rs`, `process.rs`, `signal/*` | R09; process/signal modules here are runtime glue, while OS behavior belongs to P01--P02 and SG01--SG02 |
| `src/runtime/io/*` | I05, with top-level driver composition in R09 |
| `src/runtime/time/*`, `time_alt/*` | TM04, with top-level driver composition in R09 |
| `src/runtime/tests/*` | integration/loom evidence only; not independent production proof units |
| `src/time/*` | TM01--TM03; timer internals map to TM04 |
| `src/io` trait/read-buffer files | I01 |
| `src/io/ready.rs`, `interest.rs` | I02 |
| `src/io/util/*`, `split.rs`, `join.rs`, `seek.rs` | I03 |
| `src/io/std{in,out,err}.rs`, `stdio_common.rs`, `blocking.rs` | I04 |
| `src/io/async_fd.rs`, `poll_evented.rs` | I05 |
| `src/io/uring/*` | I06 |
| `src/net/*` | N01--N04 |
| `src/fs/*` | FS01 for stateless path calls; FS02 for File/OpenOptions/ReadDir/DirBuilder and their platform/io-uring implementations |
| `src/process/*` | P01--P02 |
| `src/signal/*` | SG01--SG02 |
| `src/macros/join.rs`, `try_join.rs`, `select.rs` | M01, F01, M02 respectively |
| `src/macros/{addr_of,cfg,loom,pin,support,thread_local,trace}.rs` | M03 as Tokio-side expansion/cfg glue; consumers must additionally discharge their owning component contracts |
| `src/util/{linked_list,wake_list,idle_notified_set}.rs` | U01 |
| `src/util/{atomic_cell,bit,cacheline,metric_atomics,ptr_expose,rand,rc_cell}.rs` and `src/loom/*` | U02 |
| `src/util/{as_ref,blocking_check,error,markers,memchr,sharded_list,sync_wrapper,trace,try_lock,typeid,wake}.rs` | U03 |
| `src/lib.rs` and each `mod.rs` | routing/cfg surfaces accounted for by their destination rows; their feature diagnostics are checked with the destination |
| `src/doc/*`, `src/fuzz.rs`, and production-module `tests` subtrees | documentation or verification harnesses, not shipped semantic implementation paths; they provide no numerator credit by themselves |

The audit that added this index did **not** change the denominator: `yield_now`,
`local_runtime`, runtime glue, macro support, and the auxiliary util files share
state and closure conditions already owned by T01, R01/R09, M03, and U02/U03.
No newly mapped path introduced an independent public or downstream semantic
contract that justified splitting a manifest unit. A later split still requires
a written denominator-change reason.

## Immediate closure order

The ledger makes the first residual pass deterministic:

1. S05 `Block::has_value` membership and `Rx::reclaim_blocks` modular-order policies.
2. S02 oneshot production orchestration and payload-slot refinement.
3. S10, S11, and S12 to finish the remaining public `sync` primitives.
4. F01, T01, and U01 as small shared foundations before R02 and the schedulers.

After each closure the row must be updated with proof files, adapter use,
mutation evidence, integrated feature/cfg coverage, and any newly discovered
residual. Changing the denominator requires adding, splitting, merging, or
removing an ID in this file with a written reason; a percentage must never be
recomputed from LOC or test counts.
