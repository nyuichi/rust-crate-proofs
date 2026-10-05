# Verus refcount experiments

These runnable experiments go beyond the earlier read-only stock-API audit.
They do **not** establish concurrent correctness of bytes 1.11.1.

Run `python3 verification/probes/verus-refcount/run.py` from the crate. The
runner defaults to the session-private installation at
`/workspace/bytes-verus-tools/verus-x86-linux/verus`, with private Rust 1.98.1.
Set `VERUS` and `VERUS_RUSTUP_HOME` to use another installation. No repository
compiler, Creusot library, or native bytes ordering was modified.

The recorded verifier is release `0.2026.10.04.426d8b0`, commit
`426d8b01e7ffb910a36c15e1f870ff985bb61eef`. The runner checks exact proof/error
counts, intended failure classes, compiles and executes the PCell caller, and
records source hashes and version in `artifacts/manifest.json`.

| Experiment | Result | Meaning |
|---|---|---|
| `release_sequence.rs` | 5 verified, 0 errors | Body-proved arbitrary-length equivalence of accumulated publications and direct contiguous-RMW release-sequence definition; final-zero read-from lemma; A/B/C litmus |
| `retired_resources.rs` | 4 verified, 0 errors | Generic affine escrow; exact original tracked resources returned only after every pending entry retires; real PCell permission instantiation |
| `sc_counter.rs` | 1 verified, 0 errors | Actual stock PAtomic SC count operations for the A/B/C trace |
| `native_orders.rs` | 2 verified, 0 errors | Syntax/ordinary control-flow only; no atomic value or ownership theorem |
| Missing Acquire | 5 verified, 1 error | Cannot obtain A's published write from Release-only decrement |
| Interrupted sequence | 5 verified, 1 error | A plain Relaxed store breaks earlier release-head publication |
| Missing empty ticket | 4 verified, 1 error | A unit/zero-payload resource still blocks full recovery |
| Direct native value gap | 2 verified, 1 error | Even a private freshly initialized native atomic lacks usable value semantics |

## Weak-memory model

Modifications are ordered in one atomic location's modification order. A
release head reaches a later modification when every intervening modification
is an RMW, including Relaxed RMWs. A release adds its prior writes to the
carried publication set. A Relaxed RMW forwards that set, without acquiring it.
An Acquire load observes the publication set attached to its read-from source.
The induction proves the recursive accumulation agrees exactly with the direct
release-sequence definition; it cannot invent unrelated publication labels.
These labels describe **visibility, not affine physical ownership**.

The concrete trace is A Release 2→1 publishing marker 10; B Relaxed 1→2;
B Release 2→1 publishing 20; C Release 1→0 publishing 30; C Acquire reads zero.
The final Acquire sees all three publications, including A across the Relaxed
increment. The final Release alone does not confer A's view.

`refcount_trace` separately states RMW predecessor coherence, unit increments
or decrements, and no operation starting at zero. The final-zero lemma proves
that there can be no later modification, and a load with read-from at least its
own preceding decrement must read that zero. The no-resurrection premise would
need to come from conserved live tickets in an integrated protocol. The
write-read coherence premise would need to come from native atomic semantics.
Neither premise is claimed derived from the bytes implementation here.

## Affine resources and SC comparison

Escrow is parameterized by a tracked resource type `R`. Its ghost expected map
cannot manufacture an `R`: retirement consumes an actual tracked `R` of the
expected identity, and finish moves out the actual stored tracked map. The
invariant relates pending keys, retired keys, and their values. Retirement
reduces pending cardinality by one; finish requires zero. The compiled example
instantiates `R` with actual `PCell<u64>` points-to permissions, then recovers
and consumes the two cells with their values intact. The negative has two
unit-valued cells and attempts recovery while one ticket remains.

This is a sequential escrow proof. It is not connected to an atomic invariant,
thread spawn/join, the event model, bytes regions, or Creusot Recovery. The SC
counter comparison holds its stock atomic permission exclusively and proves
native SC values; it is likewise not a concurrent resource protocol. Keeping
these limits explicit avoids converting the event model into ownership by
assertion.

## Exact native integration attempt and trust

