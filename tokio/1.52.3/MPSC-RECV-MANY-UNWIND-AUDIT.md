# Tokio mpsc `recv_many` unwind audit

Date: 2026-08-02

This is a local-correctness and resource-accounting audit of Tokio 1.52.3's
production `Chan::recv_many` control flow. It records the confirmed
panic-safety inconsistency, the selected unwind-time accounting contract, and
the regression evidence for its repair.

## Confirmed counterexample

Before the repair, production performed these operations for each
`Read::Value(value)`:

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

## Impact before the repair

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
observe a non-idle count.

A second temporary safe-API probe confirmed the live-Sender case by closing the
Receiver after the caught panic and observing `Poll::Pending` from a
fresh-buffer `poll_recv_many`. It used the same command with the filter
`mpsc_poll_recv_many_push_panic_unbounded_stale_count_probe` and was likewise
removed after the observation.

Before the production contract was selected, the Verus `recv_many` refinement
was a conditional normal-path proof only when
`initial_prefix.len() + limit <= buffer_capacity`; it did not cover the unwind.
Vec allocation, raw allocation validity, and arbitrary value destruction remain
the agreed foundational boundaries, while Tokio's own post-pop accounting is
now an explicit S05 guard obligation rather than part of those boundaries.

## Selected production contract and regression

Two resource-consistent production contracts had different observable and
allocation behavior:

1. Reserve enough destination capacity before the first queue pop. If reserve
   panics, the message and channel accounting remain unchanged. This may
   allocate or panic even when fewer than `limit` values are currently ready.
2. Keep the current normal-path batching, but install an unwind guard after a
   successful pop so every removed value is accounted exactly once if
   `Vec::push` panics. This preserves the current allocation timing, but the
   popped value is dropped during unwind and is not restored to the queue.

Choice 2 was selected. Production now creates a call-local `RecvManyGuard` after
the outer gates. Each successful queue pop increments the guard before
`Vec::push`. On every normal nonempty completion, the guard releases the
existing `buffer.len() - initial_length` batch in one operation. If a push
unwinds, `Drop` instead returns the exact number of values popped during the
call. Thus allocation timing and normal batching are unchanged; the value whose
push failed is dropped, while the queue and bounded/unbounded accounting agree.

The permanent safe-API tests
`mpsc_poll_recv_many_push_panic_restores_bounded_capacity` and
`mpsc_poll_recv_many_push_panic_clears_unbounded_count` reuse
`vec![(); usize::MAX - 1]` and `catch_unwind`. The first of two popped values is
successfully appended and the second push unwinds. The tests assert that the
buffer retains that first value, the queue is empty, bounded capacity is
restored to two and reusable by subsequent `try_send`/`try_recv` pairs, and a
receiver closed after the unwind
returns `Ready(0)` rather than remaining Pending on a stale unbounded count.
The existing crate-local `mpsc_poll_recv_` filter includes both tests.

An independent source/accounting review confirmed the pop/push/bulk-accounting
ordering, the validity of the zero-sized-Vec probe, the bounded capacity loss,
and the unbounded stale-count terminal behavior. It also confirmed that a
successful Vec reallocation does not itself invoke `T::Clone` or `T::Drop`;
arbitrary value destruction and a destructor panic during the original unwind
remain separate agreed boundaries.

Production correspondence for `RecvManyGuard` is source-reviewed, and the safe
regressions execute a two-pop batch whose second push unwinds. The Verus
`RecvManyAccountingGuard` is a logical projection showing that a recorded
pending count can be taken once and composed with bounded
`MpscCapacity::receive_many` or unbounded `UnboundedAccounting::receive_many`,
including preservation of the unbounded receiver-closed bit. It is not a direct
refinement connection to the compiled guard, raw-list pop, standard Vec unwind,
or Drop execution. Allocation and arbitrary `T::Drop` execution remain agreed
foundational boundaries. S05 remains partial for the unrelated raw queue,
endpoint-overflow, Busy/parker, termination, and block-index residuals.

## Validation

The two permanent unwind regressions and all five module-local
`mpsc_poll_recv_many_` tests pass. The Tokio 1.52.3 Verus crate reports `546
verified, 0 errors` with both unwind-accounting witnesses. The target-local
`verify-all.bash` run completed successfully after the repair, including every
configured Tokio 1.52.3 Rust and loom target. Independent review approved the
runtime ordering, regressions, logical accounting model, and recorded proof
boundaries with no remaining findings.
