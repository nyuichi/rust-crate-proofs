# Actual unique reserve and reclaim

This gate extracts the runtime `BytesMut` descriptor, unique reserve/reclaim,
split/access helpers and explicit consuming cleanup from `src/bytes_mut.rs`.
The build records every selected fragment. Its cfg branches carry affine
physical allocation resources; ordinary automatic Drop and whole-crate
dispatch are excluded.

The Cloud resume replay proved **120 files, zero unproved leaves** using
unmodified Creusot/creusot-std 0.13, nightly 2026-06-22, Why3 `54c92f96` and
why3find `eab37557`, native Ordering, sc-drf disabled, one prover and a 1024 MiB
limit. `evidence/cloud-baseline/positive120.tar.gz` preserves the generated
source and proof trees, with the log, summary and SHA256 sidecars adjacent.
Re-running the checkpoint `3e28a3b` build extractor in a scratch directory
produced byte-identical `actual_split.rs`. This reconciles the current replay
with the checkpoint; the older saved positive archive alone did not establish
that correspondence because later Shared edits changed selected fragments.

The replay proves unique reserve's fast path, reclaim and strict growth,
visible-byte preservation and explicit final cleanup. B1–B5 remain the
documented physical allocation/access TCB; the bytes ownership/refcount
protocol is not trusted. The separate growing-reserve probe retains stale
descriptor and partial-region rejection controls.

Run from this directory after installing the pinned tools:

```sh
bash verify.bash
```

Why3 execution requires elevated execution and uses the shared proof lock.
The separate growing resize/append replay passes124 files with zero unproved
leaves, with four native tests, under the unique-only cfg. It proves exact
resize/extend bodies, retained contents, newly initialized slots and explicit
cleanup. `evidence/cloud-growth/cloud-growth.tar.gz` contains the proof-time
snapshot and correspondence notes. The archived124 proof reports and preserved observed-result receipt are the
evidence.
Its terminal proof stdout was not captured, and a later source-only replay
confirmed exact generated source after removing two later Shared assertions.
The 120-file baseline alone does not establish this later caller increment.
