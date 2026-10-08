# Guarded shared protocol core

This probe combines quota-free logical ticket registration with arbitrary-order
Release retirement and typed payload recovery. Logical ticket IDs are monotonic
and sparse: registration inserts the next ID, while retirement removes the ID
carried by its affine ticket. The protocol keeps the atomic count equal to the
live-map cardinality and bounds it by `MAX_REF_COUNT + 1`.

The generic event adapter commits one state transition only for a successful
guarded Relaxed RMW. A refused overflow observation makes no store and runs no
callback. Registration uses the successful old value; Release retirement
carries predecessor publication without acquiring it; only the last Release
followed by an Acquire can synchronize the `AtView` payload and recover the
full `LifetimeToken`. The native guard implementation is shared with
`src/ref_count_limit.rs` and audited separately from this Creusot protocol.

The default target is the strong generic lifecycle core. `Registry::new`
guarantees the initial ID is zero, public metadata equals payload metadata,
and the initial ticket's two lifetime fractions sum to one. The default proof
gate is `./run-proof.sh`; it serializes Why3 through
`/tmp/itoa-creusot-proof.lock`, uses one prover with a 1024 MiB limit, and keeps
`sc-drf` disabled.

`evidence/attempt-core-live-peer-v24.tar.gz` is the canonical captured proof
artifact for the current default source. All 37 generated `.coma` files and
37 `proof.json` files passed. The archive includes source and external inputs,
configuration, run log, and SHA-256 manifests; its adjacent `.sha256` file
verifies the archive. The v22 archive remains historical evidence for the
preceding core revision.

`evidence/attempt-core-live-peer-v24-core-std-inputs.tar.gz` links the proof
to the exact private `creusot-std` 0.13.0 source archive and its 110-file
SHA-256 manifest. The receipt records that every source-file mtime predates
the v24 proof run. Its adjacent `.sha256` verifies the sidecar archive.

`State::on_release_with_live_peer` composes a checked Release retirement with a
separately borrowed valid ticket. Fragment compatibility proves distinct IDs;
map-removal and callback postconditions prove the peer remains live and
recovery is absent. The helper makes no exact count claim.

The `driver` feature is default-off and its multi-clone sparse-hole scenario
remains an unproved proof challenge. The current `Registry::retire` API reports
whether the atomic event observed the last reference and returns typed recovery
conditionally; it does not carry a completeness receipt showing that a client
owns every live ticket. The driver does not claim a final last retirement until
a body-proved completeness capability is available.
The failed multi-clone driver attempt is preserved as
`evidence/attempt-combined-driver-v12-diagnostic-only.tar.gz`; it is diagnostic
evidence and does not count as positive coverage.

The `negative_core_lost_insert`, `negative_core_lost_removal`,
`negative_core_missing_acquire`, and `negative_core_duplicate_ticket` features
are proof-failure controls. Each runs as a separate attempt; a failure is useful
only when its captured translation and proof diagnostics identify the omitted
protocol obligation rather than a parser or build error. Against the v24
source, their immutable captures contain respectively 1, 5, 1, and 1 null
proof leaves; all four translated successfully and failed at their intended
obligations.
