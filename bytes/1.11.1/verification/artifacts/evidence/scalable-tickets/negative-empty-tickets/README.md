# Scalable affine ticket inventory

This probe uses the existing `Authority<FMap<Int, Excl<()>>>` and
`Fragment<FMap<Int, Excl<()>>>` APIs to replace one registration with two fresh
registrations. A monotonically increasing logical ID selects a singleton
fragment only after the coordinator's authority accepts that key. The ID does
not create ownership. Returned fragments compose with the active registrations
to reconstruct the authority's complete map.

The invariant-opening closure delegates the complete ticket-state transition to
`split_registration`. That named function preserves the full protocol and the
exact pending-map update while leaving Recovery and the retired physical pool
unchanged. The physical interval split remains outside the closure. The pure
`ledger_extend_fresh` lemma and the explicit map-insertion postcondition on
`issue_two_at_frontier` express the same resource conservation used by callers;
none of these operations is trusted.

The real B1 region is split alongside the ticket. Empty intervals remain
pending registrations. The positive caller replaces a root ticket at zero,
replaces the full interval at its far endpoint, retires the nonempty packet,
then returns both empty packets before B3 recovery. The negative configuration
tries finalization after all bytes have returned while both empty tickets are
still outstanding.

This is a sequential affine-ledger feasibility probe. It does not model native
reference counts, `BytesMut`, control-block access, arbitrary split-handle
selection, concurrency, or automatic `Drop`.

Run the target-scoped proof from the bytes crate root with
`./scripts/verify-bytes.sh scalable-tickets`. The negative configuration is
`./scripts/verify-bytes.sh scalable-tickets --features negative_premature_empty_tickets`.
Native coverage is `cargo test --offline --locked --manifest-path verification/probes/scalable-tickets/Cargo.toml --tests`.
