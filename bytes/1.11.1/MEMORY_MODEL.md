# Memory model design checkpoint

Established foundation: real Box slice -> full owned `Box<Perm<*const [u8]>>` ->
shared raw-pointer read -> Box recovery. Shared permission split is a borrowed
split, not independently owned BytesMut handles.

The deallocation probe requires full-allocation ownership, a nonempty allocation,
the allocation base, the actual suffix offset, and suffix length. Suffix length
zero at a one-past pointer is allowed. Its caller derives those requirements from
an actual nonempty Box. The bytes vtable/constructor/drop callers have not yet
been connected to those resources.

Next resource model must distinguish allocation identity/provenance/layout and
single deallocation authority, disjoint byte ranges, initialization per position,
read/write permissions, control-block ownership and in-flight refcount tokens.
Freeze affects one range, not the entire allocation. Bytes and BytesMut use
separate Shared implementations.

For weak memory, raw pointer Perm is non-objective. Resources stored in the
shared atomic invariant must use `AtView<Perm<...>>`; acquisition requires a
SyncView that dominates the stored view. Before claiming P3, demonstrate resource
accumulation through the actual RMW release sequence and final Acquire load.
Creusot 0.13's atomic wrapper currently lacks native `fetch_sub`; adding it must
preserve native subtraction/wrapping and original Ordering. Refcount and unique
recovery must remain bytes theorems, not new axioms.

P4 additionally distinguishes candidate control blocks from the byte allocation:
only the CAS winner transfers the payload resource; a loser frees only its own
candidate block and acquires the winner's publication. Address equality alone
is not provenance equality. Thread transfer and shared borrow obligations are
separate from merely annotating Send/Sync impls.
