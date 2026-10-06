# Modified variant: scoped explicit readers

The user selected public API/representation changes while avoiding large tool
changes. This production module replaces the initial lifecycle's implicit
Clone/Deref/Drop with internal Owner::new, share_pair, ReadHandle::read/close,
and consuming Owner::close. These helpers are crate-private: their ghost
preconditions are obligations for verified callers, not runtime checks. The
public admission surface is scoped_roundtrip and scoped_after_peer_close. Root-owned feature wiring selects this variant independently of
the legacy byte handles. The integrated bounded core and its two public callers are proved; this is
not a complete variant API claim.

Owner consumes a Vec through existing B1 and never retains a live Vec owner.
Existing FrozenOwner/FullBorrow keeps the actual physical region. A parent half
LifetimeToken anchors an actual B4 slice borrow; two affine quarter tokens and
unique close tickets accompany scoped readers over that same real byte slice.
No unsafe Send/Sync implementation or lifetime extension is added. Empty readers
retain their fractions and close tickets. Owner cannot move while its reader
borrows live, and cannot reclaim until its full synthetic lifetime is recovered.

The T02 weak protocol now also holds two stock authoritative agreement records.
Each Release retirement issues a receipt reflecting its actual RMW last flag.
The body-defined state relation ties each record to its deposited unique ticket,
and both completed records imply one final observer and withdrawn payloads.
Consuming context finalization checks the two receipt fragments against their
authorities. Parent cleanup can then select the recovered pair, join all three
fractions and call the existing B3 path unconditionally exactly once. The last
retirement may recover its peer fraction only after the native Acquire fence.

This initial witness is fixed-two, scoped, std, x86_64 and normal-return. It must
formally establish actual read values and the XOR final-observer postcondition;
native assertions are not substitutes. Arbitrary independent clones, mutable
growth, generic downstream trait laws, failure/unwind paths and the remaining
variant API/configuration inventory remain later obligations after admission.

The generic atomic primitive in atomic.rs is a byte-identical copy of the
previous weak-native-publication primitive, with unchanged Release/Acquire
semantics and explicit TCB. Physical primitives are shared canonical sources.
Stock scope/spawn/join, authoritative resources, synthetic lifetimes and AtView
remain library assumptions. No bytes-specific protocol theorem is trusted.
Preserve T01/T02 archives; retries of their rejected representations remain frozen.


The concurrent caller reads in both real child threads and closes both readers.
The ordered caller joins the first close before spawning the second reader, so
its actual read occurs after peer retirement while its own obligation remains
outstanding. Both formally return the input byte (or None outside bounds) and
opposite last-observer flags. The consuming cleanup body makes one B3 call on
normal return; positive-capacity storage is deallocated once, while capacity zero
has no allocator block to release.

Validation: the production `verified,std` gate proves 47 files. Native coverage
includes 96 lifecycle cases (48 per caller) covering empty input, spare capacity,
read bounds and reversed ticket roles, plus the unchanged atomic race test.
Missing Acquire rejects the peer AtView synchronization precondition. Abandoning
both empty-reader obligations (input length zero, arbitrary capacity) rejects
LifetimeToken::end's full-fraction precondition. Reusing one Closed result twice
is rejected by Rust E0382 before VCs. Exact positive/failure/negative sources and
proof trees are in verification/modified-variant-evidence/runs; the controls are
archived variants and are absent from this production source.
