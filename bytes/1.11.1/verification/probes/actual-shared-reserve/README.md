# Exact public cfg Shared reserve

The build extracts `BytesMut::reserve` and its actual constructor, split,
registration/read and consuming cleanup dependencies from bytes1.11.1 source.
The restricted `bytes_proof_shared_reserve` configuration passes an external
affine coordinator into the public method. The native default body is unchanged.
This gate is sequential and uses explicit consuming release, not automatic Drop.

The complete positive gate proves105 files with zero unproved files, including
the public method and two callers. One caller advances the right view after
splitting and releasing its left sibling, then reserves and reads/writes. The
other retains an empty sibling while nonunique slow growth copies into a new
unique allocation; the remaining sibling then performs final old-buffer cleanup.
The reserve interface states registration preservation in the singleton branch,
exact pending-cardinality change, preservation of other registrations and
initialized visible contents. Singleton exclusivity is a proved finite-map lemma.
No bytes ownership/refcount rule is trusted. B6 offset metadata is physical TCB
only; physical B1-B5, immutable/mutable B4, native scalar atomic and boxed typed
permission boundaries retain their separately documented trust.

Native tests cover singleton offset/reclaim/growth and nonunique copy matrices,
plus exact allocation/reallocation/free events for six boundary cases. Native
results are observations, separate from the affine proof.

`evidence/positive` saves proof-time source, extraction, Coma/proof JSON and logs.
An independent build-script rerun regenerated the positive extracted source and
fragment manifest byte-for-byte. `evidence/body-positive-callers-negative` saves
the earlier103-body-positive run with two incomplete caller files before the
reviewed interface correction. `negative_wrong_offset_namespace` is proof-only
and must reject supplying a base descriptor detached from a different Vec; never
execute negative features natively.

Excluded: native concurrent schedules, arbitrary refcounts, panic/unwind cleanup,
default public dispatch without a coordinator, general automatic Drop and the
whole crate integrated proof. Invoke `verify.bash` using the pinned toolchain
under the shared proof lock and elevated Why3 execution.

The unrelated-offset control rejects7/9 in one file; its two outstanding guards
are same-namespace/capacity metadata requirements. Saved under
`evidence/negative-wrong-offset`.
