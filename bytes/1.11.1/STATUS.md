# bytes 1.11.1 runtime proof checkpoint

## Selected modified target (2026-10-06 UTC)

The user selected route 1: public API and representation changes are authorized,
large verifier changes are avoided, and the final target is the explicitly named
`bytes::verified` variant of bytes 1.11.1. The original Bytes/BytesMut API is not
claimed fully verified. Read [MODIFIED_VARIANT_SPEC.md](verification/MODIFIED_VARIANT_SPEC.md),
[ARCHITECTURE_DECISIONS.md](verification/ARCHITECTURE_DECISIONS.md) and
[ARCHITECTURE_ASSESSMENT.md](verification/ARCHITECTURE_ASSESSMENT.md) before work.

Bounded architecture admission PASSED. Subsequent production increments prove
recursive scoped sharing, ordinary exclusive mutation connected to sharing,
checked numeric Cursor operations and reusable immutable scoped callbacks.
The latest production run, production-finite-cursor-std-287,
proves 287 files with zero null leaves and native tests pass 48 std/portable
and 22 alloc-only. It integrates checked copied splits/iterator append, concrete
IO, capacity/error frames, numeric address framing through physical recovery,
deterministic digest/hex and Copy-restricted fallible scoped-slot cleanup. The
exact actual Std dependency sources and feature graphs accompany the capture.
Proof-file counts are not API-completion counts. Read
[MODIFIED_VARIANT_PROGRESS.md](verification/MODIFIED_VARIANT_PROGRESS.md).

The complete modified variant remains NOT COMPLETE. The small generic capacity,
numeric base observation and Copy-slot contracts are reviewed trusted Std
boundaries. The bytes-specific sharing, refcount, retirement, recovery and
explicit cleanup bodies remain proved. Numeric address equality is not pointer
provenance or permission. The address client explicitly closes its returned Vec.
Universal totality is not inferred from metadata or finite-computation proofs.

Actual native floating conversion, generic Serde, callback unwind, unrestricted
spawn error-frame, and generic Hash/Formatter failures remain preserved and
frozen in D08–D13. Changed APIs are separately admitted by their actual evidence;
frozen failures never count as completed responsibilities. Final complete-source
configuration gates, the final API correspondence inventory and downstream/control gates remain open. Finite owned-cursor computation is now
checked terminating; normal-return cleanup and arbitrary callbacks/iterators
carry no unconditional totality claim.

Frozen approaches remain frozen unless their actual premises change. T02 remains
a concrete threaded transport/conditional cleanup component (37 proof files),
not eventual exactly-one completion, arbitrary sharing or actual Bytes Clone/Drop.
The admission dependency is now closed within its documented scope; subsequent
work must preserve this production representation and its proved protocol.


## Cloud resume results (2026-10-05)

This session resumed `3e28a3b1` on `bytes-runtime-verification`, audited the
archived handoff, rebuilt pinned stock Creusot0.13/Why3/why3find and matched
source extraction to saved evidence. `verification/CLOUD_RUNTIME_2026-10-05.json`
records the tool binaries. Results below are scoped gates, not whole-crate
completion. Earlier checkpoints below remain historical.

Validated and committed: actual cfg unique reserve/reclaim120, growing unique
resize/append124, singleton Shared reserve100, adjacent same-control public cfg
unsplit101, frozen receiver/read/recovery64, concrete slice iterator22, comparison
readonly trait122, UninitSlice core8, carrier checkpoint replay117, weak generic
publication3 and fixed-two-ticket physical retirement36. These counts are
proved files; helper, source-sliced trait and restricted cfg scopes differ. Each
probe README and archived source manifest states the admitted configurations.

The native Vec-vs-BytesMut PartialOrd reversal was reproduced and fixed with a
regression test. Native crate tests, including docs, currently pass1256; no_std
check passes. Native observations establish no affine or concurrent protocol.

Readonly immutable B4 classification passes34 and rejects erased mutable writes
and a stale-read claim. Production BytesMut immutable purity now translates;
mutable B4 remains non-ghost. The default-source readonly extraction still
cannot prove AsRef's initialized-authority precondition without a handle
invariant. Mutable Deref cannot be classified ghost: the preserved
mutable-B4 diagnostic is a semantic counterexample.

Actual integrated translation is still incomplete. The latest frontend attempt
reports Bytes unsafe Send/Sync requiring trusted markers, default Bytes Deref
purity, and BytesMut DerefMut purity. The unmodified compiler mandates trusted
unsafe marker traits; no bytes marker/protocol was annotated trusted to pass.
Fresh actual Clone dispatch fails unsupported function pointers; shared-reference
affine fraction splitting also fails mutable-borrow requirements. The finite
private Clone-enum translation feasibility does not integrate actual Clone or
its atomic invariant. `verification/probes/native-integration-frontier` preserves
concrete diagnostics and Astra's reviewed limits. Automatic Drop, general native
Shared/vtable concurrency, arbitrary public trait implementors, and panic/unwind
cleanup remain outside the current proof.

Exact public cfg Shared reserve now passes106 files (105 plus a body-proved
initialization-coordinate helper), including singleton and
nonunique sibling callers; bounded Shared no-allocation try_reclaim passes108 with512 native cases and rejects allocation identity changes. Checked unique advance and its RawTransition helpers
pass3. Requested-capacity construction replays101. Shared growing resize/append and callers pass115; capacity-only no-op framing and final registration matching are body-proved contracts. Vec/BytesMut semantic comparison traits pass103 alongside a fresh122-file readonly baseline. Same-control public cfg
unsplit empty/adoption/copy fallbacks pass115; independent-control fallbacks
pass114, including singleton final retirement. Mixed Unique/Shared and both-Unique public cfg gates pass121 each, including canonical empties and advanced Unique views. The adjacent gate remains separate.
UninitSlice projection and all six range-index variants pass12 per selected gate
after adding a prophetic caller condition preserving already
initialized bytes. The unsafe deinitialization counterexample against the old
contract is retained; the repaired contract rejects it at the call precondition.
The bounded raw UninitSlice constructor passes42 with the existing B4 authority;
its wrong-pointer control leaves exactly the pointer-match requirement unproved.
The ordinary bare-pointer signature is excluded. The exact BytesMut len/is_empty/reference-caller selection passes3; Bytes
len/is_empty metadata bodies pass2. Nonpromotable Bytes truncate/clear and callers pass6 under an explicit table-tag precondition; the native classifier regression checks all six table sites. Exact new/from_static constructor attempts still fail function-pointer/static-pointer translation. Concrete mutable-slice BufMut bodies pass16,
with unsafe initialization promises and public trait laws excluded. Pure
Take/Limit/Chain/Reader/Writer field projections and callers pass28. These
proofs do not discharge automatic Drop or general Shared concurrency.

