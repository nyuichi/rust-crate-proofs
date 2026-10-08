# Closed protocol issuance cursor — bounded prerequisite

This is a distinguishing experiment for AD/AF, recommended by Astra. It is not
an alternate production Bytes API, actual Bytes admission, arbitrary concurrent
closure, implicit Drop, or unwind correctness. It keeps the prior failed AD
client and its counterexample immutable.

## Claim and interface

`closed_sparse_lifecycle` starts one fresh registry, issues two children, retires
a sibling and its parent, issues another child from the surviving child, and
retires both survivors. The returned tickets, body-proved singleton/insert/remove
relations and cursor observations establish final lastness. There is no clone
quota or presumed numeric ID. Recovery still requires the actual Acquire load
and returns the wellformed payload plus a full LifetimeToken, which is ended.

`State::observe` is exactly `(live_map, next_id)`. An opaque, affine erased
`ScopeCursor` exposes only a logical Snapshot of this resource-free observation
and its model/public descriptor. It cannot yield State, Perm, LifetimeToken,
Recovery, AtView or physical authority, and has no Clone/Copy/Send/Sync or public
setter. The constructor consumes the one initialized State once, returning its
encapsulating `ScopedEventAtomic` and a matching cursor. The native atomic is
private and this facade is distinct from all old unscoped adapters.

The generic scoped event contracts retain the existing protocol, public/model,
Committer, weak-order arithmetic and callback/shot-store obligations. They add
pre-state observation equality with the incoming cursor and final callback-state
equality with the outgoing cursor. The callback's bytes-independent sparse
protocol bodies prove map updates, native old count = prior map cardinality,
and actual synchronized typed recovery. No last-owner or recovery implication
is provided by a trusted cursor law.

## Explicit TCB and scope

The NEW generic boundary interprets this affine erased cursor as the complete
serialized observation of a noninterfering closed scope. Every effect on this
invariant must use the same cursor; no external/unscoped event, reentrant call,
unknown callback, thread or raw escape is admitted. In particular the failure
branch of increment preserves the observation only because the closed scope
excludes other mutations. This boundary is not sound for arbitrary mixed callers.

The independent `check_closed_scope.py` accepts one closed straight-line client
language, checks its exact source and root module/facade interface routes,
reconstructs the seven ordered events and consumed ticket locals, and rejects
unknown syntax or remaining/escaped tickets. It does NOT derive generic facade
callback bodies, macro/codegen erasure or native MIR correspondence. Their native
interpretation and noninterference adequacy remain reviewed generic TCB, along
with the existing RawAtomic/event/weak-memory/resource primitives and Std
contracts. Native execution is a check, not proof of this TCB.

Pinned Std's erased Ghost channels and operation-bound invariant callbacks are
interface precedents, not a complete-history theorem. `Plain: Copy` does not
justify materializing this observation or making the cursor copyable. Removal
requires supported effect/history translation with the same contracts and
retained negative controls. Original Bytes integration must separately check
actual constructor extra erased result and Clone/cleanup extra erased argument
source correspondence. Cursor must stay outside Bytes and native Clone(&self)
remains unchanged. Implicit Bytes Drop must be connected to the published MIR
effect elaboration before it can be admitted.

## Evidence and reproduction

Run `./run-proof.sh` elevated: shared `/tmp/itoa-creusot-proof.lock`, pinned
nightly/Creusot/private Std, sc-drf disabled, one prover and 1024 MiB. It checks
closed-source correspondence, regenerates all proof targets and excludes none.
`./native-check.sh` executes the same client with erased ghost channels.
`python3 check_checker_controls.py` checks fifteen structural mutations.

Initial run: 38 files / 251 actual prover leaves / four nulls, in register and
retire interface goals. Exact failing source/tasks are preserved in
`cursor-v1-failed-2026-10-09`. The old State live_count observer was opaque outside
its module, and the registration contract did not export fresh-key absence.
Truthful body-proved postconditions on map cardinality and fresh insertion fix
this interface; no additional assertions, assumptions or trusted bytes laws were
used. Positive v2 proves 38 files / 219 prover leaves / zero nulls.

Semantic controls are separate diagnostic captures: false unchanged-map summary
must fail register; omitted final Acquire must fail recovery; an extra registered
but unretired owner must fail closed-source checking and semantic final recovery.
Diagnostic proof tasks are never positive coverage. Immutable evidence includes
all exact Coma/proof pairs, source dependencies outside this probe, private Std,
tool/config pins, logs and a SHA member manifest. The final restored capture and
independent audit supersede the intermediate positive for current source checks.

Control outcomes: false summary 38 files / 232 prover leaves / one null in
register; missing Acquire 38 / 222 / one null in retire; extra unretired owner
38 / 246 / one null in the client plus checker rejection. The extra-owner
variant is captured while active and its script always restores the canonical
source. Pure observation -> State and -> Perm controls are E0308 Rust type
rejections before any VC/prover; their sources/logs are in generated/type-controls.
These are different failure phases and none is counted as positive proof.

Capture timing: intermediate archives freeze their actual proof input modules
and tasks. During diagnostic development, the launcher/checker/docs were being
refined; they are not receipts that every archived launcher byte was executed
in that earlier run. The final restored run freezes and executes the final
launcher/checker, then receives the independent member/task/source audit.

Final restored proof: 38 files / 219 actual prover leaves / 0 nulls, no exclusions; native closed client and fifteen structural controls pass.
