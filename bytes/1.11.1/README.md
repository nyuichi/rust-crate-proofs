# bytes 1.11.1 runtime verification

Under the verification cfg, `BytesMut::truncate` with `unique_valid` shortens
the logical length to `min(requested, old_len)` and preserves the old prefix,
pointer, capacity, storage tag and capacity slots. `set_len` with `unique_valid`
may expose initialized bytes up to capacity; it preserves the pointer, capacity,
storage tag and capacity slots. `len` returns the modeled length without a
`unique_valid` precondition. These are proofs from the original crate source,
including the local ownership and provenance helper bodies in their call cone.

The proofs do not establish that constructors or other public methods always
establish `unique_valid`, and do not cover Shared-backed or nonzero-offset
states, immutable peers, concurrency, escaping owners or unwind behavior. They
are not full API verification. See the [selected TCB](verification/results/direct-bytesmut-len-api/TCB.md)
and [generic Std overlay](verification/std-support/README.md) assumptions.

`./verify-all.bash` translates the live crate source and runs fresh, no-cache
proofs with the pinned bytes Creusot toolchain, default `std` features, native
atomic orderings, and one prover under the shared lock. Why3 needs elevated
execution. `./verify-all.bash --check` checks the saved COMA and prover results,
source/configuration identities, target inventory and translation diagnostics
without invoking a solver. A changed input is a reuse miss and requires a fresh
proof run.

The Root lifecycle result in `verification/results/current.zip` predates the
current source changes. It and `components.zip` remain historical provenance;
neither archive is used by the runner or checker, or represents current
implementation coverage. See the [saved-results notes](verification/results/README.md).

Source: crates.io bytes 1.11.1, archive SHA-256
`1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33`, revision
`417dccdeff249e0c011327de7d92e0d6fbe7cc43`. The original API and representation
are retained with this branch's runtime fixes. [MIT license](LICENSE).