## Pre-resume checkpoint (2026-10-05)

Vtable source refactor: private clone callbacks now receive their actual current
vtable. Static/Owned clones preserve that table instead of referencing its
initializer, removing both previous translator cycles without trusted dispatch.
Native118/no_std pass. Full translation now reports unsafe Send/Sync marker
requirements and Deref ghost-purity errors (5 errors), not a whole-crate proof.
See verification/probes/vtable-leaf-integration/logs/runtime-passed-vtable.log.
The earlier vtable-cycle checkpoints below are historical.

Unique cleanup connection: ordinary BytesMut Drop's KIND_VEC branch and
explicit consuming fresh/advanced unique cleanup now share exact extracted
`release_unique_storage`. It recovers original base and capacity from the
packed offset and consumes full affine authority before B3; no initialized
prefix is fabricated by rebuilding Vec. Consuming unique and legacy shared
entries forget the handle before cleanup, retaining descriptors separately.
Current gates: carrier116, legacy split/storage111, safe traits73, public
constructors62, zero unproved in each. Both current carrier negatives118 reject
exactly one intended leaf. Ordinary118/no_std and native carrier7, legacy21,
traits1, constructors3 pass at recorded checkpoints. Evidence is
`unique-cleanup-carrier-positive`, `unique-cleanup-ticket-negative`,
`unique-cleanup-unknown-negative`, and `explicit-cleanup-regressions/*`.
Both ordinary destructor branches now use cleanup routines checked through
explicit consuming proof paths. Automatic Drop dispatch is still unverified.
The carrier extraction omits BytesMut Drop: its additional proof arguments
do not establish whole-crate compilation under bytes_proof_repeated_split.
Astra reviewed original-base cleanup, zero capacity and forget-before-release.
Final whole-crate translation was attempted with BYTES_TRANSLATE_ONLY=1.
It remains blocked by the two previously recorded Bytes vtable recursion
cycles (static_clone/STATIC_VTABLE and owned_clone/Owned::VTABLE). No whole-crate
proof was run or claimed. Log: verification/artifacts/logs/
explicit-cleanup-runtime-frontier.log.
Luna xhigh added four native ordinary-public-API allocator tests: advanced
unique cleanup after spare writes, endpoint splits in all six three-handle
drop orders, and unique/shared capacity zero. Pointer/size/alignment logs
check exactly one original buffer free when allocated and one Shared control
free when promoted; no reallocations. The test is registered in Cargo.toml and
passes through the actual crate manifest. These observations do not prove
automatic destructor effects. Bound-constructor regression32 also passes.

Actual shared-release connection: carrier explicit cleanup now calls the exact
cfg-adapted `release_shared` body extracted from bytes_mut.rs. The existing
affine retirement/full recovery proof moved into that body; native and proof
configurations share the nonfinal test, final acquire position and descriptor
disarming flow. Ordinary final release directly frees the buffer before
Box control destruction; its SharedBuffer destructor sees capacity zero.
Positive gate: 115 proof files, zero unproved. Native matrix: 7; ordinary
test_bytes: 118; no_std: pass. Existing missing-empty-ticket and Unknown-byte
negatives each retain one intended failure (117 files). Evidence prefixes:
`connected-release-positive`, `connected-release-ticket-negative`,
`connected-release-unknown-negative`. Astra reviewed cleanup ordering and scope.
This is a verified sequential adaptation of actual release_shared, not
cross-thread synchronization or automatic destructor invocation. Existing
trusted ownership contracts are unchanged; B3's reviewed native implementation
now includes the common allocator leaf.

Explicit-cleanup continuation: ordinary SharedBuffer cleanup and both existing
B3 deallocation variants now use one native `allocation_ops::deallocate_u8`
leaf: direct global deallocation with byte layout, with no allocation at capacity
zero. This replaces the prior Vec reconstruction/destructor dependency without
adding trusted clauses. Carrier consuming release suppresses handle automatic
cleanup before entering the protocol. Current positive gate: 114 files, zero
unproved; native allocation/mutation matrix: 7 tests; ordinary test_bytes: 118;
no-default-features check: pass. Evidence is `explicit-cleanup-positive`.
This connects the physical buffer-deallocation leaf, not yet original
release_shared control flow or automatic destructor invocation.

A read-only Luna xhigh Verus audit found upstream PAtomic load/add/sub use
hard-coded SeqCst. Stock Verus does not express the unchanged native
Relaxed/Release/Acquire sequence; a custom weak-memory model remains necessary.
See `verification/VERUS_REFCOUNT_FEASIBILITY.md`. No Verus proof is claimed.

The coordinator-carrier gate passes 114 proof files with zero unproved leaves.
Exact cfg-adapted from_vec, promotion, ARC shallow_clone/native increment,
repeated split_to/split_off/split, mutable/read/spare access, set_len,
advance_unchecked, truncate/clear, in-capacity resize/extend_from_slice,
reserve/try_reclaim unchanged fast paths, and explicit release compose through
one scalable affine registry. Full capacity Known/Unknown slots and discarded
prefixes remain owned; final recovery consumes the original full allocation
before B3 buffer deallocation and typed Perm control-block destruction.

