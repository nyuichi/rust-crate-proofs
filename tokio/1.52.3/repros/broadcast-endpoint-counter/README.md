# Broadcast endpoint counter ordering model

This is a reduced, deterministic model of an ordering concern in Tokio
1.52.3's broadcast endpoint bookkeeping. It is not a practical reproducer
against Tokio's public API.

The relevant production operations are:

- `Sender::downgrade` in `src/sync/broadcast.rs`: `num_weak_tx.fetch_add`
  happens before `Arc::clone`.
- `WeakSender::upgrade` in the same file: the successful `num_tx` CAS happens
  before `Arc::clone`.

The same ordering also appears in the first half of this model:

1. increment the logical endpoint counter;
2. pause every caller at that exact point;
3. create the corresponding owner token.

The production counter is `usize`. Reaching its wrap point would require an
astronomical number of simultaneously suspended calls (about `2^64` on a
64-bit target, or `2^32` on a 32-bit target). Threads, stacks, memory, or
address space would be exhausted first on real hardware. Private counters and
the pause point are also unavailable through Tokio's public API. Consequently,
this program deliberately substitutes `AtomicU8`; it demonstrates the abstract
schedule and the missing invariant, not an exploitable real-world overflow.

The bad model starts with 254 existing logical endpoints and owner tokens. Two
scoped threads increment the logical counter and wait at a barrier before
creating their owner tokens. A second barrier keeps them paused while the main
thread observes deterministic wrap from 255 to 0 and the mismatch with the
owner count.

The fixed model reverses the order. It reserves an owner token from a finite
pool capped at 255 before incrementing the logical counter. Of two competing
calls, exactly one gets the final token; the other is rejected before touching
the logical counter. This models the proof benefit of cloning/obtaining the
`Arc` owner before publishing the Tokio endpoint count. The pool is a small
stand-in for `Arc`'s finite reference-count/resource limit, not an exact model
of `Arc` allocation, abort behavior, memory ordering, or Tokio's close races.

Run without changing the Cargo workspace:

```sh
rustc --edition=2021 main.rs -o /tmp/broadcast-endpoint-counter-repro
/tmp/broadcast-endpoint-counter-repro
```

Expected final line:

```text
ok: bad ordering wrapped without owners; fixed ordering rejected before wrap
```
