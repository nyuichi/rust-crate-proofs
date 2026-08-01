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

The existing non-wrapping logical ring and receiver-conservation bodies remain
proved. Production position wrapping is **not closed**. No new trusted
assumption, production patch, or completion claim was introduced by this audit.