Explicit coordinator transfer to an initialized matching sibling is body proved.
The interleaved lifecycle retires the middle handle (native count 3 -> 2), moves
the sole coordinator into the oldest sibling, splits it again (2 -> 3), mutates
all survivors and releases them in all six orders. Retired regions survive the
re-split; old creation snapshots still match the current pending inventory;
monotone fresh ticket IDs prevent recycling when native count grows again.
The split_off target includes at > len, empty endpoints and capacity zero.

Seven native tests check boundary values, all survivor retirement orders, zero
reallocations, one original buffer free when allocated, and one control-block
allocation/free. Separate 116-file negatives each reject exactly one intended
guard: missing-empty-ticket final release (17/18) and Unknown-byte publication
through actual set_len (16/17). Final-source regressions pass legacy split/storage
110, safe traits72 and public constructors61 proof files, all with zero unproved
leaves. Native legacy21/traits1/constructors3 and ordinary test_bytes118/no_std
pass at their recorded checkpoints; final ordinary118/no_std logs are saved.

Latest evidence: coordinator-carrier-split/relocation-positive,
relocation-ticket-negative, relocation-unknown-negative, and regressions/
relocation-final, traits-relocation-final, constructors-relocation-final.
Earlier source/extraction/VC/log/hash snapshots remain frozen. The spare-prefix,
coordinate, ticket/map-conservation and finalization helpers have proved bodies;
no new trusted protocol/physical-access clause or core/std change is introduced.
Luna xhigh prepared native cases and audited scope; Astra reviewed proof
interfaces, split_off, capacity paths and coordinator relocation.

Remaining boundaries: whole-crate release/Drop caller integration and
ordinary/proof ownership interpretation (ordinary Shared already uses raw
SharedBuffer, not a live Vec); automatic scope-exit Drop effects; concurrent RMW/release-sequence/
view synchronization; Bytes/freeze read-sharing and vtable dispatch; and
storage-moving/growing reserve/reclaim/unsplit. Unmediated sibling API calls still
lack the explicit coordinator required by this adapter. Requested-capacity
constructor guarantees are not derived from the unchanged pure-Seq Vec model.
These require separate ownership/tool designs. The whole bytes crate is not
proved; this checkpoint establishes the restricted sequential cfg lifecycle.

The complete concrete slice Buf body gate passes 82 files with zero unproved
leaves and three native matrices: 19 checked integer readers, 19 normal-return
getters and remaining/chunk (40 bodies). Variable unsigned reads allow 0..=8
bytes. 19 native getters use equivalent match syntax to avoid the absent
Result::unwrap_or_else contract; ordinary test_bytes118 and no_std pass. The
83-file wrong-available negative rejects one intended leaf (1/2). Getter input
and width preconditions exclude panic/unwind; exact extracted panic bodies are
uncallable from proved paths and add no trust. Trait dispatch/refinement and
full-crate proof remain excluded. Evidence: slice-buf-overrides/complete-*.

The scalable sequential ticket/physical-region protocol passes 45 proof files
with zero unproved leaves and one native matrix. It supports repeated affine
replacement, exact dynamic pending entries, fresh monotonic IDs, disjoint
regions, returned/active fragment conservation, and full original B3 recovery.
The 46-file negative rejects exactly finish's pending-empty guard (10/11) when
all bytes have returned but two empty-region tickets remain. All protocol and
map-conservation helper bodies are proved; no TCB is added. This prototype does
not yet connect actual BytesMut split, native refcount or automatic Drop.
Canonical evidence is under scalable-tickets.

Concrete slice Buf checked-read bodies now pass a 55-file gate with zero
unproved leaves and two native matrices: try_get_u8 plus signed/unsigned
16/32/64/128-bit readers in both endian orders (17 methods). The exact source
body is adapted only by inverse-checked receiver substitution, with success
value/suffix and failure requested/available/unchanged-input contracts. The
56-file wrong-available negative rejects exactly one intended leaf (1/2).
This is body evidence, not Buf trait dispatch/refinement or full-crate proof;
no production source or trusted boundary changes. Evidence: slice-buf-overrides.


Advanced safe-trait integration after unique advance was attempted and stopped
at one current-state type-invariant obligation (advance_unchecked 27/28;
77-file diagnostic). Simplifying the unique-only predicate and computing the
next descriptor in locals did not discharge it. Native trait matrices2,
ordinary tests118 and no_std passed, but no advanced-trait gate is claimed.
The attempted changes were reverted to the last proved source; exact failed
source/VC/log evidence is under valid-handle-traits/advanced-invariant-blocker.
No new trust or core/std modification was introduced.


Actual public BytesMut::zeroed and core From<&[u8]> constructor bodies pass a
60-file exact-source gate with zero unproved leaves and three native fixtures.
Contracts establish unique validity, exact length, and pointwise Known zero /
copied bytes. The real From impl and a generic trait caller are included.
No capacity promise is inferred from Vec::with_capacity, and no trusted helper
is added. Automatic Drop and the full crate are excluded. Evidence:
public-bytesmut-constructors/positive.


The actual private advance_unchecked Vec branch is now body proved under the
packed-offset bound, in the 109-file lifecycle gate with zero unproved leaves
and twenty-one native tests. Two advances preserve owned slots and low metadata
bits, shift the visible suffix, support mutation/spare initialization, and retain
the complete original allocation. Explicit unique cleanup derives the original
base with retreat and consumes original full-capacity resources through B3.
The ARC branch additionally requires a matching affine registration, not just
range metadata; this supports its native live-allocation pointer arithmetic.
No Shared control allocation is needed. Promotion when the offset exceeds
MAX_VEC_POS, public Buf trait integration, automatic Drop, and advanced safe
trait dispatch remain excluded. The advanced Unknown-publication negative
rejects exactly one Known-prefix guard among 110 files (14/15), after establishing
full unique ownership and the new visible Unknown byte. Evidence:
sequential-bytesmut-split/unique-view-advance, with registered-positive as the
final stronger gate. A second 110-file negative rejects exactly the allocation
registration/ownership guard (10/11) when a metadata-only ARC descriptor copied
before B3 recovery is reused afterward. Descriptor geometry does not grant
allocation liveness.


