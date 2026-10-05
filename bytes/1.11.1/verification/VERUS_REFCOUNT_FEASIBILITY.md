# Verus refcount suitability audit (2026-10-05)

This is a read-only source/API audit, not a Verus verification run. No Verus
installation or new trusted contracts were introduced. Luna xhigh performed
this audit at the user's request for delegation.

Upstream inspected: Verus commit `a2e5e484a9c6a90795bad29622c593de9adcfd5b`.
Source: https://github.com/verus-lang/verus/blob/a2e5e484a9c6a90795bad29622c593de9adcfd5b/source/vstd/atomic.rs

The inspected PAtomicUsize load/fetch_add/fetch_sub API uses hard-coded
Ordering::SeqCst and exposes no ordering argument. atomic_with_ghost supports
atomic/ghost updates through this API. This can support a sequentially
consistent abstract ticket protocol, but does not establish native bytes'
Relaxed increment, Release decrement and final Acquire load. Switching the
native program to SeqCst is not part of this checkpoint.

A later weak-memory experiment must keep the exact native orders. Minimal
litmus: count starts at 2 for A/B. A writes a marker and Release-decrements
2->1. B Relaxed-increments 1->2 to create C, then Release-decrements 2->1.
C Release-decrements 1->0 and performs the final Acquire load. Require the
final owner to recover A's published resource through the release sequence
including the intervening Relaxed RMW. Do not grant all resources merely
because the decrement returned 1.

A complete solution needs a reviewed modification-order/read-from/release-
sequence model, conserved tickets/retired regions, and an explicit connection
to Creusot's affine physical resources. Importing an abstract Verus theorem
as a Creusot trusted contract would add a tool-boundary assumption and must
be recorded as such. Stock Verus alone is not an established solution here.
