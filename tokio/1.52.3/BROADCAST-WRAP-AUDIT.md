# Tokio broadcast position-wrap audit

This audit records production obligations discovered while attempting to
refine `broadcast`'s logical, non-wrapping ring proof to Tokio's `u64`
positions. No finite-window assumption is part of the frozen trusted
foundation, so these findings remain open until the position policy is chosen.

## Confirmed production boundary defects

### `Receiver::len` and `is_empty`

Production computes `(tail.pos - receiver.next) as usize`.

- With `receiver.next == u64::MAX` and `tail.pos == 0`, one send is unread.
  The subtraction panics when overflow checks are enabled; a wrapping distance
  is one.
- On a 32-bit target, `receiver.next == 0` and `tail.pos == 2^32` produces a
  logical distance of `2^32`, but the `usize` cast yields zero. Consequently
  `is_empty`, which delegates to `len() == 0`, can report true for a lagged
  receiver.

A local repair needs wrapping subtraction, a nonzero-preserving saturating
conversion to `usize`, and an `is_empty` comparison that does not depend on a
truncating length conversion.

### `Receiver::drop` snapshot drain

Production snapshots `until = tail.pos` and drains while
`receiver.next < until`. For capacity one with `next == u64::MAX`, `until == 0`,
and the retained slot at position `u64::MAX`, the comparison is immediately
false. The receiver's remaining-reader contribution is not released, so the
payload and queued state persist until a later overwrite or destruction of the
shared channel.

Changing `<` to `!=` is insufficient: after the tail lock is released, a
concurrent sender can make `recv_ref`'s lag recovery advance beyond the
snapshot. A correct repair needs a bounded snapshot distance as its progress
measure and must stop when a lag jump reaches or passes that original distance.
It must release only slots whose send captured this receiver, never sends that
occurred after the receiver count was decremented.

## Full-cycle representation limit

`Tail::pos`, `Slot::pos`, and `Receiver::next` are all `u64`. If a receiver at
position zero is not observed while exactly `2^64` sends complete, the tail is
again zero. With capacity one the newest slot is tagged `u64::MAX`, yet
`recv_ref` can classify the receiver as empty or closed. After one additional
send it can consume the latest value without reporting the unrepresentable lag.

This is an ABA collision, not an arithmetic lemma that can be repaired with
another `wrapping_*` operation. `RecvError::Lagged(u64)` also cannot represent
all larger distances. Closing the proof requires one reviewed policy:

1. an explicit invariant that fewer than `2^64` sends occur between relevant
   observations of a receiver;
2. a wider/tagged position representation together with a defined saturation
   policy for `Lagged(u64)`; or
3. a terminal, documented overflow behavior before position reuse.

Under policy 1, the existing send, power-of-two mask, initial slot tags,
subscribe, and `recv_ref` Ready/Empty/Lagged arithmetic can be refined for the
single-cycle machine interval. The `len`, `is_empty`, and drop defects still
need production changes and regression tests.

## Verification status

`verification/src/broadcast_refinement.rs` now body-proves a conditional,
production-shaped single-cycle slice:

- exact `u64::wrapping_add`/`wrapping_sub` position adapters and equality-based
  empty classification;
- subscribe-at-tail and send reservation across `u64::MAX`;
- power-of-two mask index bounds;
- the production initial tag `index.wrapping_sub(capacity)` and its empty
  relation `tag.wrapping_add(capacity) == index`;
- exact send-ticket publication into a physical-slot tag, once-only overwrite
  indication, and tag-based Ready/Empty/DifferentGeneration classification;
- wrap-crossing ready and lag recovery witnesses, including two tickets one
  capacity apart that share an index but retain distinct generation tags; and
- a sequential bounded drop skeleton that skips overwritten generations and
  releases exactly `min(snapshot_unread, capacity)` retained reader
  contributions.

The module-local production tests
`broadcast_position_wrap_send_receive_preserves_generation_order` and
`broadcast_position_wrap_lag_recovers_to_oldest_generation` seed the private
tail cursor near `u64::MAX` and exercise the compiled send/recv/lag branches
across rollover. Existing exact loom cases continue to cover ordinary physical
ring wrap, two receivers, and receiver drop races.

The finite observation window is only a proof precondition for a candidate
policy; it is neither frozen nor selected. The sequential drain model also
holds its tail snapshot fixed and therefore does not refine sends concurrent
with production's unlocked drop loop. The confirmed `len`/`is_empty` and drop
defects above are unchanged. Generic mask-generation equivalence and the full
physical slot/rem/waiter lock composition remain residuals.

Production position wrapping is therefore **R(partial), not closed**. No new
trusted assumption or production behavior change was introduced; production
source changes are confined to `cfg(test)` regressions.