The derived pointer-base recovery primitive gate proves 32 files with zero
unproved leaves and one native roundtrip fixture. BoundPtr's lower-address
invariant follows from the existing NonNull base; advance and retreat preserve
it. The body-proved retreat returns sealed offset-zero metadata without
creating permissions; the caller recovers through B3 using original full
capacity/resources. No trusted fact/spec is added. This is not yet actual
unique BytesMut advancement. Evidence: bound-pointer-offset/retreat-positive.


The in-capacity runtime update gate proves 103 files with zero unproved leaves
and nineteen native tests. Actual reserve's fast branch preserves the handle;
actual resize and extend_from_slice use the same body-proved MaybeUninit fill /
copy loops in ordinary and proof builds. Their contracts preserve storage
identity, initialize exactly the written interval, frame all outside slots,
and preserve the retained split partner. Explicit recovery is proved in both
release orders. Reserve growth/reallocation is excluded by an explicit capacity
precondition. Native allocator checks observe no buffer allocation/reallocation
for these updates. Ordinary test_bytes passes 118 tests and no-default-features
build passes. No trusted bridge is added. Frozen evidence: sequential-bytesmut-split/noalloc-update/positive.


The independent valid-handle-traits gate proves 70 files with no unproved leaves
and one native fixture matrix. It extracts actual core AsRef<[u8]> and
AsMut<[u8]> implementations, including implementation refinement and generic
trait callers. Its checked BytesMut invariant admits initialized unique
ownership and canonical empty handles only; Shared registrations, promotion,
and split transitions are excluded. This is not an integrated split-to-trait
proof. The unregistered safe-AsRef negative has exactly one unproved leaf among
71 files (5/6 in its caller). Frozen evidence is under valid-handle-traits.
Final-source regression also passes the explicit-precondition lifecycle gate
(93 files/native16) and actual_from_vec constructor gate (56 files). These remain
separate configurations; evidence is under their trait-scope-regression paths.
Three fixed-zero physical reference bridges are added to the explicit TCB;
these grant no byte access, allocation liveness, or ownership. Nonzero full
buffers retain the correct spare endpoint pointer. Deref/DerefMut remain
blocked by standard ghost-purity requirements, recorded separately.


The unified unique-at-offset-zero / Shared access gate passes 93 proof files
with no unproved leaves and sixteen native tests. Actual as_slice, as_slice_mut,
spare_capacity_mut, truncate, clear and set_len use the constructor's full unique
Recovery/Region or the matching Shared packet. Unique writes preserve Recovery;
no second detach or Shared allocation occurs. Explicit unique cleanup requires
full ownership and offset-zero metadata, and safely frees Unknown spare slots
without reading them. Access/publication still require Known bytes. B4-bound's
canonical Known postcondition is logically redundant with its exact slot-value
postcondition; no new physical trusted primitive is added. Canonical evidence
is under unique-access/positive. Unique advance and an invariant covering
Shared transitions remain unproved; the separate unique-only trait gate is above.


The storage-ops helper gate passes four proof files (nine discharged goals,
no unproved leaves) and four native tests. The ordinary fill/copy loops operate
on MaybeUninit<u8> prefixes, establish Known values, preserve destination length,
and frame the untouched suffix. No trusted contract is added. Runtime resize /
extend integration is subsequent work; these isolated helpers are not yet a
proof of either public method. Canonical evidence is under storage-ops/positive.


The read-only sequential gate passes 90 proof files and fourteen native tests.
Actual as_slice and its packet wrapper support two simultaneous shared reads
while a disjoint split packet is mutated. B4-read is an explicit physical slice
bridge; the read wrapper and callers are body proved. The Unknown-read negative
has exactly one rejected obligation among 91 files (17/18 in its caller).
Canonical evidence is in readonly-access. Actual AsRef/Deref/Buf/BufMut trait
integration is still pending: safe trait preconditions require a valid-handle
invariant covering unique as well as shared states. No whole-type invariant or
trait-dispatch proof is claimed by this checkpoint.


The preceding bounded sequential lifecycle gate passes 86 proof files and twelve
native tests. Actual spare_capacity_mut connects to standard MaybeUninit<u8>
initialization through reviewed B4-uninit. Callers initialize Unknown spare
slots on both sides, publish via actual Known-only set_len, access initialized
bytes and explicitly release both orders. Truncate -> uninit -> rewrite ->
republish is proved; an 87-file negative rejects exactly one set_len Known
prefix guard (16/17) after re-uninitializing a retained byte. Canonical sources,
configuration, proof code/results and native logs are in spare-initialization.
The only new trust is the u8 physical reference/slot-interpretation bridge.
No Creusot compiler/std changes or trusted ownership protocol are added.

The concrete sequential milestone is complete. Whole-crate verification is not:
automatic destructor effects and Bytes vtable translation remain upstream
blockers. Concurrent native ordering requires a synchronization/interference
model beyond exclusive CounterOwn; repeated ARC splits require a scalable
registration protocol beyond the two-ticket gate. Buf/BufMut trait dispatch,
reserve/reallocation and freeze/Bytes remain outside this ownership extraction.
Those exclusions must not be interpreted as failed numeric VCs or verified APIs.


The retained-prefix actual advance_unchecked gate passes 79 proof files and
nine native tests. Handles keep the full original affine packet while their
live view becomes an interior suffix. The body-proved packet-borrow adapter
accepts that interior view, without changing B4 or the physical TCB. First split
contracts still expose exact initial pointer bindings at offsets 0/at. The
caller mutates remaining Known bytes, proves discarded prefixes unchanged,
and explicitly releases both orders to recover all original capacity. The
claim covers the actual internal method, not the Buf trait entrypoint, whole
crate or automatic Drop. An 80-file negative rejects exactly one Known-prefix guard (19/20)
when actual set_len grows after advancing into Unknown spare capacity.


