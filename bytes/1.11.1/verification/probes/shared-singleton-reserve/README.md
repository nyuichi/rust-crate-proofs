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
no-copy spare capacity, and empty cases. The complete proof gate is being checked.
No new trusted helper is introduced here. The growth branch uses the separately
recorded physical B5 boundary; the ownership transitions and copy are body-proved.
This gate is sequential and explicit-release; automatic Drop/concurrency and
public reserve integration are separate gates.
