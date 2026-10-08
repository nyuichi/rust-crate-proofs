# Refcount overflow: post-RMW abort versus guarded RMW

Read-only architecture review, plus an executable arithmetic scheduler model.
No production source, allocator, actual native refcount, or proof configuration
is changed. No solver is run. Snapshots in `sources/` and source-manifest.json
record exact inspected Rust/Creusot/bytes source hashes.

## What the pinned sources establish

Rust nightly-2026-06-22 `alloc/src/sync.rs` lines 2399–2433 implements Arc::clone
using Relaxed fetch_add followed by an overflow abort. Its own comment says:

> This check is not 100% water-proof: we error when the refcount grows beyond
> `isize::MAX`. But we do that check *after* having done the increment, so there
> is a chance here that the worst already happened and we actually do overflow
> the `usize` counter.

The next lines call the required growth before abort “exceedingly unlikely”.
This is not an invariant proof. The top-of-file MAX_REFCOUNT comment also warns
that the soft limit applies to all increment paths. Arc's Weak::upgrade uses a
checked_increment CAS loop (lines 3283 onward); that is a useful native analogue
for guarding an increment, not a proof of Arc::clone's overflow exclusion.

Creusot Std `std/sync.rs` gives Arc::clone an extern contract `result == *self`.
The shipped arc_and_rc example proves observable Arc values and pointer equality
from that contract; it does not prove refcount arithmetic, pending-thread bounds,
or last-owner deallocation. That contract cannot substitute for a body proof of
bytes' refcount protocol under the current policy.

## Concrete abstract-model counterexample

Let M=2^w and L=M/2-1. Complete ordinary clones until the counter is L+1.
Pause M/2 further clone calls just after each fetch_add and before its abort
check. Their recorded old values are L+1 through M-1; the counter wraps to zero.
Another clone now fetches old zero, increments to one, passes the normal-return
guard and returns. Releasing this handle observes old one and selects final
cleanup while earlier handles remain live. No paused thread has executed abort.

`toy_schedule.py` executes this finite schedule for w=3: four existing handles,
four suspended increments from 4 through 7 to 0, one returning clone from 0 to
1, then a release from 1 to 0 with four other completed handles still live.
`toy-result.json` records the assertion-checked output.

This is a counterexample in an unbounded-concurrency arithmetic model. It is
NOT evidence that a native OS can allocate that many threads/stacks, or a native
use-after-free demonstration. Excluding this schedule without a code change
needs a justified bound on simultaneously pending operations (including whatever
reentrancy the execution model permits). The existing proof model supplies no
such physical bound. An arbitrary bytes-specific thread-bound axiom is not a
permitted replacement. Fairness does not exclude this finite bad prefix.

## Small candidate native change (not applied)

For the selected Shared increment, replace fetch_add followed by its postguard:

```rust
match (*shared).ref_cnt.fetch_update(
    Ordering::Relaxed,
    Ordering::Relaxed,
    |old| {
        if old > (usize::MAX >> 1) { None } else { Some(old + 1) }
    },
) {
    Ok(_) => {},
    Err(_) => crate::abort(),
}
```

An explicit compare_exchange_weak loop with the same guard is equivalent for
this purpose. The update closure may run repeatedly; it must contain only the
pure guard/arithmetic, not an affine registration callback.

A successful CAS checks the expected actual value old<=L and stores old+1<=L+1
at one atomic event. Failed/refused attempts do not store. Positive decrements
preserve [0,L+1]. Therefore this interval is inductive, and L+1<M excludes wrap
regardless of the number of suspended callers. The toy script exhaustively
checks these individual event cases for widths 2 through 12; the preceding
inequality argument is the general mathematical justification, not a claim of
an all-width Creusot proof.

Successful clones retain their pointer/length and +1 refcount effect at a Relaxed
RMW. This is not an instruction/performance-equivalence claim: there is an extra
load/CAS retry loop, possible individual starvation, and different intermediate
counts and abort timing on overflowing executions. Existing safety proofs remain
normal-return proofs; do not infer total termination from the loop.

Integer fetch_update is stable since Rust 1.45, below this crate's Rust 1.57
MSRV. The inspected newer nightly deprecates that name in favor of try_update;
using the older name preserves MSRV. The installed Loom 0.7.2 and portable-atomic
1.15.0 also expose fetch_update; this read-only API check is not a configured
build/proof for those backends.

The same pattern also appears in bytes.rs::owned_clone and
bytes_mut.rs::increment_shared. Fixing only bytes.rs::shallow_clone_arc would
establish a selected-representation result, not a whole-crate overflow theorem.
All counter-changing paths and constructors must preserve the chosen bound.

## Generic atomic adapter obligations

A reviewed operation-bound guarded-increment primitive can require limit<MAX
and expose:

- Ok(old): old<=limit, actual successful Relaxed/Relaxed RMW on the exact field,
  store=old+1; one mutable Committer and one FnGhost transition, restored state.
- Err(observed): observed>limit at a Relaxed read, no store BY THIS OPERATION;
  no issued registration or moved retirement token. This does not imply global
  history/state is unchanged while other threads run.
- Internal unsuccessful/spurious CAS retries do not invoke registration or
  mint fragments. Commit only at the actual successful RMW, not every closure
  evaluation. The exact typed field ward and owned-control lease remain required.

Stock atomic::compare_exchange/compare_exchange_weak already provides the
success-mutable-Committer versus failure-read-Committer contract shape. Its
native implementation selects the requested orderings when sc-drf is disabled;
do not take the outdated “always sequentially consistent” doc sentence as the
semantics. The new field/invariant adapter still needs its explicit generic TCB
review and controls.

A successful Relaxed CAS is an RMW and must retain the predecessor's carried
release-sequence publication. That does NOT give the caller an Acquire view or
publish all of its current thread view. In particular, do not copy the existing
Release completion rule's `current <= publication` clause into a Relaxed rule.
The final Release decrement and actual Acquire load remain unchanged. Required
controls include missing Acquire, dropped prior publication, wrong field,
duplicate commit, and issuing a ticket on a failed update.

With count==live-map cardinality (including registered in-flight results), the
nonwrapping bound permits old==1 on release to imply one outstanding ticket.
It does not itself prove map conservation, typed physical recovery, or cleanup:
those remain body-proved bytes protocol obligations.