The bounded actual split_off / length gate passes 77 proof files and seven
native tests. Actual split_off preserves all slots, including Unknown spare
capacity, and proves pointer/length/capacity partition for at<=capacity. The
probe harness clamps requests to constructed capacity because Vec->Seq does
not model reserved capacity; native tests supply valid points so no clamp
occurs. Actual set_len allows only a capacity-bounded Known new prefix.
Truncate/clear preserve full packet resources and all slot values; restoring
retained Known bytes is proved. A 78-file negative rejects exactly one actual
set_len guard when growing into Unknown. Allocator tests observe zero reallocs
and exact A/S frees in both orders. No new TCB is added. Actual constructor
with its strengthened spare-Unknown postcondition freshly passes 54 files.
Automatic Drop, concurrent release and repeated ARC splitting remain excluded.


The actual split/mutable-view gate now passes 68 proof files. Exact as_slice_mut
uses the matching affine packet through a body-proved borrow helper and the
explicit B4-bound physical access bridge. Callers hold both disjoint slices
live, write distinct values, prove both ledgers updated, and explicitly release
in either order. Native tests (4) cover simultaneous mutation, empty views and
A/S allocation events. Known slot structure has a body-proved equivalence to
the existential initialized-value definition; no initialization condition is
weakened. Proof-mode Send/Sync impls are disabled; concurrency and automatic
Drop remain outside this sequential gate. Each of four negative configurations rejects exactly one intended guard
(69 proof files): Unknown access, Pending access, stale contents and missing
empty ticket. Fresh actual constructor passes 54 files, pool passes 32 files,
and ordinary native test_bytes passes 118 tests.


The first actual BytesMut split gate passes 61 proof files: exact from_vec,
promote_to_shared(2), shallow_clone, split_to and explicit consuming release
execute with affine buffer regions and registration tickets. Both release
orders are proved. Native tests (2) observe the matching A/S frees. The
62-file missing-empty-ticket negative rejects exactly one finish guard.
This covers one split from unique offset zero; independent mutable access,
automatic Drop and concurrency remain outside this checkpoint. The generic
Box alignment bridge adds one explicit physical TCB fact; no Creusot compiler
or std modification is used. Canonical evidence is sequential-bytesmut-split.


The restricted actual-Shared control gate passes 45 files. Exact source helper
bodies allocate a real Shared box, join physical retirement/tickets to the
native sequential counter, branch on the native final decrement, recover all
buffer regions, disarm SharedBuffer, B3-free the buffer and call standard
Perm::drop on Shared. No bytes ownership protocol is trusted. Two native tests
pass, including allocation events for both release orders; nonzero capacity
observes one S allocation and two A/S frees. A missing empty-region ticket
rejects exactly one finish guard (46-file negative configuration).
That earlier control-only gate is the source's restricted helper block, not the existing from_vec,
promote_to_shared, split_to or release_shared path. Automatic Drop and
concurrency remain unproved. See the sequential-shared-control probe scope.

BoundPtr now preserves sealed allocation binding through a body-proved bounded
advance. B1's existing trusted allocation invariant is strengthened with the
numeric nonwrapping address extent; no new physical trusted function is added.
Only numeric metadata and offsets are proved here, not a native provenance
theorem. The fresh constructor gate passes 42 files and the physical pool gate
passes 31 files against this core (extra offset/provenance helper files account
for the increases from 41/25).


The sequential native-counter bridge now has two proved caller files and one
native boundary-value test. Wrong-identity and underflow configurations each
reject exactly one guard. Its four primitive methods are explicitly trusted:
this is additional scalar-atomic TCB, not a proof of concurrent refcount or bytes
ownership. See `sequential-native-counter-manifest.json` and TRUSTED_BASE.md.
No compiler or standard-library change is used.


The native BytesMut Shared now retains a single raw SharedBuffer (base/capacity),
not an ordinary Vec. Promotion, unique reserve, both Vec conversion paths and
shared-to-mutable conversion use that descriptor. Taking a Vec empties it before
control-block destruction. Raw reallocation preserves spare bytes without
asserting an initialized prefix across split_off gaps; allocation/layout failure
leaves the descriptor unchanged. Three focused native regressions pass, including
reuse after a caught overflow panic. This runtime preparation does not body-prove
promotion, reserve, refcount or Shared destruction. The constructor extraction
also includes the exact new SharedBuffer type; its body scope remains unchanged.


The sequential retirement-registry gate now proves 27 files with real affine
registration tickets, B1 physical regions, NonAtomicInvariant::open_mut, both
retirement orders and explicit B3 cleanup. Two native tests pass. A missing
empty-region ticket rejects exactly one finalize guard despite full byte
coverage. No new trusted declaration or Creusot/creusot-std edit was added.
This is a unique mutable coordinator, not native Shared/refcount/Drop. See the
[registry scope](verification/probes/sequential-retirement-registry/README.md).

The sealed retired-region pool core gate proves 23 generated files, including
empty seed, affine retirement, full-coverage extraction and both explicit
retirement orders followed by B3 deallocation. These are genuine B1 physical
resources; no new trusted function was added. The gate does not contain native
Shared, reference counting, or automatic Drop. The extended gate now proves 25 files and passes two native tests, including
independent B4 changes followed by pool recovery and B2 Vec restoration.
The genuine half-region path rejects one coverage leaf; the sealed-input
namespace API control rejects two matching leaves (not a two-allocation
construction proof).
A fresh full-runtime translation after this core change again stops on only
the two known static/Owned vtable recursion cycles; no integrated runtime
proof was produced (`runtime-retired-pool-translation.log`).

A fresh exact-source constructor/explicit unique-at-zero cleanup gate now
passes 41 proof files with the current physical-pool and cap-class source,
and the native extracted-constructor tests pass 3/3. No mixed kernel snapshot
is needed for this gate. See `bound-constructor-current-pool-manifest.json`.

The earlier constructor gate passed 35 generated proof files. This proves the extracted actual annotated from_vec
body, original byte prefix and dispatch metadata, plus the verification-only
explicit cleanup body/caller. It does not prove normal Drop or full-crate
integration. The descriptor conversion is now body proved, with nonnull construction
confined to B1. Native extraction tests pass twice, including the unchanged
four-word normal layout. Eight B2/B3/B4 negative features reject their intended
VCs; two native affine fixtures reject moves with E0382. See the
[bound constructor probe](verification/probes/bound-vec-constructor/README.md)
and its exact source/evidence snapshots.

