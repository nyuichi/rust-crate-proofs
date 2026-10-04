# Runtime source correspondence

`bytes.rs`, `bytes_mut.rs`, `buf/`, `fmt.rs`, and `loom.rs` remain byte-for-byte
identical to the published crate at this checkpoint. Changes to `lib.rs` remove
the old proof-only module substitutions and move the two unchanged arithmetic
bodies to `arithmetic.rs` with corrected/untrusted contracts. Ordinary builds retain
the published representation and algorithm. Proof dependency/CLI/toolchain changes
are recorded separately.

The helper probe compiles the exact runtime helper file. The storage-foundation
probe uses actual Box allocations and permission primitives, but is not Bytes.

The deallocation probe reproduces the actual `free_boxed_slice` body with only:

1. a ghost live-allocation witness;
2. a permission-aware wrapper erasing to native `offset_from`;
3. a token-consuming wrapper erasing to native `dealloc`.

`scripts/check-erasure.py` removes these reviewed substitutions and compares the
result to the source function body. Its hash result is recorded in
`verification/artifacts/deallocation-correspondence.json`. The compiler's
`#[erasure]` checking is an additional check. This source check is not a mechanical
proof of the entire compiler's erasure or of Bytes caller resource supply.

No SeqCst substitution, payload copy, permission fabrication, vtable replacement,
or alternate algorithm has been adopted for the runtime crate.

## Bounded adapter extraction

`src/bounded_ops.rs` is compiled into the normal runtime and included unchanged
by the `bounded-ops` proof crate. Take/Limit remaining and Reader/Writer transfer
sizes call `bounded_len`: the original `cmp::min` expression is unchanged inside
the helper. Take::chunk calls `bounded_chunk`, which uses that same minimum and
the original prefix slice operation. Take/Limit budget decrements call
`decrease_limit`, retaining the original subtraction after the underlying operation
and retaining the original preceding assertions. No trait contract, ownership or
Drop behavior is assumed to claim whole-adapter coverage. The proved helpers are
connected at these runtime call sites; generic adapter composition remains pending.