Current stock Verus accepts native `AtomicUsize` with Relaxed/Release/Acquire.
Its [standard atomic specifications](https://github.com/verus-lang/verus/blob/426d8b01e7ffb910a36c15e1f870ff985bb61eef/source/vstd/std_specs/atomic.rs)
explicitly provide **no support for reasoning about atomic values**. They are
empty `assume_specification` declarations. The archived direct experiment
constructs a private atomic initialized to 1, Release-decrements it, performs
an Acquire load, and fails the single assertion `old == 1`. This is a missing
semantic contract, not a compiler translation failure or counterexample to
Rust atomics. The positive syntax-only bodies must not be advertised as
refcount proofs.

[Stock PAtomic](https://github.com/verus-lang/verus/blob/426d8b01e7ffb910a36c15e1f870ff985bb61eef/source/vstd/atomic.rs)
provides useful permission/value contracts, but hard-codes SeqCst. This is why
the same values are provable in the SC comparison. Replacing native calls with
that interface would change the implementation's ordering and is not done.

No project `assume`, `admit`, `external_body`, or new trusted contract is added.
The experiments rely on Verus, its SMT solver, and the standard vstd logical
collections, tracked-map operations, PCell, and PAtomic specifications. The
model definitions are a deliberately restricted single-location account of
release sequences; they are not a validated embedding of all Rust/C++ memory
semantics. Native standard-atomic callability additionally uses the stock
empty specifications described above.

To connect unchanged bytes, a reviewed generic weak-memory atomic interface
must establish modification order/read-from/coherence and transfer views
without letting a Relaxed operation extract acquire-only resources. Its
adequacy to actual Rust atomics would be a new trusted primitive boundary
unless independently checked against a memory-model semantics. Next, live
handle tickets and retired physical regions must inhabit that protocol.
Finally, using Creusot's existing PhysicalRegion/Recovery requires a checked
cross-tool resource interpretation, or verification of that ownership slice
entirely in Verus. Importing these abstract theorems as Creusot trusted release
contracts would assume those missing connections and is not performed.

## Follow-up: small native bridge and invariant boundary

The user permits small generic native primitive contracts, so the investigation
also tried those rather than treating absent stock specifications as decisive.
`native_bridge_boundary.rs` contains an **experimental, unadopted** generic
native counter permission interface. Its constructor, Release decrement/swap,
and Acquire load are `external_body` primitives with numerical contracts under
exclusive permission. No bytes type, retirement policy, or physical permission
is mentioned in those trusted contracts. The private native counter proof now
passes (**1 verified**). This removes the first missing-value obstruction.

`native_retirement.rs` also body-proves a tokenized state machine relating
remaining count, pending tickets, deposited tracked resources, and full
withdrawal (**5 verified**). Its transition/invariant bodies add no trust.
This fixed-inventory model does not yet allow post-initialization resource-value
mutation or dynamic splitting and is not a completed bytes protocol.

The next actual attempt is `--cfg attempt_weak_invariant`: opening stock
`AtomicInvariant` around the numerical weak-memory primitive is rejected with
`open_atomic_invariant cannot contain non-atomic operations`. Calling the
primitive atomic in Rust does not supply Verus's stronger invariant rule.

The paired **deliberately rejected** switch `--cfg rejected_sc_rule` adds
`#[verifier::atomic]` to the weak primitive. The diagnostic then verifies
(**2 bodies**) despite extracting a real PCell permission through a Release
RMW without any Acquire. It demonstrates an overstrong proof rule; it is not
accepted ownership evidence. Such an interface could transfer writes across
threads without the synchronization required by the earlier missing-Acquire
negative. The numerical contract itself does not prevent this misuse.

Therefore the missing connection is not merely a few value postconditions.
A generic weak-memory resource modality must distinguish transfer into a
release publication from extraction after Acquire and track the appropriate
thread views/release sequence. Stock SC invariant opening cannot simply be
relabelled. A sealed retirement library could enforce a discipline at its API,
but using the overstrong internal rule would still require an independent
soundness argument for that restricted proof rule; accepting the bytes-specific
protocol by inspection would be trusted protocol, not a body proof under an
adequate generic rule. This experiment does not adopt that shortcut.

The original four probe files still add no project trust. Only the explicitly
experimental `native_bridge_boundary.rs` adds the numerical primitive clauses;
none are imported by bytes, Creusot, or the accepted component proofs. The
manifest distinguishes both boundary diagnostics as `adopted: false`.