A subsequent constructor gate strengthens the packed original-capacity class
relation to the native capacity. Its 35-file proof and three native tests
(including a nonzero 2 KiB class) use the archived `2592fae` physical modules
plus the strengthened constructor/arithmetic source, in an isolated temporary
crate. This is an exact recorded component snapshot, not a replay of in-flight
physical-pool source or full-crate integration.


The earlier local B1/B2/B4 Vec/raw bridge checkpoint passed its eleven-file isolated gate:
detach, split, mutate independently through two live disjoint slices, join,
and restore the changed contents while preserving all other bytes. The three
physical Vec/access bridges are trusted and reviewed; split/join and caller
bodies are proved. Three B2 negative controls reject partial coverage, Unknown
initialization and mismatched namespaces. This uses unmodified
Creusot/creusot-std 0.13 and keeps the ordinary Vec model unchanged. See the
[physical trusted boundary](verification/RAW_VEC_TRUSTED_BOUNDARY.md).
Explicit raw cleanup also passed the fourteen-file checkpoint described
below. The current constructor gate above connects actual from_vec source;
Shared, split and normal Drop integration remain open.

The actual `BytesMut::try_unsplit` now uses an address-only comparison helper
for its buffer and Shared pointers. Normal builds retain native thin-pointer
equality; proof builds use the `addr_eq` contract, which grants no logical
pointer/provenance identity. The [helper probe](verification/probes/address-comparison/README.md)
proves six files and rejects the identity-forging negative VC. Fresh ordinary
bytes tests (997), doctests (246), and the no-default-features build pass for
this change. This does not establish a try_unsplit body proof or runtime
ownership integration.

Implementation of the approved ownership design has started. The new
[feasibility results](verification/OWNERSHIP_FEASIBILITY_RESULTS.md) record
vtable translation and automatic Drop as excluded integration paths under the
no-large-compiler-change constraint. A real boxed immutable control block can
be shared through existing lifetime fractions and recovered after joining all
fractions; this is an isolated foundation, not a BytesMut/refcount proof.
The new owned-region kernel proves seven isolated files, including split/join
and exact slot preservation across a returned pair of regions. Its overlap
negative rejects the intended join precondition. That isolated kernel is a model-only ledger. The subsequent reviewed
Vec/raw bridge binds a sealed PhysicalRegion wrapper to native allocation
ownership; the model-only constructor itself grants no byte access. Separate native RawBuffer bodies have five passing
isolated tests; no physical trusted contracts or BytesMut integration are
established by those tests.

The 21-target replay and ordinary-test counts below refer to source checkpoint
`27cc729`, preceding the ownership implementation changes. Its retained
`checkpoint-manifest.json` records that historical snapshot; it is not a hash
manifest or fresh replay of the current worktree.

Full runtime verification remains **incomplete**. The 19-target inventory in
the historical section below was captured at commit `60e81ad`; its counts,
hashes, and replay results apply only to that snapshot. The current working tree
contains later structural proof changes and `src/provenance_specs.rs`, so those
historical totals must not be read as a replay or validation of the current
tree. The cfg-only Buf/BufMut convenience and comparison changes have passed
the final native-source checks summarized here. No integrated crate proof of
the current runtime is established.

The earlier runtime ownership checkpoint reached stop condition **D**. The
local sealed Vec/raw bridge now supplies an initial memory-resource interface;
connecting that interface to the actual handles remains open. The concrete target is
`Vec<u8> -> BytesMut::from_vec -> split_to/split_off -> mutate both handles ->
drop both`: the split path promotes the shared backing allocation, gives each
handle a distinct writable interval, and the last drop must recover and destroy
the allocation with the actual Release/Acquire protocol. Current `Perm` and
`PtrLive` support borrowed regions whose lifetimes end with a borrow of one
owner; they do not provide independently transferable owned regions plus a
separate allocation-recovery authority. The actual Vec raw-parts bridge and the
Vec stored in `Shared` also need exact allocation/layout/init tracking and a
suspended-ownership interface. Details and the proposed minimum extension are
in [`verification/BYTESMUT_OWNERSHIP_DESIGN.md`](verification/BYTESMUT_OWNERSHIP_DESIGN.md).

`src/provenance_specs.rs` adds `STD-PTRWRAP-01`, a narrow Creusot extern
specification for one-byte `wrapping_add`. Its postcondition describes only
the numerical address calculation. It does not specify pointer provenance,
allocation liveness, `Perm`/`PtrLive`, or dereferenceability, and it does not
make the tagged-pointer runtime path an ownership proof. Its isolated positive
probe proves 8 files/23 VCs, including address tagging/untagging and metadata
address calculations. The forged-metadata dereference negative control fails
at the intended permission VC. Both are component-level evidence, not an
integrated pointer-ownership proof.

A fresh sequential positive replay of the current working tree passed all 21
configured component targets (194 generated proof files and 581 named VCs,
including repeated dependencies). The new ownership-frontier probe contributes
2 helper-only files/18 VCs; it reuses the proved borrowed `Box` region helper
and does not prove actual `BytesMut` split, mutation, or drop. The 39 recorded
negative controls reject their intended VCs. Default std tests/doctests (997
tests and 246 doctests) and the no-default-features build also pass. These
results are component and behavior checks; no full-runtime proof is established.
The final current-source checks also pass for the `cfg(miri)` compilation path
(compilation only; Miri was not run). The current rustdoc API inventory has 997
items, and the coverage updater links 67 actual helper rows; the normal-build
pointer-address cast is excluded because only the proof-specific address
variant was probed.

