# T02: objective physical resource transport

The default configuration **proves all 37 files**, including a real stock
Creusot scoped-thread caller, the modified weak retirement protocol, and
conditional deallocation by the parent after joining. Native execution passes
two tests: **84 physical threaded cases** and the unchanged primitive's
64-iteration atomic race test. No new trusted item or production change is added.

This closes T01's concrete Objective/Sync obstruction for **RetiredPart**. It is
an architecture prerequisite, not the complete bytes lifecycle or the admission
witness. Actual Bytes handles, Clone/Deref, byte reads while peers retire,
arbitrary registration, last-thread deallocation, automatic Drop, and generic
transport for every `Payload` implementation remain outside this proof.

## Representation and callable contracts

T01 stored `Snapshot<(RetiredPart, RetiredPart)>` in its invariant, preventing it
from being Objective. T02 stores `PartMetadata`: allocation namespace, capacity,
interval bounds, and whether that part carries Recovery. These are only Id/Int/
bool values; they cannot construct any capability. The metadata deliberately
does not describe byte contents, which deallocation does not inspect.

Actual affine PhysicalRegion/Recovery values are moved to the children and
sealed in the unchanged stock `AtView<T>` modality on retirement. A body-defined
`wellformed` predicate retains the real region invariant and resource identity,
and the optional real Recovery invariant with matching namespace and capacity.
The protocol guarantees that recovered actual payloads satisfy this predicate
and have the expected metadata. Metadata equality is not full payload equality.

The thread caller retains BoundPtr in the parent, avoiding any new unsafe Send
implementation for its NonNull field. Each child receives fresh Tokens from
stock scoped spawn. The final observer returns the real capabilities through
stock join; parent cleanup joins the actual regions and consumes actual Recovery
through the existing B3 deallocation boundary. This is last-owner-to-parent
handoff, not deallocation by the last thread.

The constructor now states which returned ticket is left/right through a
body-defined logic accessor. Its `accepts_payload` predicate has crate-scoped
logical transparency, exposing the exact callable relation between the ticket,
expected metadata and actual payload wellformedness. No trusted law is added.
The generic protocol can be proved parametrically, but this thread-transport
result applies only to the concrete objective RetiredPart metadata; the generic
Payload trait imposes no universal Objective guarantee on its associated type.

## Limits and trusted boundary

Exactly one final observer is asserted by the native test. The formal caller
proves conditional safe cleanup using the retirement results; its current
contract does not establish eventual/exactly-one completion of both calls.
There is no new count-completion theorem hidden in a wrapper or an assumed
precondition. The public thread caller has no `requires(false)` or reachability
restriction.

The existing native Release-RMW/publication primitive and physical B1–B6
boundaries are imported unchanged by path and hash checked. Stock scoped
spawn/join, Tokens, AtomicInvariant and AtView remain tool/library assumptions.
This experiment does not establish their generic weak-memory adequacy. It does
not reopen the rejected SC-rule shortcut or classify mutable B4 access as ghost.

## Retained attempts

The first body run proved all files except `thread_roundtrip` (39/41): the
constructor contract did not expose ticket orientation. The second had the same
two child retirement preconditions missing because the payload-matching relation
was still opaque across the module boundary. Both complete source/Coma/proof
trees are archived. The reviewed orientation and transparent-relation interface
repair yields the 37-file positive result.

Separate frontend diagnostics preserve the ambiguous Clone derive, required
prophetic annotation, exported private-field `ERROR_UNBOUND_left`, and overly
broad `logic(open)` visibility. Those are not negative VCs. The final source uses
a qualified derive, prophetic wellformed predicate, body-defined accessor and
`logic(open(crate), prophetic)` relation.

## Commands and evidence

Run native tests from this directory with the pinned tool activation and
`cargo test --offline --locked -- --nocapture`. Run `bash run-proof.sh` with
elevated execution for Why3 sockets. The shared lock serializes one prover;
the recorded actual Why3 configuration sets 1024 MiB. `sc-drf` is rejected.

The same threaded missing-Acquire control is `bash run-proof.sh --features
negative_no_acquire`. It rejects exactly one leaf in `SharedRetirement::retire`
(11/12): the peer's `AtView::sync` requires its published view to be covered by
the current view, which the omitted Acquire no longer establishes. Its full
37-file tree has one null leaf; the exact task is archived separately from the
positive. This dependency failure prevents verification of the threaded
composition even though the caller can consume the unproved callee contract.
Existing empty-ticket and publication controls remain prior component evidence,
not admission controls newly proved here.

`EXPERIMENT.md` records the plan before implementation. Evidence manifests pin
every archived member and separately audit each archive by read-back. The
positive archive contains 37 proof JSON files with zero null leaves and exact
proof-time source, dependencies, Coma and logs. `protocol-correspondence.diff`
shows every change to the original local protocol; primitive source hashes are
unchanged. T01 remains immutable in its separate directory.
