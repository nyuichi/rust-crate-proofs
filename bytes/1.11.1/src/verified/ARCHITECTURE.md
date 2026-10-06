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

## Finite scoped sharing and exclusive mutation

`scoped_tree` extends the fixed pair to every finite `usize` leaf count at least
2. It uses one local pair counter per internal node, rather than one flat global
refcount. A consumed handle of fraction f becomes a retained f/2 anchor and two
f/4 children. The stack-owned BranchContext retains the original parent ticket;
children carry no Rust reference to that context. After child joins, consuming
finish checks both receipts, restores exactly f, and returns a handle accepted
by the original parent context. The worker then retires upward with its own
spawn-provided Tokens, which stayed in that thread while its children ran.

The recursive worker proves the requested read count and actual input byte for
arbitrary counts, while each split proves positive, strictly smaller child
counts. The scope remains normal-return correctness: stock thread contracts do
not promise scheduler termination, and no total-termination claim is made.
Counts below 2 return None after explicit B1/B3 cleanup, including empty storage.
The missing-anchor control rejects the parent acceptance postcondition, so two
child quarters cannot silently replace their original full parent fraction.

`exclusive::ExclusiveBytes` retains one ordinary Vec owner for checked mutation.
Its selected method bodies are verified against the stock Vec contracts.
`scoped_set_and_read` proves a checked set, then consumes that Vec into the same
B1/physical-region/tree-read/cleanup chain. No ordinary Vec owner coexists with
its detached physical ownership. Reserve methods currently prove preservation
of byte contents; their native capacity guarantees are not yet in the formal
contracts. This limitation and the remaining API/configuration inventory prevent
a whole-variant completion claim.

The integrated gate proves 67 files with no null proof leaves. Native coverage
has six test functions: the original 96 lifecycle cases, 180 tree cases (including
rejected counts and non-power-of-two trees), 135 mutation-to-tree cases, exclusive
sequence/explicit-close checks, and the unchanged atomic race test. These finite
native cases supplement the symbolic all-counts normal-return proof.
