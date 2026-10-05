# Shared singleton reclaim transition

This gate extracts the actual coordinator-carrier BytesMut source and adds two
body-checked ownership helpers. It does not add trusted ownership contracts.

`singleton_region::recover` consumes the sole pending registration and its
coordinator, joins the current region with all retired regions, and recovers the
full original allocation. Its postcondition preserves every current packet
slot, including Unknown slots. This strengthens the usable interface over simply
composing retire/finish, whose public contracts do not expose slot preservation.
The helper is a child of the sealed scalable ticket module; its proof consumes
the entire inventory and never manufactures a physical allocation association.

`shared_reclaim::acquire` performs the existing native Acquire load under the
matching exclusive CounterOwn and requires the value one. It consumes the
registry into a lease containing the same native count-one authority, the exact
same typed Shared owner, and full physical allocation capabilities. It performs
no native decrement, buffer free, or control free.

`reactivate` registers those full capabilities as one root packet, retaining the
same count-one authority and typed Shared owner. The new registration inventory
can be used for subsequent splits and eventual explicit release.

The integrated caller performs actual from_vec, split_to, and release_shared of
one sibling; acquires and reactivates the remaining singleton; proves that its
visible contents were preserved; writes and reads a byte through the actual
access methods; and explicitly releases the final handle. The native allocator
matrix covers empty, zero-capacity, endpoint, interior, exact-capacity and spare
states. It observes no reallocations and exactly one original buffer free when
allocated plus one Shared control allocation/free.

Run `bash verify.bash` outside the sandbox (Why3 requires a Unix socket). The
script uses vanilla Creusot 0.13 and the existing shared proof lock. Run
`cargo test --locked` after sourcing the bytes proof tools for the native matrix.

This is preparation for actual Shared reserve/reclaim integration. The current
caller does not move the visible bytes or grow capacity. Automatic Drop,
concurrent reference counts, and ordinary public reserve dispatch remain outside
this gate. Existing physical and sequential-atomic primitive TCB is inherited.

Recorded validation: positive 123 proof files, zero unproved; negative 125 files, exactly one rejected singleton guard (25/26), despite both physical regions being empty. Native allocator matrix passes. The three-second solver limit closes the unchanged replace_at dependency; no source weakening was needed. Evidence archives, SHA256 manifests and logs are retained here.