The latest fresh full-crate translation now clears the earlier recursive
`Buf`/`BufMut` and `PartialOrd` normalization blockers, but stops before VC
generation on two vtable cycles: `static_clone` with `STATIC_VTABLE` and
`owned_clone` with `Owned::VTABLE`. No Coma tasks are produced for the actual
comparison adapters or constructor bodies, so those are not counted as
translated or proved. See
[`verification/artifacts/logs/runtime-ownership-frontier-translation.log`](verification/artifacts/logs/runtime-ownership-frontier-translation.log).
The formal runtime proof entry also stops during translation with the same two
cycles and produces no Coma tasks; see
[`verification/artifacts/logs/runtime-ownership-frontier-proof-entry.log`](verification/artifacts/logs/runtime-ownership-frontier-proof-entry.log).
This translation failure is separate from the stop-condition-D ownership
resource gap described above. Historical totals and source hashes below remain
specific to commit `60e81ad`; do not combine them with the fresh replay as if
they came from one snapshot.

## Historical component checkpoint at commit `60e81ad`

Full runtime verification is **not complete**. Isolated helper and storage
proofs establish useful parts of the implementation, and the helpers are wired
into selected native call sites. They do not verify complete `Bytes` or
`BytesMut` ownership, trait, reference-count, or destruction behavior.

The crate-level `verify-all` attempt recorded at this historical checkpoint
stopped during translation in
`src/buf/buf_impl.rs` and `src/buf/buf_mut.rs` on illegal recursive `Buf` and
`BufMut` traits. It also stopped at a rustc internal compiler error while
normalizing the `Bytes: PartialOrd<T>` `DeepModel` projection. The corresponding
log at that revision is
`git show 60e81ad:bytes/1.11.1/verification/artifacts/logs/runtime-entry.log`. The
default-std integrated tests (997 tests and 246 doctests) and the `no_std`
check pass; these are build
and behavior checks, not a successful full-runtime proof. The compiler API
inventory contains 993 items, with 64 helper function rows linked to isolated
exact-source proofs. The final component replay passed all 19 targets: 184
Coma/proof files and 540 named VCs, including repeated dependency helpers.
These totals are not a completion percentage or a count of unique obligations.
All 38 negative controls failed at their intended VCs. Source/configuration
hashes and paired proof artifacts are the files as they existed at that
revision, including
`git show 60e81ad:bytes/1.11.1/verification/artifacts/component-evidence.json` and
`git show 60e81ad:bytes/1.11.1/verification/artifacts/checkpoint-manifest.json`. The
current working-tree evidence manifest has since been refreshed for the
21-target replay above.

| Component | Isolated vanilla Creusot 0.13 evidence | Runtime coverage |
|---|---|---|
| Saturating arithmetic and bounded adapter helpers | Bodies proved; bounded helpers cover minimum, prefix, budget decrement, and representative callers (5 files) | Bounded helpers are called by Take, Limit, Reader, and Writer; trait-level composition is unproved |
| Slice, cursor, comparison, and chain helpers | Slice (4), cursor (10), comparison (3), and chain (5) proof files pass | Selected slice/cursor/chain helpers are wired; comparison helpers are used in `Bytes` and `BytesMut` equality/order impls, but those callers remain part of the integration blocker |
| `BytesMut` capacity and packed metadata | 14 proof files pass, including the native `leading_zeros` encoding, reconstruction bounds, packed repr round-trip, and vector-position update/preservation | Pure helpers are wired into `BytesMut`; pointer/integer casts at call sites are outside this proof |
| Fixed-width byte codecs and endian helpers | u16/u32 codecs (7), u64/u128 codecs (14), and endian operations (27) pass | Selected `Buf` and `BufMut` read/write call sites use these helpers |
| Checked fixed-width reads | Narrow reads (12) and wide reads (18) pass | Selected `Buf` read methods use these helpers; whole trait/API proofs remain blocked |
| Signed-wide fixed-width operations | Work in progress for i64/i128 operations | Not wired into the runtime or counted as proved component evidence |
| Mutable slice operations and uninitialized storage | Extracted helper probes pass; uninitialized-storage helpers have 4 proof files | Selected `BufMut` operations are wired; surrounding ownership and trait obligations remain unproved |
| Variable-width unsigned reads | 14 proof files pass for unsigned reads | Wired to the `Buf` `try_get_uint` and `try_get_uint_le` overrides; fixed-width signed i64/i128 readers also have 24 proof files and eight runtime overrides |
| Initialized Box storage | 13 proof files pass in the promoted Astra run | Isolated storage foundation only; does not prove `Bytes`/`BytesMut` ownership or drop behavior |
| Disjoint Box-region permissions | Probe proves a real `Box<[u8]>` can be split into unique borrowed regions, mutated independently, and recovered with other bytes preserved | Foundation only; uses standard `Perm`/`PtrLive` primitives and does not prove Bytes aliasing, provenance, reference counts, or `Drop` |
| `free_boxed_slice` and explicit deallocation | Extracted body and real Box-to-suffix caller prove with reviewed standard pointer/layout/allocator primitives | Bytes callers and resource supply are not connected/proved |
| Full `Bytes`/`BytesMut`, traits, vtables, provenance, reference counts, and automatic `Drop` | Not proved | Blocked by translation and model support; no replacement model is counted as runtime coverage |

No new helper introduces a `#[trusted]` contract. Proofs that use standard
pointer, permission, layout, or allocator primitives rely on the existing
`creusot-std` contracts for those primitives. A proved helper body is distinct
from a proved caller, an adopted trusted contract, and successful full-runtime
verification.

Negative VCs exercise representative wrong results and ownership errors,
including incorrect capacity encoding/reconstruction, lost packed flags,
wrong vector positions, wrong byte order/value, short-read behavior, overlapping
regions, and invalid Box recovery. The corresponding historical logs live
beside each probe at commit `60e81ad`. These negative probes check that the
specified properties reject the mutations; they do not expand runtime
coverage.

The default-std integrated test/doctest and no-std logs for this checkpoint are
in `verification/artifacts/logs/` at commit `60e81ad`. Component probe sources
are in `verification/probes/` at that revision. Its `./verify-all.bash` run
reproduces the historical runtime translation blockers noted above. Proof
commands require elevated execution because Why3 uses Unix-domain sockets.

## Remaining frontier

