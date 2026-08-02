# Tokio mpsc `recv_many` unwind audit

Date: 2026-08-02

This is a local-correctness and resource-accounting audit of Tokio 1.52.3's
production `Chan::recv_many` control flow. It records a confirmed panic-safety
inconsistency and possible regression contracts; no production behavior was
changed.

## Confirmed counterexample

For each `Read::Value(value)`, production performs these operations in order:

1. remove `value` from the raw queue;
2. decrement `remaining`;
3. call `buffer.push(value)`;
4. after the receive loop finishes, return the whole batch to the bounded
   semaphore or subtract it from the unbounded encoded message count.

`Vec::push` can unwind on capacity overflow. This is distinct from ordinary
allocation failure, whose standard infallible-collection path normally aborts
without exposing a caught state. A temporary module-local test used only safe
Rust APIs: `vec![(); usize::MAX]` constructs a zero-sized-element Vec with both
length and capacity equal to `usize::MAX` without allocating a huge buffer. The
next `push(())` panicked with `capacity overflow`.

The probe sent one value through `channel(1)`, caught the `poll_recv_many`
panic, and then observed all of the following:

- `Receiver::try_recv()` returned `Empty`, so the value had left the queue;
- `Sender::capacity()` remained `0`, so the consumed bounded permit had not
  been returned;
- consequently the empty channel could not accept another `try_send`.

The exact crate-scoped bounded probe command was:

```text
cargo test --manifest-path tokio/1.52.3/Cargo.toml --locked \
  --features full,test-util --lib \
  mpsc_poll_recv_many_push_panic_safe_api_probe \
  -- --nocapture
```

It completed with one passing probe after catching the expected standard
library panic. The temporary test asserted the observed inconsistency and was
removed rather than making that behavior a permanent expectation.

## Impact and open boundary

On a bounded channel the queue/capacity conservation relation is broken after
the caught unwind: there is neither a queued value nor an available permit nor
a live reserved-permit owner. A receiver-close path can therefore continue to
observe the semaphore as non-idle even though no future action can restore that
permit. A later raw-list `Closed` observation also reaches production's
`debug_assert!(semaphore.is_idle())` with the leaked permit in debug builds.

The analogous unbounded path removes the value without applying
`fetch_sub(2)`. Its encoded message count can therefore retain a phantom
message after the raw queue is empty. After `Receiver::close`, the stale count
can keep an otherwise terminal receive Pending while a Sender is alive; after
the final Sender is dropped, the same raw-list `Closed` debug assertion can
observe a non-idle count. No security property is claimed or evaluated.

A second temporary safe-API probe confirmed the live-Sender case by closing the
Receiver after the caught panic and observing `Poll::Pending` from a
fresh-buffer `poll_recv_many`. It used the same command with the filter
`mpsc_poll_recv_many_push_panic_unbounded_stale_count_probe` and was likewise
removed after the observation.

The existing Verus `recv_many` refinement is a correct conditional normal-path
proof only when `initial_prefix.len() + limit <= buffer_capacity`. It cannot be
extended across this unwind without first choosing a production panic contract.
Vec allocation, raw allocation validity, and arbitrary value destruction remain
the agreed foundational boundaries, but the ordering of Tokio's own queue and
accounting transitions around `Vec::push` is an S05 obligation and is not moved
into those boundaries.

## Production choice and minimum regression

Two resource-consistent production contracts have different observable and
allocation behavior and therefore require an explicit decision:

1. Reserve enough destination capacity before the first queue pop. If reserve
   panics, the message and channel accounting remain unchanged. This may
   allocate or panic even when fewer than `limit` values are currently ready.
2. Keep the current normal-path batching, but install an unwind guard after a
   successful pop so every removed value is accounted exactly once if
   `Vec::push` panics. This preserves the current allocation timing, but the
   popped value is dropped during unwind and is not restored to the queue.

The minimum regression should reuse the safe `vec![(); usize::MAX]` boundary and
`catch_unwind`. For choice 1 it must assert that the value is still receivable
and capacity remains occupied. For choice 2 it must assert that the queue is
empty and capacity is restored to one. Both variants must also cover an
unbounded channel. After closing the Receiver, choice 1 must receive the
still-queued value as `Ready(1)` into a fresh buffer and then reach `Ready(0)`;
choice 2 must reach `Ready(0)` immediately because the popped value was dropped.
Neither may remain Pending because of a stale encoded count. The selected tests
should initially fail on this audited source, pass only with the chosen
production change, and then be added to `verify-all.bash`'s crate-local mpsc
test filter.

An independent source/accounting review confirmed the pop/push/bulk-accounting
ordering, the validity of the zero-sized-Vec probe, the bounded capacity loss,
and the unbounded stale-count terminal behavior. It also confirmed that a
successful Vec reallocation does not itself invoke `T::Clone` or `T::Drop`;
arbitrary value destruction and a destructor panic during the original unwind
remain separate agreed boundaries.

Until that choice is made, no repair or proof extension should be committed and
S05 remains partial.

## Validation

After the temporary probes were removed, `tokio/1.52.3/verify-all.bash`
completed successfully with all configured Tokio 1.52.3 Rust and loom targets,
`540 verified, 0 errors` for the integrated Verus crate, and `0 verified, 0
errors` for the explicitly external oneshot connection probe. This validates
the unchanged configured proof baseline; it does not make the reproduced unwind
path pass a resource-conservation contract.
