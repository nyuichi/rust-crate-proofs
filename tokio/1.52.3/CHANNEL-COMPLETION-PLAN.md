# Scoped channel-completion specification map

The completion target is every watch-, broadcast-, and mpsc-specific protocol
transition. The following common runtime foundations remain explicit trusted
adapters: Rust weak-memory atomics, lock and Arc implementations, UnsafeCell,
Pin/Poll/Context/Waker execution, raw allocation validity, arbitrary user trait
and destructor execution, and scheduler liveness.

No other channel-specific behavior may be moved into those boundaries.

## watch

Public operation groups and contracts:

| Operations | Required contract |
|---|---|
| `channel`, `Sender::new` | one current value, generation zero, one endpoint of each kind |
| `send`, `send_replace` | exact replacement, generation advances iff published, notify after publication |
| `send_modify`, `send_if_modified` | modified/unmodified/panic outcomes preserve the exact documented value and generation behavior |
| `borrow`, `borrow_and_update`, `has_changed` | returned value and per-Receiver seen generation come from one locked snapshot |
| `changed`, `wait_for` | register before checking, recheck after registration, advance only to an observed generation |
| clone/drop/subscribe/closed | exact endpoint counts, only final Sender closes, subscription starts at current generation |

Canonical model: a current value, a logical generation, independent Receiver
seen generations, endpoint counts, and closed state. Runtime relation: the low
atomic bit is closed and the remaining bits are an even wrapping version; the
RwLock owns the current value; BigNotify implements generation-independent
broadcast wake-up.

Internal proof boundaries: encoded state, exact-value update/unwind, one
Receiver changed protocol, endpoint lifecycle, and top-level refinement.

## broadcast

Public operation groups and contracts:

| Operations | Required contract |
|---|---|
| `channel`, `Sender::new` | positive rounded capacity, logical tail, exact initial Receiver count |
| `send` | unique position, captured Receiver count, exact slot replacement, notify after slot publication |
| `subscribe`, `resubscribe` | cursor starts at the current tail and affects only future sends |
| `recv`, `try_recv` | per-Receiver FIFO, exact Clone source, Empty/Closed/Lagged distinction |
| `len`, `is_empty` | cursor/tail/ring-capacity projection including lag |
| clone/drop/weak/closed | endpoint counts, slot remaining-reader conservation, final cleanup |

Canonical model: an unbounded logical message history, fixed physical capacity,
per-Receiver cursor, retained interval `[tail-capacity, tail)`, per-message
remaining-reader set/count, and endpoint lifecycle. Runtime relation: logical
positions map to `position & mask`; slot generation distinguishes reuse.

Internal proof boundaries: logical retained interval, ring-index refinement,
slot ownership/rem, lag transition, Receiver drain/drop, waiter refinement, and
Clone/unwind refinement.

## mpsc

Public operation groups and contracts:

| Operations | Required contract |
|---|---|
| bounded/unbounded constructors | one Receiver, one Sender, empty FIFO, exact capacity state |
| `send`, `try_send`, timeout forms | exact success/full/closed ownership result and FIFO claim |
| reserve/owned/many permits | linear capacity transfer; commit or cancellation returns every permit exactly once |
| `recv`, `try_recv`, `recv_many` | sole-Receiver FIFO, Busy cannot be passed, register then recheck |
| `close`, endpoint drop | no new permits, reserved sends drain, values precede close marker, exact cleanup |
| clone/downgrade/upgrade/counts | strong/weak counts and no resurrection after final strong Sender |

Canonical model: an arbitrary logical FIFO, claimed/published prefix positions,
one Receiver head, optional final close marker, linear bounded permits, and
endpoint counts. Runtime relation: logical positions partition into fixed-size
blocks; ready bits publish cells; the Receiver alone advances and reclaims the
block prefix.

Internal proof boundaries: arbitrary FIFO orchestration, one-block slot
publication, block-chain ownership/reclamation, semaphore/permit lifecycle,
poll refinement, endpoint lifecycle, and top-level bounded/unbounded
refinement.

## Development and stop conditions

- Each loop or recursive traversal uses one logical position: watch generation,
  broadcast cursor/tail, or mpsc slot index.
- Raw representation arithmetic is proved separately from value ownership.
- No channel function should generate roughly 100 goals; split it first.
- A same-shaped caller/interface failure twice triggers interface review.
- Three failures in one proof area trigger a structural checkpoint rather than
  more assertions or prover depth.
- Completion requires body proof, representative caller, production-scoped
  tests, mutation sensitivity, and a successful `verify-all.bash` run.
