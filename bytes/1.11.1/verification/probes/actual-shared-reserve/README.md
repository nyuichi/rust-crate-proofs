# Exact public cfg Shared reserve

The build extracts `BytesMut::reserve` and its actual constructor, split,
registration/read and consuming cleanup dependencies from bytes1.11.1 source.
The restricted `bytes_proof_shared_reserve` configuration passes an external
affine coordinator into the public method. The native default body is unchanged.
This gate is sequential and uses explicit consuming release, not automatic Drop.

The positive shared-growth gate proves 115 files with zero unproved files. It
covers the public reserve, growing `resize` and `extend_from_slice`, and four
callers. One caller advances the right view after splitting and releasing its
left sibling, then reserves and reads/writes. Another retains an empty sibling
while nonunique slow growth copies into a new unique allocation, then performs
final old-buffer cleanup. The two new callers grow from an empty split view
through `resize` or `extend_from_slice`, check the written byte after publication,
and explicitly release the carrier. The shared methods carry an affine
coordinator only under the verification cfg; their default runtime bodies are
unchanged.

The reserve interface states registration preservation in the singleton
branch, exact pending-cardinality change, preservation of other registrations
and initialized visible contents. It also exposes the body-proved no-op case
needed to frame packet identity when growth stays within capacity. Singleton
exclusivity is a proved finite-map lemma. No bytes ownership/refcount rule is
trusted. B6 offset metadata is physical TCB only; physical B1-B5,
immutable/mutable B4, native scalar atomic and boxed typed permission boundaries
retain their separately documented trust.

Native tests cover singleton offset/reclaim/growth and nonunique copy matrices,
plus exact allocation/reallocation/free events for six boundary cases. Native
results are observations, separate from the affine proof.

`evidence/shared-growth/positive-115-verified.tar.gz` saves the exact proof-time
source, extraction, fragment correspondence, all 115 Coma/proof JSON files and
proof/native logs. Its SHA-256 and validation details are recorded in
`evidence/shared-growth/positive-115-verified.json`. The archived correspondence
checks all 51 source-fragment records and confirms the selected method fragments
match the generated extraction byte-for-byte. The 11 changed legacy build
scripts also passed standalone compilation, extraction and offline Cargo test
smoke checks; see `evidence/shared-growth/legacy-buildrs-smoke.log`.

`evidence/positive` saves the earlier reserve-only source and proof. The older
`evidence/body-positive-callers-negative` run has two incomplete caller files
before the reviewed interface correction. `negative_wrong_offset_namespace` is
proof-only and must reject supplying a base descriptor detached from a different
Vec; never execute negative features natively. The unknown-publication negative
feature is also proof-only and its rejection is not part of the positive receipt.

Excluded: native concurrent schedules, arbitrary refcounts, panic/unwind cleanup,
default public dispatch without a coordinator, general automatic Drop and the
whole crate integrated proof. Invoke `verify.bash` using the pinned toolchain
under the shared proof lock and elevated Why3 execution.

The unrelated-offset control rejects7/9 in one file; its two outstanding guards
are same-namespace/capacity metadata requirements. Saved under
`evidence/negative-wrong-offset`.
