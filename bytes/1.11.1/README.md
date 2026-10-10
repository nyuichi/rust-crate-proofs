# bytes 1.11.1 runtime verification

Under the verification cfg, `BytesMut::from_vec` consumes a `Vec<u8>` and
returns an offset-zero `unique_valid` state with the Vec's pointer, length,
capacity and initialized contents. Its spare-capacity slots are modeled as
unknown. `BytesMut::with_capacity(n)` uses this path and returns length zero,
empty contents and capacity at least `n`.

`BytesMut::len` returns the modeled length without a `unique_valid`
precondition. For a `unique_valid` state, `truncate(n)` sets the length to
`min(n, old_len)` and preserves the initialized prefix, pointer, capacity,
storage tag and all capacity slots. Unsafe `set_len(n)` requires `n <=
capacity` and known initialized slots for any growth; it changes only the
logical length and preserves the pointer, capacity, storage tag and all
capacity slots. The original-source proofs include the local ownership,
provenance and capacity-representation helper bodies in their call cone.

These guarantees cover the selected offset-zero Vec path and normal-return,
sequential behavior. They do not establish `unique_valid` for other
constructors or transitions, and do not cover Shared-backed or nonzero-offset
states, immutable peers, lifecycle operations, concurrency or unwind behavior.
See the [selected TCB](verification/results/direct-bytesmut-len-api/TCB.md)
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