A final read-only review found no additional small helper that closes the missing
`Bytes` content/ownership relation. A static-only Bytes view is a possible future
restricted target, but still needs a justified pointer/byte model and dispatch
correspondence. General `Bytes::as_slice`, `Buf for Bytes` and capacity-preserving
`BytesMut` writes require allocation/provenance and writable/readable interval
resources carried by the actual handles. Clone, split, promotion, release and
Drop must preserve/recover those resources. Extending arithmetic helpers cannot
establish these premises. The current full-runtime translation failure was
reproduced after the comparison helper extraction.

## Reproduction and configuration

Run `bash scripts/verify-components.sh` for all 19 positive component probes,
`python3 scripts/verify-negative.py` for the 38 intended VC rejections, then
rerun the positive suite to restore default-feature live outputs. The collector
requires every configured component and checks source/configuration/artifact
hashes. Its `integrated_runtime: false` means no full-crate proof; the legacy
`foundation`/`connected_helpers` scope label is a grouping, while runtime call
connections are recorded above and in `SOURCE_CORRESPONDENCE.md`.

All proof results use unmodified Creusot 0.13.0, x86_64, native atomic Ordering
and no `sc-drf`. Default std tests and a no-default-features build were checked;
32-bit, optional features, Miri/Loom/Verus and performance were not validated.

## Local physical bridge: disjoint mutation checkpoint

The isolated raw-vec bridge now passes eleven generated files, including a
caller that detaches a Vec, splits exclusive regions, holds two disjoint mutable
slices, changes bytes independently, rejoins and restores the current contents.
Two native probe tests pass. B1 detach, B2 resume and B4 slice access are audited
trusted physical boundaries; region split/join and the caller are body proved.
The exact B4 contract was reviewed by Astra. Source/task snapshots and hashes
are retained in `verification/artifacts/evidence/raw-vec-b1-b2-b4-manifest.json`.
Explicit deallocation, actual BytesMut integration and automatic Drop remain
open; this is not a full-crate proof.

## Explicit physical cleanup checkpoint

RVB-03 `deallocate_vec` requires full coverage plus Recovery with matching sealed
identity/capacity and consumes all three. It requires no Known prefix and
performs trusted native zero-length Vec destruction. Astra reviewed the exact
declaration/body. The positive probe passes fourteen generated files and four
native tests, including zero capacity, all-Unknown spare capacity and nonempty
fragment joins in both tuple return orders. This is explicit caller proof, not
automatic Drop or Shared last-owner proof. Evidence/source hashes are retained
in `verification/artifacts/evidence/raw-vec-b1-b2-b4-b3-manifest.json`.

The latest full-runtime translation still stops on the two vtable cycles, before
VC generation; the retained log is
`verification/artifacts/logs/runtime-raw-vec-checkpoint-translation.log`.

## Constructor source connection and remaining ownership frontier

The proof configuration of the actual BytesMut constructor now stores a sealed
bound pointer and affine ghost Recovery/full-region capabilities. The normal
representation retains its existing NonNull field. Proof metadata copying no
longer uses ptr::read on the ghost ownership state. Unadapted mutable access,
spare-capacity access, reserve and promotion discard that witness; pointer
updates become unbound. No persistent all-method ownership invariant is claimed.

The exact source-body gate imports real helpers and declarations without
trusted bytes protocols. The normal native test run passes 997 tests and 246
doctests; the no-default-features check passes. The full proof translation still
reports the same two vtable cycles, with no whole-crate VC proof; see
`verification/artifacts/logs/runtime-bound-constructor-translation.log`.

Shared integration must replace the ordinary Vec owner with one raw descriptor,
keep actual affine retired regions and Recovery, track ticket registrations,
and link their retirement to native atomic decrements. Ordinary Vec ownership
may not coexist with PhysicalRegions. Native fetch_sub needs a generic modeled
primitive; concurrent release-sequence synchronization and automatic Drop
remain separate tool/model limits.


## Current practical boundary

No further abstract ownership layer is needed. The remaining actual Shared
integration must replace its ordinary Vec owner with one sealed raw descriptor
and adapt promotion, release, reserve/recovery and conversion paths. Native
AtomicUsize::fetch_sub has no vanilla modeled contract: connecting it requires
a reviewed generic primitive bridge, which would add atomic trust. These are
deferred difficult integration work, not already proved or impossible locally.

Full-crate vtable translation and automatic Drop effects remain compiler-model
blockers under the no-large-Creusot-change constraint. Concurrent release
sequences/view transfer, control-block destruction, Bytes/freeze read-sharing
and the remaining allocation paths are also unproved. The explicit local gates
above do not establish complete bytes verification.

The readonly-access/lifetime-diagnostic archive records Rust E0505 rejecting
release of a handle while a later read keeps its as_slice borrow live. This is
translation/borrow-check evidence only, not a Why3 VC or automatic Drop proof.

The unique-access/negative-unknown-publication archive records exactly one
failed Known-prefix guard in actual set_len after unique spare storage is
re-uninitialized (94 proof files; caller 13/14). Full allocation ownership and
Some(None) are established before that attempted publication.

Fresh actual_from_vec constructor regression after unique access passes 56
proof files, with zero unproved leaves. Frozen source/configuration and exact
extraction are in bound-vec-constructor/unique-access/positive. Automatic Drop
is excluded; cleanup is explicit.

The deref-purity-feasibility diagnostic confirms two existing vanilla 0.13
translation restrictions: ordinary Deref must be ghost, and a ghost Deref
cannot call program(terminates) access operations. Native Rust checks the
reduced example. These are compiler diagnostics, not rejected Why3 VCs or
proof of actual BytesMut Deref. Current physical B4 operations remain ordinary
program operations; no ghost writeback or trait-body axiom is introduced.

The completed open-invariant-borrow reduced gate rejects nine mutable
reborrow/resolve invariant leaves across three functions. Even with open_inv
arguments and open_inv_result, an intentionally invalid intermediate value
cannot cross those generated checks in vanilla 0.13. The actual strong
lifecycle-invariant attempt is not claimed proved; valid-handle-only trait
verification is being separated from the explicit-predicate lifecycle gate.
Canonical reduced source/configuration and failed proof leaves are archived
in open-invariant-borrow/rejected, without weakened invariants or new trust.
