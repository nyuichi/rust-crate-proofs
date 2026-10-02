# Tokio async RwLock S12 closure checklist

Scope: Tokio 1.52.3 `sync::RwLock<T>` and its six guard representations under
the standard `sync` feature.

| Obligation | Evidence | Status |
|---|---|---|
| constructor bounds | exact `u32::MAX >> 3` / divide-by-eight maximum and nonzero custom-reader validation | body-proved / tested |
| reader permit accounting | every borrowed/owned reader holds exactly one of `mr` permits | body-proved / tested / loom-tested |
| writer exclusion | every borrowed/owned writer holds all `mr` permits | body-proved / tested / loom-tested |
| immediate try paths | exact read/write success conditions and mutation-free failure | body-proved / tested |
| FIFO and writer preference | partially funded oldest writer prevents later readers from bypassing | body-proved / tested |
| pending-future cancellation | exact writer removal restores surviving FIFO order and allows later reader progress | body-proved / tested |
| read map/try_map | borrowed and owned projection preserves exactly one reader permit; failure returns original guard | body-proved / tested |
| write map/try_map/into_mapped | borrowed and owned projection preserves all `mr` permits; mapped type cannot downgrade | body-proved / tested |
| atomic downgrade | retains one permit and releases exactly `mr - 1`; no intervening writer can acquire | body-proved / tested / loom-tested |
| downgrade_map/try_downgrade_map | projection success returns one read capability; failure preserves original writer and all permits | body-proved / tested |
| all guard destructors | read forms release one; write and mapped-write forms release exactly `mr` | body-proved / tested / loom-tested |
| owned lifetimes | every owned projection retains the same Arc-backed lock identity | body-proved above Arc adapter / tested |
| value ownership | write mutation, `get_mut`, and `into_inner` conserve arbitrary `T` exactly once | body-proved / tested |
| blocking and trait surfaces | blocking read/write, From/Default, Debug state selection, guard formatting, existing Send/Sync/Unpin checks | body-proved where Tokio selects state / compile/runtime-tested |
| mutation sensitivity | one-permit writer, premature mapped release, and wrong downgrade release count are rejected | body-proved |
| cfg integration | ordinary `full,test-util`, standard sync compilation, bounded loom races, Verus crate | integrated |

The refinement composes S06's already-closed batch semaphore. Rust weak-memory
atomics, Arc lifetime mechanics, UnsafeCell and raw-pointer validity,
Pin/Poll/Context/Waker mechanics, arbitrary Drop execution, and scheduler
liveness remain the frozen foundation. The mixed read/write loom case uses a
preemption bound of one because bound two has a deliberately explosive search;
the unbounded protocol state space is covered by the Verus FIFO/accounting
proof rather than claimed from bounded model checking.

No production source change, temporary trusted function, `assume`, `admit`, or
new axiom is introduced. Taskdump-gated tracing remains owned by R08.
