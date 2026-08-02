# Tokio mpsc index-wrap audit

Date: 2026-08-02

This is a source audit of Tokio 1.52.3's production block geometry. It records
a bug candidate; no production behavior was changed.

The configured `BLOCK_CAP` is 32 on non-loom 64-bit targets, 16 on other
non-loom pointer widths, and 2 under loom. Each is a power of two smaller than
the number of bits in `usize`. `start_index` and `offset` therefore use the
expected masks, while tail claims, receiver advancement, distance, and most
next-block links explicitly use wrapping arithmetic.

Two expressions do not follow that policy:

| Production path | Expression | Debug behavior at the final aligned block | Release behavior |
|---|---|---|---|
| `Block::grow` | `self.header.start_index + BLOCK_CAP` | integer-overflow panic when growing from the final aligned block to index zero | wraps to zero |
| `Block::has_value` | `self.header.start_index + BLOCK_CAP` | integer-overflow panic while checking a slot in the final aligned block | wraps the upper bound to zero, causing the range check to report every slot outside that block |

That release-mode `has_value` result propagates into `Rx::is_maybe_closed` and
`Rx::len`: a real final-slot value can be treated as the synthetic close marker
and subtracted. In the one-message case this can make `len()` report `0` and
therefore make `is_empty()` report `true` even though the value is present.

Reaching the boundary requires approximately a full `usize` cycle of claimed
messages, so ordinary and loom schedules cannot practically reproduce it.
Nevertheless, the inconsistency is semantic: neighboring `try_push`,
`load_next`, receiver index advance, length, and distance operations explicitly
wrap. A future production fix and regression should use an exact near-boundary
block fixture, cover both overflow-checking and release builds, and then choose
a reviewed full-cycle generation/ABA policy before S05's index/reuse refinement
can close.

Until then, Verus models use logical non-wrapping indices or finite local queue
positions and do not claim the final machine-word cycle.
