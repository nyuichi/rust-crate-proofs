# Tokenless atomic event boundary — bounded plan

Changed premise: user authorized a reviewed new generic synchronization/resource
TCB boundary. Existing AtomicInvariant::open_at is NOT extended: its alternate
Tokens opening would admit reentry. No production bytes source is modified.

New opaque EventAtomic owns one actual native AtomicUsize and a private ghost
protocol. Only ordinary native atomic methods expose the ghost state, through
FnGhost callbacks. There is no ghost open, state projection, into_inner, or
conversion to a stock AtomicInvariant. Native callbacks are erased; this is an
explicit new generic trusted linearization rule, not a body proof of that rule.
State Sync requires Send+Objective. Actual atomic identity is fixed on binding.
Each RMW callback receives its matching stock Committer plus mutable state and
must restore public/protocol and shoot that same event against its actual Perm.
Stock SyncView/Committer ordering contracts remain in force. Relaxed does not
acquire physical payloads; SC is never enabled.

First question: can an ordinary &self method issue a fresh authoritative-map
fragment using an actual Relaxed increment, without mutably borrowing its
receiver, explicit Tokens, resource duplication, or a trusted registration law?
The body-checked fixture registers arbitrary fresh IDs, preserving the source
fragment. Native counter arithmetic is modular; this first registration-only
probe does NOT assert a nonoverflow refcount or allow zero-based reclamation.
Original overflow-abort control and physical lifecycle remain future obligations.

Positive: body-proved creation, shared-reference issuance, distinct registrations,
actual native increment and receiver preservation. Native repeated/threaded
increments exercise the actual AtomicUsize. Controls: wrong atomic permission
must fail a commit precondition; duplicate fragment must fail Rust ownership;
a callback invoking another ordinary EventAtomic operation must fail ghost
purity; empty callback must fail the requirement to shoot the event. These do
not prove primitive adequacy. A physical Release/Acquire follow-up requires a
separate recorded contract/review and missing-Acquire negative; no claim from
this Relaxed-only result.

Stop: preserve first failure; two equivalent semantic failures require review;
third requires structural change, never increased prover timeout. Proof starts
elevated, shared flock, one prover, 1024 MiB, current private Std. Preserve exact
source, native/translation/proof logs, Coma, full proof tree and TCB inputs.
