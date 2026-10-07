# Removal ledger — 2026-10-07

The user changed the target after the removal inventory and authorized removal
of the modified `bytes::verified` API and the identified high-rework alternatives.
The target returns to the upstream bytes 1.11.1 API and representation, retaining
the genuine Vec reverse-comparison bug fix.

The authorized source cleanup covers the modified implementation under
`src/verified/`, `src/verified_ownership.rs`, proof-only metadata in
`src/verification.rs`, and `src/ownership_proof/sequential_counter.rs` (also
unregistered from `src/ownership_proof/mod.rs`). The `verified` Cargo feature
and its `lib.rs` API/cfg wiring are removed. `BytesMut` is restored to upstream
behavior except for the retained reverse Vec comparison fix, implemented by
reversing the `BytesMut`-to-`Vec` ordering. The exact final source diff is
authoritative for which lines changed.

The 379-file `production-vec-owner-io-accessors-379` result and modified-variant
proof gates are retired from the current target. They remain historical evidence
and do not validate the restored source. Existing archival evidence and
counterexamples are retained; this cleanup does not delete them.

No new trusted contract or protocol axiom was added in this cleanup.
Temporary local contracts require strong reviewed assumptions and a concrete
small-removal path. They remain open proof obligations; generic physical/library
TCB is separately recorded.

The inventory did not establish whether trust for actual Clone/automatic Drop can be
removed with few changes; removability remains unknown pending source correspondence.

No full-crate proof of the restored target is claimed by this ledger.

## Exact cleanup and retained foundations

Both `src/bytes.rs` and `src/bytes_mut.rs` return to the flattened upstream
baseline `e678b774`; `bytes_mut.rs` retains only the reverse Vec ordering fix.
This removes the coupled frozen-reader/vtable instrumentation as well as the
raw SharedBuffer representation. The original Vec-backed Shared and native
AtomicUsize operations are restored. Dependent inactive source helpers
`carrier_protocol.rs`, `shared_reclaim.rs`, `shared_reserve.rs`, the finite
two-ticket `shared_protocol.rs`, and synthetic-lifetime `frozen_region.rs`
are removed.

The alternative proof launcher/installer is removed. Generic allocation,
capacity, base-address contracts remain in a separately scoped two-patch Std
installer; the alternative API's Copy-slot wrapper is retired from that installer.
Pure codec/arithmetic helpers and generic physical resource boundaries remain.
`retired-probes.json` prevents launching 26 historical adapters against source
whose representation and extraction markers were removed. Their saved evidence
is retained; those are not current-source proof targets. `ownership-frontier`
remains available as an independent generic boundary/control probe.

The pre-removal inventory, its source-line indexes, and source-parity receipts
remain an immutable snapshot of that earlier revision, not current API coverage.

Private alternative candidate workspaces (16, including the detached copy-split
worktree) were removed after moving their unique capture/evidence directories
and diagnostic logs to `/workspace/bytes-retired-experiment-evidence-2026-10-07`.
`private-candidate-removal.json` records those locations. This session-local
archive is supplementary; canonical committed evidence and Git history remain
the durable historical reference. The legacy/upstream comparison controls and
bytes toolchain were retained.

## Validation

Native std library/integration tests pass: 1009. No-default-features library
check passes. Serde + extra-platforms targeted tests pass: 2. These are runtime
checks, not formal ownership/concurrency proofs.

The final runtime verification entry was attempted with translation-only mode.
Creusot stops with an internal compiler error normalizing `Bytes`' DeepModelTy
for generic comparison traits. No Why3 proof phase was reached. The first
pre-final diagnostic is separately marked; final logs/source hashes and exit
statuses are in `cleanup-evidence-2026-10-07/manifest.json`. No workaround trust
or replacement model was added to conceal that boundary.
