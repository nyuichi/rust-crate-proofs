# Original BytesMut unique allocation/write experiment

Changed premise: the original four native fields remain in place. A cfg-only
optional sidecar connects them to the actual B1 allocation; None establishes no
authority. This gate extracts marked source verbatim, including the actual
with_capacity, no-growth reserve, spare access, memcpy call and advance_mut.
The advance_mut method is selected from its original trait impl into a local
inherent surface; this does not prove open BufMut laws. reserve_inner's interface
requires false: its body is outside scope, and the selected reserve precondition
must prove that branch unreachable. No alternative buffer representation.

Spec map: from_vec consumes the source Vec once via B1; sealed BoundPtr equals
the actual ptr (not merely its address); full Recovery/PhysicalRegion covers
[0,cap), native tag denotes Vec/off0; known prefix matches source bytes. Spare
access uses ordinary B4 with slot writeback, raw memcpy initializes only its
prefix, and advance_mut publishes only known slots. The public caller returns a
live original BytesMut. No Drop, freeze, Clone, growth, shared state, concurrency,
or whole-crate claim. B1/B4 and the local typed memcpy contract remain explicit
TCB, and proof cfg/native correspondence is a reviewed obligation.

Before run: generate extracts and hashes from current source. Preserve each
failed source/Coma/JSON/log before edits. Two equivalent failures require review;
a third requires structural change. No increased proof timeouts. Positive needs
completed engine output and all JSON leaves closed; zero generated goals is not
success. Native tests are bounded correspondence checks, not an ownership proof.

## Recorded results

`positive-50.tar.gz`: completed `Proved (50 files)`, exit 0, 50 Coma files,
50 proof JSON files, zero null leaves. The selected bodies and both live-return
callers close, including append onto a nonempty Vec prefix and unchanged Unknown
spare slots. Extracted native empty/nonempty/spare cases pass; original
`test_buf_mut` and `test_bytes` passed 23 and 118 tests respectively.

`negative-uninitialized-publication.tar.gz`: completed expected failure, one
null in `negative_publish_unknown` (5/6). Exported task identifies exactly
`advance_mut requires #2`, the newly published interval's Known-slot requirement.
The caller allocates capacity one through the actual from_vec/B1 path and omits
the write. It is proof-only and is never executed natively.

Earlier attempts are retained: Rust macro-depth/prophetic annotation failures,
then missing nonnull and packed-tag facts. The first finite case split was
mistakenly placed in excluded reserve_inner (recorded source shows this); the
correctly placed split exposed that integer-mode shift/or are uninterpreted.
The final body-proved bitwise metadata lemma isolates that arithmetic, with no
new trusted property. The original data field still contains the exact native
packed value. Its address is below 32 and congruent to 1 modulo 4, denoting
Vec storage at offset zero.

### Source correspondence and limits

`extract.py` copies marked source spans verbatim and records their SHA-256 and
line numbers, plus full bytes_mut source/generated hashes. It copies the actual
four-field struct with its selected proof field, actual Shared field layout and
constants. The proof cfg replaces only the physical representation boundaries:
original ManuallyDrop Vec detachment is interpreted by B1 once; spare raw-slice
construction is interpreted by ordinary B4 using the same sealed pointer.
The sidecar is absent from native builds. This is a conditional exact-source
method gate, not a proof that every original method preserves this sidecar.
`advance_mut` is selected into a local inherent surface; original BufMut laws
are outside this gate. reserve_inner and panic_advance have false-precondition
unreachable interfaces in this gate, requiring the callers to prove they cannot
reach either. No result follows for growth or invalid advance.

The native debug pointer-address assertion is retained using the existing
address helper; exact allocation authority comes from B1, never that address.
The generic BoundPtr::as_non_null postcondition now states exact pointer equality
and its field-return body is proved. There is no public constructor/extend trust.
Returning a live handle deliberately makes no destructor, freeze, sharing,
refcount, deallocation, or allocation-failure termination claim.
