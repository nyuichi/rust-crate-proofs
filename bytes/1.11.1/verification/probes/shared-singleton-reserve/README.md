# Shared singleton reserve

The exact from_vec/split_to/release source methods establish a live Shared
singleton. Source shared_reclaim acquires its complete allocation capabilities
without decrement or free. Source shared_reserve implements all three singleton
reserve branches: expose an already available capacity prefix; copy disjoint
visible bytes to allocation offset zero; or physically grow while preserving the
old offset and bytes. It updates the actual Shared buffer descriptor through its
exclusive Perm and reactivates a single registration for the resulting view.

Reactivation can retire an unused suffix in the ghost registry. The intermediate
two-ticket registry is never packaged into a ControlContext: the actual native
counter stays one throughout. This is ghost inventory reshaping, with no native
increment, decrement, or Release publication.

Six native allocator scenarios pass, checking copy, realloc, initial allocation,
no-copy spare capacity, and empty cases. The archived complete gate has 100 proof files with no unproved leaves.
No new trusted helper is introduced here. The growth branch uses the separately
recorded physical B5 boundary; the ownership transitions and copy are body-proved.
This gate is sequential and explicit-release; automatic Drop/concurrency and
public reserve integration are separate gates.

The archive audit was performed on 2026-10-05 against
`verification/cloud-handoff/session-evidence.tar.gz`; the checked manifest pins
all archived files. Counts concern `live-verif/shared-singleton-reserve/` and are historical
checkpoint evidence, not a fresh replay or whole-crate integrated proof. Current
source must still be replayed after shared contract changes. No negative-control
proof result for this gate was present in that archived directory.

Fresh resumption: the bounded doubling branch now falls back to the requested
capacity when doubling would exceed `isize::MAX`; the old `2 * capacity <= MAX`
precondition and caller bailout are removed. The modified complete gate passed
100 files with zero unproved leaves (`evidence/cloud-resume/manifest.json`), and the native
allocation matrix passed again. Public Shared reserve is a separate gate.
