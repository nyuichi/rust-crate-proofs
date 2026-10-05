# Adjacent Shared unsplit

The source adjacent_region adapter keeps the left ticket identity, retires only
the right ticket, and joins their adjacent physical regions. Empty intervals are
set-disjoint while their tickets remain real registered ownership obligations.
The source shared_unsplit wrapper performs the actual Release decrement, then
merges two BytesMut views with the same control and adjacent allocation ranges.
The actual caller makes three views, merges the first two, mutates/reads the
result, and explicitly releases the two survivors in either order.

The archived complete gate has 100 proof files with no unproved leaves, including
handle reconstruction and caller framing. Native allocator
tests cover every first/second boundary, zero/exact/spare capacity, and both
release orders, with exactly one original buffer/control cleanup and no realloc.

This is a sequential same-control adjacent merge gate. It does not infer common
provenance from equal numeric addresses, and does not cover fallback copying,
the fallback branches of public unsplit, automatic Drop effects, or concurrency.
No trusted bytes ownership-protocol function is introduced.

The archive audit was performed on 2026-10-05 against
`verification/cloud-handoff/session-evidence.tar.gz`; the checked manifest pins
all archived files. Counts concern `live-verif/shared-adjacent-unsplit/` and are historical
checkpoint evidence, not a fresh replay or whole-crate integrated proof. Current
source must still be replayed after shared contract changes. No negative-control
proof result for this gate was present in that archived directory.

Fresh public-method increment: the gate now extracts the actual cfg
`BytesMut::unsplit` method and invokes it through its public signature with an
explicit ghost coordinator. Its reviewed preconditions require same-control,
adjacent registered capacity regions and a full left view. Under this scope,
concatenation, sibling framing and explicit final cleanup passed 101 proof files;
the native allocator matrix also passed. Source, generated Rust, Coma, proof JSON
and logs are archived under `evidence/cloud-resume`, with per-file hashes.
The normal runtime method and the copying fallback remain separate proof paths.
