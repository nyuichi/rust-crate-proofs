# Native integration frontier, 2026-10-05

The first resumed crate-wide `verify-all` attempt failed before body verification
on five trait diagnostics: `Bytes` Send/Sync, `Bytes::deref`, `BytesMut::deref`, and
`BytesMut::deref_mut`. That historical diagnostic log is in `logs/verify-all.log`.
Passing cfg/extraction probes do not establish that the native crate verifies.

At pinned Creusot `318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`,
`creusot/src/validate/traits.rs:56-65` unconditionally classifies Rust Send and
Sync as trusted traits. Their implementations must be marked `#[trusted]`;
there is no marker-body verification route in this validator. The preserved
validator source records this tool limitation. Marking the bytes implementations
trusted would not satisfy the requested protocol verification.

An isolated extraction of the current `Bytes::clone` body (`src/bytes.rs:759`)
reproduces `unsupported function call type` at `(self.vtable.clone)(...)`.
The probe's only pointer adaptation converts its sealed descriptor back to the
raw pointer expected by the actual callback. `negative_actual_clone` enables
that body. `negative_shared_clone` attempts the affine ticket split through the
actual `&self` receiver and fails E0596: a shared reference cannot supply the
mutable token required by `split_off`. Both current failures are archived with
the exact diagnostic fixture/build source. They are not claims that all future
encodings are impossible.

A small finite private clone-dispatch enum could replace the six known callback
choices (Static, Owned, PromotableEven, PromotableOdd, Shared, BytesMutShared),
removing this dynamic-call frontend blocker without changing public layout.
It would not solve the ownership protocol: an interior AtomicInvariant could
hold spare fractional authority, but opening it requires `Tokens`, which the
stock `Clone::clone(&self)` interface cannot accept and an arbitrary method
cannot freshly mint. The current coordinator adapters therefore do not prove
actual native Clone, Send/Sync or automatic destruction.

The isolated `readonly-b4-purity` gate assesses only immutable slice observation.
Mutable B4 must remain an ordinary program operation: classifying it as ghost
allows erased writes to change the ghost initialization ledger, contradicted by
the existing native counterexample. No mutable bridge annotation is proposed.

After the reviewed immutable chain classification, fresh production translation
reports four errors: Bytes Send/Sync, Bytes default Deref, and BytesMut DerefMut.
The BytesMut immutable Deref purity diagnostic is removed. The new log is
`logs/production-readonly-frontend.log`. The separate `default-readonly` gate
proves the exact default internal read body under its existing authority
precondition, but public AsRef cannot establish `self.proof_initialized()` from
the default field-only type invariant. The frozen Bytes read chain is classified
only under its capability-carrying cfg. Mutable B4 remains ordinary.

The final resumed-source `verify-all.bash` again stops before VC generation on
the same four frontend errors (exit1); its exact log is
`logs/production-current-frontend.log`. `native-latest-manifest.json` pins the
source hashes and reports1255 ordinary native tests (including docs), no_std
check exit0, and no whole-crate proof phase. No bytes Send/Sync marker or
ownership protocol was made trusted to bypass these errors.
