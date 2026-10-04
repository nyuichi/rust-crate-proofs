# Tokio mpsc S05 closure checklist

This is the working specification map for closing S05.  A checked item means
the contract was reviewed, the body was proved, and the result was included in
the crate-scoped integrated run; those three statuses are recorded separately
until closure.

## Public surface and intended contracts

- Bounded `channel`, `Sender`, `WeakSender`, `Receiver`, borrowed `Permit` and
  `PermitIterator`, and `OwnedPermit`: preserve one channel identity, exact
  strong/weak ownership, linear capacity, FIFO publication, receiver-close
  semantics, and no resurrection after the last strong sender.
- Bounded send/reserve forms (`send`, `try_send`, `send_timeout`,
  `blocking_send`, `reserve{,_many,_owned}`, and try variants): a successful
  reservation owns exactly the corresponding permits; commit publishes once;
  cancellation or Drop restores them once; closed/full/timeout returns preserve
  the input value and accounting.
- Unbounded `unbounded_channel`, `UnboundedSender`, `WeakUnboundedSender`, and
  `UnboundedReceiver`: the encoded atomic count is twice the queued count plus
  the close bit; successful send publishes one FIFO value and failed send
  returns it.
- Both receiver families (`recv`, `recv_many`, `try_recv`, blocking and poll
  forms, `close`, `is_closed`, `is_empty`, `len`, sender counts): pop values at
  most once in FIFO order, retain the latest registered Waker, preserve the
  caller's existing Vec prefix, restore capacity/count exactly once, and report
  terminal closure only after published and reserved values are drained.
- Identity/count/debug/error/trait surface: `same_channel`, downgrade/upgrade,
  strong/weak counts, capacity/max-capacity, `SendError`, `TrySendError`,
  `TryRecvError`, `RecvError`, timeout errors, Clone/Drop/Debug/Display/Error,
  Send/Sync and unwind marker implementations reflect the same model.
- Feature surface: `sync`; `time` for timeout; `rt` and non-`rt` blocking paths;
  standard and loom configurations; unstable trace/coop gates where compiled.

## Mathematical model

- A logical FIFO sequence with monotonically claimed positions and a unique
  close marker after the final strong sender.
- A block chain whose block starts are aligned by `BLOCK_CAP`, whose slot-ready
  bits distinguish claimed-but-busy from published values, and whose next links
  advance exactly one generation.
- Bounded accounting: `available + reserved + queued == capacity`.
- Unbounded accounting: encoded count represents queued values and receiver
  close independently.
- Endpoint accounting includes live, constructing, and retiring strong/weak
  owners; `strong == 0` is terminal and requests list close exactly once.
- Receiver progress uses one canonical absolute queue position. `recv_many`
  uses the number of successfully appended values as its sole batch progress
  measure. Raw traversal loops use remaining block distance.

## Production representation relation

- `list::Tx::{tail_position,block_tail}` represents the next claim and a
  reachable finalized-prefix hint; `list::Rx::{head,index,free_head}` represents
  the unique consumer cursor and the not-yet-reclaimable prefix.
- `BlockHeader::{start_index,next,ready_slots,observed_tail_position}` represents
  one aligned generation, its successor, value/close/released flags, and the
  release fence needed before reclamation. `Values<T>` owns exactly the values
  whose ready bits are set and which have not been popped.
- `Chan::{tx,rx_fields,rx_waker,semaphore,tx_count,tx_weak_count}` refines the
  FIFO, receive registration, capacity/accounting, and endpoint models.
- The unique `Rx` and its `UnsafeCell<RxFields<T>>` permission serialize raw pop,
  cursor advance, and reclamation. Published-slot permissions move sender to
  queue to receiver; close has no `T` payload.
- `RecvManyGuard::pending` is the linear count of values popped from the raw
  list but not yet reflected in semaphore accounting; normal return and unwind
  each consume it exactly once.
- `CachedParkThread`, Waker execution, allocator/raw-pointer validity, atomic
  ordering, arbitrary `T::{Clone,Drop}`, and scheduler fairness remain only the
  already-frozen shared foundations; all mpsc-specific state above them must be
  body proved.

## Proof split and order

| Component | Contract | Body | Integrated | Removal/closure condition |
|---|---:|---:|---:|---|
| Thin public/orchestration composition | yes | yes | Verus | All leaves compose without reopening raw representation |
| Index/block arithmetic and terminal policy | yes | partial | Verus/tests | `Block::grow` now wraps explicitly; `Block::has_value` still awaits the selected final-index membership policy |
| Block slot publication/read/close flags | yes | yes | Verus | Exact ready-bit/value ownership and Busy/Closed classification |
| Raw list traversal and `QueueRead` refinement | yes | yes | Verus | Logical oracle eliminated; raw list/slot chain determines each result |
| Block reuse and reclamation | yes | partial | Verus/test fixture | Traversal/release lease proved; `Rx::reclaim_blocks` still awaits the selected modular-order policy |
| Receive gate snapshot | yes | yes | Verus | Queue, buffer and capacity/count snapshots cross trace/coop gates |
| `recv_many` raw-pop/Vec/unwind | yes | yes | Verus | Compiled guard/Drop linked; prefix and capacity hold without spare-capacity precondition |
| Endpoint full-count/Arc ordering | yes | yes | Verus | Mpsc-scoped finite execution resources replace the finite count window |
| Last-strong close completion | yes | yes | Verus | Exactly one raw close insertion and receiver wake after count reaches zero |
| `try_recv` Busy/blocking loop | yes | yes | Verus | Raw Busy and arbitrary finite scheduler-witness prefix use a `nat` rank |
| Trace/coop and blocking wrappers | yes | yes | Verus | Both cfg result mappings connect without new Tokio-specific trust |
| Public traits/errors/cfg surface | yes | yes | Verus/tests | Public result/error/identity mappings and standard blocking/cooperative routes are integrated |

## Current residuals

- Select and implement the final-index membership policy for
  `Block::has_value`; its exact debug/release boundary probe remains evidence,
  not a production fix.
- Select and implement the modular ordering policy for the
  `required_index > self.index` decision in `Rx::reclaim_blocks`; the reduced
  lifetime-order fixture deliberately performs no reclamation.

`Block::grow` is no longer a residual: it uses `wrapping_add(BLOCK_CAP)` and
the final-generation regression passes in the configured debug build. The
new queue/value, compiled recv-many, finite-resource endpoint, raw try-recv,
orchestration, and public-surface modules are body-proved and integrated. S05
remains **L / R(partial) / I**, not C, until the two production comparisons
above have reviewed policies and connected regressions.

## Temporary trust policy

No new trusted item is authorized. Thin orchestration may initially call
strongly contracted proof-only leaf scaffolds only if each is marked temporary,
records the exact missing body, and is removed before S05 is reported C/R/I.
`assume`, `admit`, or an unreviewed `external_body` is forbidden. Existing
frozen shared adapters do not establish an mpsc-specific representation fact.

## Validation and stop conditions

- Develop with leaf-local Verus targets and target-local Cargo tests only.
- Same-shaped caller failure twice: redesign the leaf contract. Same area three
  times, proof regression, or 30--45 minutes without structural progress:
  checkpoint the exact VC and report before continuing.
- Final integration is only `tokio/1.52.3/verify-all.bash`, plus target-local
  formatting and diff checks. S05 is C/R/I only when every row above is complete
  and the ledger lists every remaining frozen foundation precisely.
