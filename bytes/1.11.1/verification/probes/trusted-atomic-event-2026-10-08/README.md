# Trusted atomic event invariant: isolated original-API prerequisite

This is a generic synchronization/resource boundary experiment, authorized after
inspection of stock Creusot invariants and atomic Committer rules. It does not
change production bytes, provide a replacement buffer API, or trust a bytes
ownership/refcount theorem. The original freeze/read61 result is not changed.

## What the new generic TCB means

`RawAtomic::new` creates the actual native AtomicUsize and its matching model
permission/history, following the existing generic native-atomic adapter.
`EventAtomic::bind` consumes RawAtomic and ONE owned ghost state S, checks its
protocol and atomic association, and stores that affine state in an opaque
logical resource invariant. Every increment borrows THAT stored state once at
its actual native RMW event and restores its successor. It must never invent,
reset or reallocate an Authority between events. PhantomData is erased native
representation, not a body proof of this semantic storage rule. This follows
the stock opaque AtomicInvariant/public/existential callback shape. A pure
private_state(self) projection is deliberately absent: it would incorrectly
frame concurrent interior mutation as an immutable value.

The exact native instruction is self.atomic.native.fetch_add(1, Relaxed).
The wrapper contract relates this field's event to its fixed ModelAtomic ward,
old/new values and stock Committer. That native/model interpretation is trusted;
it is not established by bare pointer or model equality. Wrap is explicit:
MAX -> 0, otherwise old+1. No SC feature or strengthened ordering is enabled.

Only ordinary atomic operations enter the invariant. There is NO ghost open,
Tokens conversion, AtomicInvariant export, state projection, or into_inner.
The FnGhost callback cannot invoke another ordinary atomic operation. Thus the
known reentrant old-open_at/stock-Tokens combination is not available. The
callback may open an unrelated stock invariant with its own tokens, but cannot
access this EventAtomic state by that route. Bind consumes S; callbacks must
preserve its public identity and protocol. The exact event must be committed
using its actual Perm; the boundary does not provide or duplicate that Perm.
Sync requires S: Send + Objective, as in the weak stock AtomicInvariant.

This is an explicit NEW generic TCB rule. Its machine-checked callers do not
prove the rule's semantic adequacy. A future generic verifier/resource-semantics
implementation could replace these trusted signatures locally; no such body
implementation or small-removal guarantee is claimed today. The trusted
Send/Sync markers accompany that same generic invariant interpretation.

## Body-checked client and limits

State contains the actual atomic Perm/history and an authoritative finite map
of fresh integer IDs to exclusive registrations. Its protocol ties the native
counter at the latest history event to the number of registrations modulo
usize::MAX+1. The history domain is contiguous, allowing the stock RMW rule's
fresh next timestamp to establish the actual latest event. Issuance updates
that authority through stock body-checked RA operations; no trusted ticket law
or bytes-specific protocol is introduced.

`clone_registration(&self, Ghost<&Ticket>)` keeps the original affine fragment
and returns a fresh one. Its postcondition ties the returned native old counter
to the new ticket ID modulo the machine range. Private `Registered::duplicate
(&self) -> Self` obtains the source from its own field and needs no public
context argument. Both are diagnostic types, not Bytes or a replacement owner.
The caller verifies three distinct simultaneously owned registrations. The
separation lemma requires one mutable borrow: two shared references could alias
the same ticket and cannot soundly establish distinctness.

The concrete EventAtomic<State> also passes a Sync type-bound check in the
proof configuration, exercising the Objective requirement. Native tests repeat
sequential and real two-thread increments, checking returned old values. These
native observations are not a proof of scheduler behavior or threaded bytes.

This first gate has no retirement, read permission, physical payload, B4 read,
Acquire, last-owner recovery, byte ownership, or automatic Drop claim. In
particular zero after modular wrap is NOT a reclamation condition. Original
refcount overflow/abort logic is not proved. A later physical Release/Acquire
experiment must use an independently reviewed ordering-sensitive interface and
a meaningful missing-Acquire negative; a Relaxed registration proof cannot
supply it. Existing missing-Acquire counterexamples remain preserved elsewhere.

## Controls and evidence

Feature gates provide separately captured negative builds:

- `negative_wrong_bind`: bind a state whose atomic differs from the native ward;
- `negative_wrong_ward`: shoot a committer against another atomic permission;
- `negative_empty_commit`: return from the actual event callback without shooting;
- `negative_double_commit`: shoot twice in the actual increment callback;
- `negative_duplicate`: move one affine ticket into two results (Rust rejection);
- `negative_reentry`: call an ordinary operation in FnGhost (purity rejection).

No invalid native control is executed. Proof failures, Rust typing rejection,
ghost-purity rejection and native authoring failures are distinct classifications.
The initial native authoring and unwrapped ghost-constructor failures are
preserved, not counted as positive evidence. Later native tests pass.

`run-proof.sh` uses the shared flock, one solver process and 1024 MiB. Invoke it
elevated for Why3 sockets. `capture.py` saves exact sources, active Std analogue
sources, all Coma/JSON trees and logs. `evidence/audit.json` records final outcomes
and hashes. No installed Std, production bytes source or existing evidence was
modified by this probe.
