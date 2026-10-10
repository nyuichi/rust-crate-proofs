# bytes 1.11.1 working rules

Only this crate is in scope. Preserve the original public API and representation,
the genuine Vec reverse-comparison fix, and actual implementation verification.
Small source changes for verification are allowed. The retired `bytes::verified`
API and old length/capacity model are not current implementation coverage.
Commit/push validated increments to `origin bytes-runtime-verification`, never
main or force-push.

Start with README.md. Before changing architecture or starting a new proof
experiment, read verification/ARCHITECTURE_DECISIONS.md. Keep the actual frozen
counterexamples and prior component evidence. Reopening a failed route requires
a changed premise and one bounded distinguishing experiment; extra assertions,
wrappers, timeouts or agents alone are not a changed premise.

Use strong contracts and prove the actual bodies. Temporary local trust requires
precise assumptions, a reviewed strong contract and a concrete removal path;
bytes-specific ownership/refcount/last-owner/destructor laws remain obligations.
Reviewed generic physical/synchronization/tool primitives may stay trusted while
callers are proved. When blocked, consult pinned creusot-std and upstream examples;
record the analogue, native interpretation, assumptions and replacement interface.
Never count caller proof as primitive adequacy or disconnected gates as full API
verification. Reentrant invariant opening and SC permission rules for weak
atomics remain rejected.

Use the affected proof wrapper, shared lock, one prover, 1024 MiB and elevated
Why3 execution. Work on default std with native Ordering and sc-drf disabled.
Prove a complete applicable positive once per proof increment. Reuse unchanged
audited controls; add selective negatives only for changed trust, ownership,
correspondence or a specific suspected specification gap. Identify diagnostic
exclusions. Do not rerun an unchanged whole-crate frontend blocker.

Preserve source/capture correspondence and immutable evidence, including failures.
Cargo.toml, Cargo.lock and production sources are hash-bound by current checkers;
changes require updated correspondence, not merely changing expected hashes.
Editorial-only changes need no solver rerun or recapture of frozen documentation.
Update one current guarantee/scope overview, not multiple milestone histories.
Do not append per-iteration handoff reports or replay ancestor audits by default.

No mandatory model routing or separate Astra consultation is needed.
A blocked or partial target is not complete verification.
