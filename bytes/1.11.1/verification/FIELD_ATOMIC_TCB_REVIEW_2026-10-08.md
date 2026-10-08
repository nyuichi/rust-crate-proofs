# Actual atomic field boundary review — 2026-10-08 UTC

Scope: the private original Shared source-correspondent probe, default core
atomic alias, with sc-drf disabled. This is a reviewed generic TCB, not a proof
of the native atomic memory model or the public bytes ownership protocol.

The adapter constructs the actual core AtomicUsize and returns one affine
Perm<ModelAtomic>. Its model is per-object identity: it follows an ordinary move
into Shared and must distinguish different live atomic objects. It is not a
storage-address model. This native/model correspondence is assumed generically;
the body-proved constructor establishes that its returned permission names the
actual ref_cnt field after the move, and the typed Box permission names the same
Shared allocation. Constructor evidence: commit 3ba052a6, 34 Coma/proof files,
110 discharged leaves, selected constructor nine discharged leaves.

Every event borrows the actual native atomic field and requires model equality
with the supplied descriptor. The restricted FnGhost callback receives protocol
state and the matching typed committer. There is no independent state getter or
ghost event opener. Ordinary event operations cannot be called from that ghost
callback. Orderings are native Relaxed increment, Release decrement, and Acquire
load. Wrap semantics are admitted by the generic primitive; the bytes body must
rule out inappropriate zero/wrap states.

Luna independently compared the callback, Send/Objective bounds and committers
to the installed creusot-std 0.13 sources. An initial concern about duplicate
binding of Copy protocol state was withdrawn after concrete review by root and
Astra: Copy state cannot duplicate an affine atomic permission/history. Universal
callback preconditions do not supply a relation between an arbitrary committer
load and a stale integer in Copy state. Multiple metadata-only invariants may
coexist; the boundary must not claim a unique descriptor per atomic. The actual
bytes protocol carries affine permission and authoritative ticket resources.

Send and Sync bounds match AtomicInvariant (S: Send, and S: Send + Objective for
Sync under Creusot). These generic markers do not prove Bytes or Shared markers,
and AtView makes subjective state objective without making raw-pointer-bearing
payloads Send. No arbitrary-last-thread transport proof is inferred.

Publication is a separate body obligation: callback must use the retained atomic
permission and the typed load/store rules. Relaxed does not acquire a published
payload; final recovery needs the actual Acquire observation. The initial
bounded clone quota restricts the selected clone to before any retirement, so it
does not establish arbitrary concurrent clone/overflow or a Relaxed RMW rule for
preserving previous Release publication.

No bytes registration, count-to-handle, last-owner, exactly-once reclamation or
destructor theorem is trusted here. Frozen rejected architectures remain frozen;
this generic field boundary does not reopen them by itself.

## Selected source integration review

The scoped control-event interface now accepts the actual affine LifetimeToken,
not a metadata-only lease projection. A typed FullBorrow protects the actual
Shared allocation; its AtomicField implementation body proves that projection
selects ref_cnt. A decrement receives the same owned token in the callback after
the last native field access. No native reference escapes, and no alternate
Snapshot materializes an atomic permission. This primitive remains generic TCB;
its callers prove the bytes-specific registration and last-owner rules. The
rejected TokenLease draft and corrected static review are preserved separately.

The readonly AtomicPtr binding consumes the unique model write permission at
bind_read_only; get_mut borrows that readonly descriptor and copies the actual
value, matching the default native with_mut shim. Its no-store-history
interpretation is generic TCB, analogous to permission-consuming Std AtomicPtr
into_inner; it does not infer pointer identity from a number. BoundPtr::as_ptr
exports a body-proved address observation only. Typed control deallocation
consumes the recovered permission for the actual pointer and Layout::new<T>.

The reviewed subset of creusot-std0.13 inputs is byte-compared in the source
archive. The registry package and pinned Creusot source have different VCS SHAs;
no full-package identity or proof of native memory-model adequacy is claimed.
The current43-file positive replaces the constructor-only coverage statement
above for this selected source leaf, without extending to public vtable/Drop.
