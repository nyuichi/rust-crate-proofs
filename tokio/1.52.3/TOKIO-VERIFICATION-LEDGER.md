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
- **current full-closure score: 2/53 (4%) C**, all fully production-refined
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

## Component/API-path manifest

| ID | Area and production paths | Public or downstream surface | Feature / cfg | Status and evidence | Trusted adapters used | Residual gap / next closure condition |
|---|---|---|---|---|---|---|
| S01 | SetOnce: `src/sync/set_once*.rs` | `SetOnce`, set/get/wait/take/traits | `sync` | **L / R / I**; `verification/src/{lib,published_cell,release_acquire,writer_lease,wait_protocol,...}.rs`, `PROVENANCE.md` | atomics, loom Mutex/UnsafeCell, poll/Waker, raw links, user trait/Drop execution | Generic Clone/PartialEq outcome contracts and state preservation are proved, but Debug/Display/Error formatting is tests-only. Prove their production trait orchestration and state preservation to reach C. |
| S02 | oneshot: `src/sync/oneshot.rs`, `oneshot_value.rs` | channel, Sender/Receiver, poll/close/drop | `sync`; internal under `rt` and Windows `process` | **L / R(partial) / I**; `oneshot_{state,atomic,value,poll,closed,drop,refinement}.rs`, `ONESHOT-MUTATION-AUDIT.md`; exact reachable four-bit/CAS, staged store-close/drop-complete races, outer endpoint/local-Arc and physical proof-slot capability refinement, repeated terminal try/close, TX-Waker terminal cleanup; `rt`-only and `sync`-only compile gates | raw atomic/UnsafeCell semantics, Arc, Pin/Poll/Context/Waker, user Drop execution | Connecting the state capability to the PCell permission inside Tokio's shared-`&self` UnsafeCell wrapper is an unclosed Tokio-specific representation refinement, not a frozen adapter. Prove task-bit/Waker transition-in-progress states with a concurrent atomic invariant, `coop::poll_proceed`/`trace_leaf`, `blocking_recv` through the `rt` runtime-context/BlockingRegion and non-`rt` CachedParkThread runtime/park paths, Future Ready-repoll panic, unstable tracing-panic preservation, and public error/Debug/Display/Error/Clone paths; run the Windows process-only cross-build before C/R. |
| S03 | watch: `src/sync/watch.rs` | channel, send/update, borrow/changed/wait_for, endpoint lifecycle | `sync` | **L / R(partial) / I**; `watch_{protocol,completion,refinement}.rs`, `WATCH-MUTATION-AUDIT.md`; exact wrapping state/update phases, circular selection, eight-shard call generations, registration snapshots, repeated fanout, and poll priorities proved | atomics, RwLock, Waker, Clone/Drop execution | Full watch and Notify-counter cycles remain explicit ABA boundaries; the Notify WAITING path also has a debug-overflow boundary. Refine `wait_for` predicate orchestration, endpoint/refcount/closed registration races, and the RNG selector/U02 connection before C/R; update-closure outcomes remain specification inputs. |
| S04 | broadcast: `src/sync/broadcast.rs` | channel, send/recv/try_recv, lag, subscribe, weak/endpoints | `sync` | **L / P / I** under the earlier channel scope; `broadcast_{protocol,completion}.rs`, `BROADCAST-WRAP-AUDIT.md` | atomics, Mutex, raw waiter links, Waker, Clone/Drop execution | Choose a reviewed full-cycle policy; repair and refine wrapping `len`/`is_empty`, bounded drop drain, mask/slot generation, and waiter/slot orchestration. |
| S05 | mpsc: `src/sync/mpsc/{chan,list,block,bounded,unbounded}.rs` | bounded/unbounded send/reserve/recv/close, permits, weak/endpoints | `sync`; timeout forms also `time` | **L / R(partial) / I**; `mpsc_{protocol,completion,refinement,endpoint_refinement,try_recv_refinement}.rs`, `MPSC-{MUTATION,WRAP,RECV-MANY-UNWIND}-AUDIT.md`; exact `recv`/`recv_many` first-pop/register/recheck priorities, latest-Waker retention, reserved-permit send/cancel continuations, bounded single/bulk permit restoration, preallocated caller-prefix/capacity preservation, logical-oracle observation-order batches, exact unbounded encoded-count bulk decrement, source-reviewed unwind accounting after post-pop `Vec::push` failure with two-value production regressions and a linear pending-count model, phase-accurate strong/weak endpoint counts, Sender/OwnedPermit ownership moves, upgrade/no-resurrection, and below-saturation finite-prefix `try_recv` Busy wake/register/recheck/park safety, limit-zero behavior, bounded/unbounded public-surface tests | S06 semaphore and S09 AtomicWaker contracts; atomics, Arc, UnsafeCell, Pin/Poll/Context/Waker, raw allocation/pointers, Drop execution, liveness | `QueueRead` remains a logical oracle not yet connected to the raw list/slot chain. Gate preservation covers only Waker/returned-permit/progress bookkeeping, not queue/buffer/capacity snapshots. The recv-many buffer-content proof remains conditional on `initial_prefix.len() + limit <= buffer_capacity`; outside that window, the source-reviewed guard and two-value safe-API regressions cover the selected capacity-overflow accounting transition, while the logical witness proves once-only consumption of a recorded pending count. Raw-pop/Vec-unwind/compiled-Drop refinement, allocation, and arbitrary destructor execution remain outside the Verus model. Endpoint additions assume the finite `endpoint_count_room` window; production full-count and concurrent increment-before-Arc-clone overflow remain unproved. The endpoint proof stops at last-strong `CloseRequested`; raw-list close insertion/wake completion and CAS-loop liveness remain open. `try_recv` does not yet refine raw Busy detection or CachedParkThread construction/waker/park mechanics. Production loop termination is unproved; the generally allowed scheduler-liveness foundation has not been instantiated into a termination argument. Fix/review final-index `grow`/`has_value` overflow, then refine index generations, block geometry/reuse/reclamation and trace/coop execution. |
| S06 | semaphore: `src/sync/{batch_semaphore,semaphore}.rs` | Semaphore, borrowed/owned permits, acquire/close/add/forget | `sync`; internal Mutex support under `fs` | **C / R / I**; `semaphore_{protocol,refinement}.rs` | atomics, Mutex, Arc, raw waiter links, Waker | Closed under frozen scope. |
| S07 | Notify: `src/sync/notify.rs` | notify_one/last/waiters, Notified/OwnedNotified enable/poll/drop | `sync`; internal under `rt`, `signal`, `process` | **L / R(partial) / I**; `notified_core.rs`, `notify_{protocol,refinement}.rs`, `NOTIFY-MUTATION-AUDIT.md` | atomics, Mutex, raw links, Pin/Poll/Waker | The WAITING `notify_waiters` path uses checked `data + 4` and panics at the final counter value in debug builds; release wraps and permits a full-generation ABA. Decide and refine a finite observation window or change the production counter policy before C/R. |
| S08 | Barrier: `src/sync/barrier.rs` | new/wait, reusable cohorts and leader result | `sync` | **L / R / I**; `barrier_{protocol,refinement}.rs` | Mutex, watch storage/wake, liveness | Exact Barrier state is refined, but follower release depends on the not-yet-refined watch subset in S03. Re-evaluate composition after S03 reaches C; whole-word-stalled follower remains the declared finite-counter liveness boundary. |
| S09 | AtomicWaker: `src/sync/task/atomic_waker.rs` | internal wake/register primitive used by channels/runtime | `sync` and internal consumers | **C / R / I**; `atomic_waker_{protocol,refinement}.rs` | atomics, UnsafeCell, Waker execution | Closed under frozen scope. |
| S10 | OnceCell: `src/sync/once_cell.rs` | new/get/set/get_or_init/get_or_try_init/take/traits | `sync` | **U** | expected SetOnce, futures, user init/Drop | Reuse S01 and prove cancellation, retry after error/panic, recursive-init behavior, and every public constructor/trait path. |
| S11 | async Mutex: `src/sync/mutex.rs` | lock/try_lock, owned and mapped guards, get_mut/into_inner | `sync`; internal under `fs` | **U** | batch semaphore, Arc, UnsafeCell, Drop | Refine one-permit exclusion to guard mapping/owned lifetimes, cancellation FIFO, unlock-on-drop, and value ownership. |
| S12 | async RwLock: `src/sync/rwlock.rs`, `src/sync/rwlock/*.rs` | read/write/try/owned guards, map/downgrade/get_mut/into_inner | `sync` | **U** | batch semaphore, Arc, UnsafeCell, Drop | Prove reader/writer permit accounting, writer exclusion/fairness, downgrade atomicity, all mapped/owned guard Drop paths. |
| F01 | future combinators: `src/future/{maybe_done,try_join}.rs`, `src/macros/try_join.rs` | `try_join!` orchestration and internal MaybeDone | `macros` for macro; core future internals | **U** | Pin/Poll/Context, arbitrary Future/Drop | Prove state transitions, no repoll after completion, early-error cancellation/drop, output ordering, and macro arities. |
| T01 | cooperative yield/budget: `src/task/{yield_now,coop/*}` | yield_now, poll_proceed/consume_budget/unconstrained | `rt`; `test-util` controls | **P / R(partial) / I**; `verification/src/coop_refinement.rs`, `COOP-MUTATION-AUDIT.md`; exact logical, production-shaped models of `Budget::decrement` and accessible-context `poll_proceed` branches, conservation through zero, forced-Pending Waker registration, uncommitted restoration/progress commit, LIFO initial/unconstrained nesting including unwind reset, and trace-before-yielded defer/recheck ordering; production correspondence checked by source review, unit/integration tests, and both scheduler loom yield cases | Pin/Poll/Context/Waker mechanics, arbitrary Drop execution, scheduler liveness | The selected sequential budget/yield state slice is body-proved, with production correspondence supplied by source review and tests rather than a direct compiled connection. Task-local/context access, the compiled TLS closure, ResetGuard Drop connection, `poll_fn` capture/pinning for `consume_budget` and `yield_now`, `Unconstrained<F>::poll`, inaccessible-TLS fallback, metric increment, and scheduler defer ownership remain unproved residuals before C/R. Liveness remains only a frozen adapter and is not established by this safety slice. |
| T02 | spawn/join/abort surface: `src/task/{spawn,builder}.rs`, `src/runtime/task/{join,abort,error,id}.rs` | spawn, JoinHandle, AbortHandle, JoinError, ids | `rt`; builder unstable | **U** | runtime task core, panic payload/Drop, Waker | Close after R02 task-state refinement plus exact join/abort/panic/cancel result ownership. |
| T03 | JoinSet: `src/task/join_set.rs` | spawn/join_next/abort/shutdown/detach_all | `rt` | **U** | task core, owned linked list, Waker | Prove membership and output ownership, wake/recheck, cancellation/drop, and task-id consistency. |
| T04 | LocalSet/local spawn: `src/task/local.rs` | LocalSet, spawn_local, run_until, enter | `rt` | **U** | thread identity/TLS, scheduler, Waker | Prove owner-thread confinement, queue ownership, wake routing, enter nesting, and shutdown/drop. |
| T05 | task-local storage: `src/task/task_local.rs` | task_local!, LocalKey::scope/sync_scope/get/try_with | `rt`; macro surface core | **U** | TLS/context, Future/Drop/unwind | Prove scoped replacement/restoration including panic/cancel and nested values. |
| R01 | runtime construction/lifecycle: `src/runtime/{builder,runtime,handle,context,config}.rs`, `local_runtime/{runtime,options}.rs` | Builder, Runtime, LocalRuntime, Handle, enter/block_on/shutdown | `rt`; multi-thread options `rt-multi-thread`; local runtime unstable | **U** | OS threads/parking/time/I/O contracts | Define lifecycle state and prove construction rollback, enter context, shutdown ownership, and driver/scheduler composition. |
| R02 | task core/state: `src/runtime/task/{state,core,raw,harness,waker}.rs` | downstream basis of spawned tasks | `rt` | **U** | atomics, raw allocation/vtable, Pin/Poll/Waker, panic/Drop | Prove exact state bits/refcount, schedule/poll/cancel/complete/join linearization, deallocation, and unwind paths. |
| R03 | current-thread scheduler: `src/runtime/scheduler/current_thread/*` | current-thread Runtime execution | `rt` | **U** | R02, parking/clock/I/O, liveness | Prove local/remote queue ownership, tick/poll ordering, wake injection, block_on and shutdown drain. |
| R04 | multi-thread local queues: `src/runtime/scheduler/multi_thread/{queue,overflow}.rs` | internal work queue / stealing | `rt-multi-thread` | **U** | atomics, raw task ownership, liveness | Prove ring indices, push/pop/steal/overflow races, wrap, and exact task conservation. |
| R05 | multi-thread workers: `src/runtime/scheduler/multi_thread/{worker,idle,park,handle}.rs` | multi-thread Runtime scheduling | `rt-multi-thread` | **U** | R02/R04, threads, parking, randomness, liveness | Compose task ownership across workers/inject/steal, search/park transitions, shutdown, and blocking handoff. |
| R06 | inject/defer support: `src/runtime/scheduler/{inject,defer,lock}.rs` | remote scheduling and deferred wake/free | `rt` / `rt-multi-thread` variants | **U** | Mutex/atomics/raw task links | Prove closed queue semantics, batch pop, deferred ownership, and no lost/duplicated tasks. |
| R07 | blocking pool: `src/runtime/blocking/*`, `src/blocking.rs`, `src/task/blocking.rs` | spawn_blocking, block_in_place | `rt`; block_in_place `rt-multi-thread` | **U** | OS threads/condvars, task core, liveness | Prove queue/task ownership, worker creation/retirement, cancel/shutdown, and scheduler handoff. |
| R08 | metrics/hooks/dump: `src/runtime/{metrics,task_hooks,dump}.rs` and scheduler metrics | RuntimeMetrics, unstable hooks/taskdump | `rt`; `tokio_unstable`; `taskdump` | **U** | counters, backtrace/platform stack inspection | Prove metric invariants and hook ordering; isolate taskdump platform contracts and verify snapshot ownership. |
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
| `src/sync/*` including guard submodules | S01--S12; `src/sync/tests/*` supplies I evidence and is not a proof unit |
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

1. S04 broadcast production position wrap and ring-generation refinement.
2. S03 watch BigNotify/fanout and production update/poll refinement.
3. S05 mpsc index/block/reclamation and last-close completion refinement.
4. S02 oneshot production orchestration and payload-slot refinement.
5. S10, S11, and S12 to finish the remaining public `sync` primitives.
6. F01, T01, and U01 as small shared foundations before R02 and the schedulers.

After each closure the row must be updated with proof files, adapter use,
mutation evidence, integrated feature/cfg coverage, and any newly discovered
residual. Changing the denominator requires adding, splitting, merging, or
removing an ID in this file with a written reason; a percentage must never be
recomputed from LOC or test counts.
