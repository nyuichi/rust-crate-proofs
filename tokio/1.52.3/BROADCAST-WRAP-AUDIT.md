# Tokio broadcast terminal-position audit

This audit records the reviewed policy and production refinement for Tokio
1.52.3 broadcast's `u64` positions. The previous implementation reused
position zero after `u64::MAX`, which admitted a full-cycle ABA and made
`Receiver::len`, `is_empty`, and Drop behave differently or incorrectly at
rollover.

## Selected policy

Position exhaustion is terminal and identical in every build:

- successful sends reserve only positions `0..=u64::MAX - 1`;
- `tail.pos == u64::MAX` is an irreversible sentinel, never a send ticket;
- with at least one active Receiver, a send at the sentinel panics before any
  tail, slot, `rem`, value, or waiter mutation;
- with no active Receiver, the pre-existing `SendError(value)` branch retains
  priority and does not panic; and
- subscribing at the sentinel creates an empty Receiver at `next == MAX` but
  cannot restart sending.

The input to a terminal send is dropped during unwind. Arbitrary Drop execution
and a destructor's own panic/abort behavior remain in the frozen foundation;
the Tokio-specific fact proved and tested here is that the gate precedes every
channel mutation.

The sentinel does not close the Sender. A Receiver at the empty terminal
position can remain Pending until all Sender handles are dropped. This is the
documented terminal liveness behavior, not a claim that exhaustion publishes a
close event.

## Production repairs

### Send reservation and slot generation

`Sender::send` checks `rx_cnt == 0` first and then rejects `tail.pos == MAX`
with an all-build `assert!`. Its successful advance is ordinary checked
addition. Consequently position zero is never reused and the maximum published
slot tag is `MAX - 1`.

Channel initialization still uses `index.wrapping_sub(capacity)` for empty slot
tags. At a reachable terminal tail, every physical slot has long since been
overwritten; the slot queried by `next == MAX` was most recently published at
`MAX - capacity`. Production's existing equality order therefore rejects Ready
and observes `slot.pos + capacity == next`, classifying Empty or Closed without
position reuse. This generic relation is source correspondence; the current
body proof instantiates it at capacity two, while generic mask-generation
equivalence remains open.

### `Receiver::len` and `is_empty`

Reachable cursors now satisfy `next <= tail`. `len` computes the exact
non-wrapping `tail - next` distance and saturates it to `usize::MAX`, preserving
nonzero lag on 32-bit targets. `is_empty` compares `next == tail` directly and
does not depend on a truncated length conversion.

### Lag recovery and Drop

Under monotonic positions, lag recovery computes `oldest = tail - capacity` and
`missed = oldest - next`. The lag branch has strictly positive `missed`; the old
wrapping `missed == 0` special Ready path is removed. A Ready receive advances
only through `MAX - 1` to the empty sentinel.

Receiver Drop snapshots `until = tail.pos` while decrementing `rx_cnt` under the
tail lock. Its existing `while next < until` condition is now the correct
monotonic bound. Sends after the snapshot have tickets `>= until` and do not
capture the dropped Receiver. A Ready iteration releases only its current
ticket `< until`; a concurrent-overwrite lag jump releases nothing and may move
`next >= until`, where the loop stops. It therefore cannot decrement `rem` for a
post-snapshot send.

## Proof and regression evidence

`verification/src/broadcast_refinement.rs` body-proves:

- no-Receiver/terminal/reserved send-gate priority and rejection-state
  preservation;
- monotonic terminal reservation with `MAX - 1` as the final ticket;
- exact non-wrapping unread distance, host-independent 32/64-bit saturation,
  and equality-based emptiness;
- a capacity-two reachable terminal queried-slot witness and Empty relation;
- exact Ready/Empty/Lagged cursor transitions with positive lag;
- retained-slot sequential drain conservation; and
- a concurrent Drop step in which every released ticket is below the snapshot,
  while lag advances without releasing.

Module-local regressions cover final-ticket send/receive, terminal lag recovery,
mutation-free panic and input destruction, no-Receiver error priority,
subscribe-at-terminal, 32-bit-limit saturation, terminal emptiness, and retained
value release. The exact loom regression
`drop_rx_preserves_concurrent_send_for_surviving_receiver` covers both sides of
the Drop snapshot race and checks that the surviving Receiver retains the
concurrent value. Existing broadcast loom cases continue to cover ordinary
physical ring reuse, two receivers, and receiver drop.

## Remaining S04 boundary

The full-cycle ABA, debug/release discrepancy, `len` truncation, wrapping Drop,
and post-snapshot-release defects are closed by the selected policy and repairs.
S04 remains R(partial), not C: generic mask-generation equivalence, the direct
physical `Slot<T>`/`rem` coupling, waiter/slot lock orchestration, weak endpoint
atomics, and public trait/error surfaces remain to be connected above the frozen
atomics, Mutex, raw-link, Waker, and arbitrary Clone/Drop foundation.
