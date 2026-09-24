# Tokio mpsc index-wrap audit

Date: 2026-08-02

This is a source audit of Tokio 1.52.3's production block geometry. The
debug/release inconsistency in `Block::grow` is fixed with explicit wrapping;
the other candidates remain unchanged pending reachability review.

The configured `BLOCK_CAP` is 32 on non-loom 64-bit targets, 16 on other
non-loom pointer widths, and 2 under loom. Each is a power of two smaller than
the number of bits in `usize`. `start_index` and `offset` therefore use the
expected masks, while tail claims, receiver advancement, distance, and most
next-block links explicitly use wrapping arithmetic.

Two expressions originally did not follow that policy:

| Production path | Expression | Current status |
|---|---|---|
| `Block::grow` | `self.header.start_index + BLOCK_CAP` | **fixed:** `wrapping_add`, with a final-generation regression in every build |
| `Block::has_value` | `self.header.start_index + BLOCK_CAP` | unchanged: debug can overflow-panic; release wraps the upper bound to zero and can report a final-block slot outside the block |

The release-mode `has_value` result propagates into `Rx::is_maybe_closed` and
`Rx::len`: a real final-slot value can be treated as the synthetic close marker
and subtracted. In the one-message case this can make `len()` report `0` and
therefore make `is_empty()` report `true` even though the value is present.

The reclamation path has a separate comparison to review. After `tx_release`
records a modular `observed_tail_position`, `Rx::reclaim_blocks` uses
`required_index > self.index`. Across the word boundary, a small post-wrap
required index compares below a large pre-wrap receiver index even though it is
logically ahead. The Verus closure work therefore includes an explicit sender
traversal/release lease, but does not yet claim that this ordinary comparison
implements the lease at the boundary.

Reaching the boundary requires approximately a full `usize` cycle of claimed
messages, so ordinary and loom schedules cannot practically reproduce it.
Nevertheless, the remaining inconsistency is semantic: neighboring `try_push`,
`load_next`, receiver index advance, length, and distance operations explicitly
wrap. A production decision for `has_value` and reclamation should first use an
exact near-boundary block fixture and a controlled sender-traversal schedule,
cover both overflow-checking and release builds, and then choose a reviewed
full-cycle modular-order policy before S05's index/reuse refinement can close.

Until then, Verus keeps the logical non-wrapping generation, erased modular
geometry, and traversal lease separate and does not claim the two unresolved
production comparisons implement the final machine-word cycle.
