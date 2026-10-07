# Generic trusted-boundary workflow

User direction: 2026-10-08 (Asia/Tokyo). Applies to bytes 1.11.1 only.
The user selected reviewed generic trusted contracts for missing verification
support and directed consulting analogous creusot-std cases whenever blocked.

## Before adding a boundary

1. Search the actual pinned creusot-std source AND its upstream tests/examples.
   Record the version/source hash, exact API, contract, native operation and trust.
2. Distinguish a missing public adapter, a missing specification, an unsupported
   frontend construct, an unproved caller body, and a counterexample to the rule.
3. Copy the relevant contract discipline: permissions, ward identity, lifetime,
   invariant restoration, Objective/AtView constraints and native memory ordering.
   Copying only the method name or `trusted` annotation is insufficient.
4. State strong input/output/effect contracts before proving callers. Explicitly
   list the generic assumption and its native interpretation. Keep bytes-specific
   registration, refcount, access, last-owner and destructor reasoning in callers.
5. Check a bounded caller and discriminating negatives before broad integration.
   Tests/proofs of callers validate composition, not the trusted rule's adequacy.
6. Specify the future Std/tool replacement with the same interface; isolate trust
   so caller guarantees remain meaningful. The primitive implementation itself
   may remain TCB under the user's authorization. Do not weaken guarantees merely
   to produce successful caller proof counts.

## Concrete analogues examined for shared atomic updates

The active tool source is pinned at Creusot commit
`318615be3b8bbc60d1f6d52469ba5c0bdebed4f1`; the private Std overlay is prepared by
scripts/prepare-proof-std.py and has its own manifest. Source snapshots in each
probe determine the exact version used for a proof.

| Source/API | Trusted guarantee used by clients | Relevant limitation |
|---|---|---|
| upstream tests/should_succeed/mutex.rs: lock, guard deref/set | Runtime lock returns a guard tied to predicate I; guard reads/writes maintain I | This is a trusted runtime Mutex wrapper/example, not a proof of Mutex internals, arbitrary DerefMut, poisoning or Drop effects |
| ghost/invariant.rs: AtomicInvariant::new/open | Affine state is encapsulated, public projection preserved, Protocol restored by callback | Shared opening requires namespace Tokens; Objective bound for weak cross-thread sharing |
| std/sync/atomic.rs: native-operation wrappers | Exact atomic ward, event value and ghost callback completion | Native callback is erased; its proof does not supply mutual exclusion or permission by itself |
| std/sync/committer.rs: shoot_load/shoot_store | Actual atomic permission/history and synchronization-view effects | Requires matching Perm; store completion is once; Relaxed is not Acquire |
| std/sync/view.rs: AtView | Subjective physical payload can be carried objectively | Access requires a justified current SyncView dominating publication; a Snapshot of a view is not such a witness |
| ghost/resource.rs and resource/auth.rs | Affine RA authority/fragments and checked local updates | Resource/fragment identity must match; numerical IDs are not ownership |

## Restricted event interface requirements

The changed-premise design uses a new encapsulated interface, not another opening
method on an existing AtomicInvariant. Only an ordinary actual atomic operation
may expose a temporary mutable proof state to its FnGhost callback. There is no
public ghost open/open_const, no namespace-Tokens alternate entry, no access to
hidden state through a shared reference, and no cloning of the owning atomic.

Callback state must preserve its public binding and Protocol. Its event must be
associated with the exact atomic and matching permission; event completion cannot
be omitted or repeated. Reentry via another ordinary atomic operation must be
rejected by ghost checking. Objective shared state and the existing weak ordering
and AtView rules remain mandatory. No hardware lock is added or claimed: atomic
linearization and the erased ghost transition are part of the explicit generic
TCB interpretation, as distinct from runtime Mutex exclusion.

The old open_at(existing AtomicInvariant, committer, callback) design remains
rejected: a callback could reopen the same invariant via existing Tokens.
Negative controls must target the new interface, not rely only on that rejected
old signature. Original automatic Drop, immutable read lifetimes and complete
bytes refcount protocol remain separate obligations until actually connected.
