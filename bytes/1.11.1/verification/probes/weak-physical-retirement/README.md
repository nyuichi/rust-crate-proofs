# Physical buffer retirement through weak-memory publication

Fresh pinned replay proves all **36 files**. The protocol carries actual
`PhysicalRegion` partitions and the unique `Recovery` capability produced by
`detach_bound_vec`. Final retirement unseals the peer's resource only after the
stock Acquire fence, joins the physical regions, and calls the actual
`deallocate_bound_vec` boundary. Protocol bodies are proved; the generic native
atomic primitive and existing physical allocation boundaries remain explicit TCB.

The native matrix covers zero capacity, spare capacity, every partition endpoint
and both retirement orders. Exact source, Coma, proof JSON and logs are archived
with per-file hashes in `evidence/positive.tar.gz` and `positive.json`.

This is a two-ticket invariant instantiated with real buffer capabilities and
explicit cleanup. The native harness invokes both retirements sequentially.
It does not prove native Bytes Shared-cell retirement, dynamic vtables,
Send/Sync, arbitrary reference counts or automatic Drop.

The fresh `negative_no_acquire` control rejects `SharedRetirement::retire` at
11/12 goals: actual peer capabilities cannot be unsealed without the Acquire
fence. Source, proof and log are in `evidence/negative-no-acquire.*`.
