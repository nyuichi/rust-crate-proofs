# First promotable Clone and closed cleanup lifecycle — 2026-10-09

D2026-10-09-AL, directed by Astra after audited/published AK. Client-only nonempty
Box premise, both parities, closed single-threaded first CAS, explicit cleanup.
PromotionScope directly receives constructor-owned raw physical resources and
unconsumed root AtomicPtr permission; no logical resource getter, readonly
history after mutation, guessed issuance, trusted promotion success or count1
plus fictional increment. First prove owned-pointer CAS and initialize_pair,
then actual native client. No promotion autoDrop/concurrent loser/full admission.
Canonical gate: 112 files / 808 prover leaves / zero nulls, complete target
closure, no defect features, correspondence status 0. This admits only the
bounded lifecycle below, subject to explicit generic TCB; original full-crate
architecture remains NOT ADMITTED.

Development prerequisites (not lifecycle admission):

- Actual count-2 pair initializer: 76 targets / 450 prover leaves / 0 nulls.
  `pair-positive-diagnostic-v3` archive SHA-256
  `09387c3550e92ad212f13d5b8277d64fe4f4d1cc76b5fff54befd75b72e2f3e5`.
  Two real affine tokens issue IDs 0/1 and fractions 1/2, 1/4, with pool 1/4.
  Independent archive/source audit: `PAIR_PREREQUISITE_AUDIT.md`.
- Owned-pointer first strong CAS: 79 targets / 482 prover leaves / 0 nulls.
  `cas-positive-diagnostic-v1` archive SHA-256
  `30586cdeab09d8d12158a9ac899c227ed13acaf1399a60cd43ece266827fef98`.
  The success follows from the body proof over the owned initial history and
  the strong failure deep-model inequality. The updated permission retains
  both old/new history; it is not converted to a read-only binding.

Both captures deliberately skip the not-yet-implemented client correspondence
checker, with structured `not_run` and diagnostic status 2. Neither establishes
the actual Box-to-Shared ownership transfer or admits a clone lifecycle. The
initial pair body failure and exact tasks are archived. The intervening frontend
type-error log is preserved as `evidence/pair-frontend-v2.log`; it is a log-only
diagnostic, not an immutable full input/proof capture.

Post-store owned-pointer prerequisites pass diagnostic 83/507/0 in
`cas-positive-diagnostic-v2`, SHA-256
`38dcaba847d610db040509177f0bea39764bd9a9def9d316c6ca86d45035f5b0`.
The terminal get_mut bridge returns a pure maximal-history timestamp only; it
consumes the permission and supplies no SyncView or implicit Acquire. Root
AsRef must retain its actual native shape (no invented atomic load).

Isolated generic pointer semantic controls (all complete mini-crate targets):

| Defect | Files / prover / null | Archive SHA-256 | Actual failed goal |
|---|---|---|---|
| Wrong CAS expected | 8 / 67 / 1 | a5ac22d79ac061a03f9a52e857a64795a9067ffc0df94e4d7790bec97645b98a | initial owned singleton must equal wrong expected singleton |
| Success callback omits store | 8 / 77 / 1 | 7f471e9a6468fd6b41aecc4fe65e1c4a017981049aaf99903248c04d1fe4165f | success Committer final shot_store |
| Re-seal updated history as old read-only value | 8 / 69 / 1 | 367ac1d07d4246cc90962bcfc5a4cecf26a1b8f8c2ce054d3b11f9c4f3a1eed1 | every updated history entry must still equal initial |

Inputs, every translated mini target, proof trees and logs are immutable under
`evidence/cas-control-*.tar.gz`; printed null tasks are saved alongside. These
exercise generic boundary prerequisites and are not native Bytes lifecycle
negative controls. A first mini launch lacked its local why3find configuration;
the infrastructure-failure log is retained separately and counted as no solver
rejection. The fixed invocation reused the exact pinned configuration. Mini
working `verif/` is removed after archival to prevent stale negative results
being confused with the later canonical integrated gate.

Mini archive replay layout: restore `inputs/repository/` at a fresh repository
root, copy archive `probe/` to
`bytes/1.11.1/verification/probes/original-promotable-first-clone-2026-10-09/`
inside that root, and restore the pinned tools/Std installation under the paths
recorded in the activation/configuration. The mini path attributes then resolve
to the captured `owned_pointer.rs` and `pointer_event.rs`. Captured backend
`.why3find` caches are hashed diagnostic data, not replay inputs; remove them
and `verif/` before invoking the mini `run-control.sh` to obtain an uncached
replay. The archived script removed `verif/` only. The current script also removes
`.why3find`, and future captures exclude backend caches. The old archives
remain untouched; this improvement has not been counted as an uncached replay.

Integrated translation v1 failed at backend Coma parsing before solver, due to
external postconditions referencing private State.pool. Immutable diagnostic
archive SHA-256 fcd0f8258c73d2ec362a51fabbca7038e1912e6c9ec714c381132e8dc9b55bc9
contains the full 112-target policy but only 12 empty result files; its 0/0
leaves are not a success statistic. The interface now exports only
`pool_fraction() -> Option<PositiveReal>` and preserves the quarter/full-sum
postconditions without exposing a lifetime token. A failed attempt to make the
private projection transparent was separately archived before the successful
opaque pure-projection frontend. Integrated v2 left six actual null tasks. Constructor-established address/parity
facts and a pure release-ledger cardinality postcondition resolved them without
new ownership trust. Diagnostic v3 passed 112/808/0 but retained checker status
2; it is not the canonical admission.

Native correspondence now binds the count-two strong-CAS winner and loser
MIR blocks/normal edges, correct pointer values, returned field order and typed
cleanup route. All 19 selected headers are distinct and checked. Structural
controls reject 24/24 targeted mutations, including branch placement changes
with their markers still globally present. This native checker is a
component of the final separate native-to-proof correspondence gate. The main
checker binds exact source contracts, complete bodies, saved vtables and three
actual runtime callback arguments, both root tag routes, physical effects and
the actual compiled OUT_DIR public_records.rs. It runs after translation and
before proof, and rejects all 25 targeted controls.


## Canonical scope and boundary

The source client uses actual nonempty `From<Box<[u8]>>`, first `Clone`, child
explicit cleanup, root `AsRef<[u8]>::as_ref(...).to_vec()`, and root explicit
cleanup, in that order. Both native tag parities are covered mathematically.
The native one-test witness checks lengths 1, 2, 31 and 256; it does not prove
both physical pointer parities. Nineteen distinct selected MIR bodies and the
normal CAS winner/loser edges are checked. Nested AtomicMut closure effects are
source-checked; their outer invocation is bound by selected MIR.

Promotion transfers the constructor's actual Recovery, full PhysicalRegion and
owned AtomicPtr permission. An actual initialized count of two issues distinct
real tickets with IDs 0/1 and exact fractions 1/2, 1/4, pool 1/4. The root keeps
its updated old/new atomic history. Only the freshly initialized child's
separate data field receives a read-only binding. Child retirement leaves the
real one-entry ledger; final root retirement consumes it, proves the actual
zero-entry ledger, acquires final recovery and returns both physical allocation
receipts. Typed storage free is not a theorem about arbitrary T::drop.

Explicit generic TCB comprises low-bit exposed-provenance roundtrip and equal-
pointer distance; Core-field/Std atomic model identity and callbacks, strong
CAS, exclusive get_mut latest-value interpretation; allocation/borrow/free
primitives; exact-item erased registration/invocation; compiler/source/MIR and
shadow correspondence. Their native interpretation and removal routes are in
the source interfaces and independent audits. Counter transitions, ticket
issuance, successful closed promotion, phase/resource transfer, lastness,
contents and both cleanup effects are body-proved. GetMut returns a pure
history timestamp, not a new SyncView or invented Acquire. No resource-valued
logical getter or Bytes-specific ownership/destructor axiom is introduced.

Concurrent CAS losers, independently escaping arbitrary clones, promotion
normal automatic Drop, panic/unwind, allocator failure, arbitrary API/mutation
composition and remaining configurations are outside this gate. The native
loser branch is retained and checked; its exclusion is proved only from this
client's exclusive singleton-history premise.

## Distinguishing controls and evidence

Seven full 112-target semantic runs mutate count=1, retain an unretired child,
use the wrong CAS expected value, choose the raw root-drop branch after
promotion, omit Acquire, omit payload free, or omit control free. Their actual
null counts are 1/1/2/1/2/2/2. Feature-based omissions affect both the new
promotion path and the imported Shared gate; their two failures are not counted
as two independent promotion defects. Missing-free templates use false-premise
Ghost::conjure branches, so they establish verification sensitivity to those
omissions, not a standalone theorem that missing storage receipts are impossible.
Printed tasks and archive-bound audit summaries distinguish the exact goals.

Five frontend snapshots record E0507 for an initial direct Ghost-field move,
E0382 for consuming/reusing the root permission as ReadOnly, E0382 for reusing
the moved physical region, E0382 for cleaning the same child twice, and
E0505/E0502 for holding a root slice across root cleanup. These are type
rejections with no proof artifacts; they are not failed Why3 VCs. The generic
stale-history mini control separately has a real semantic failed goal.

Canonical capture label is `promotion-canonical-v1`; its receipt and immutable
archive include all source/support dependencies, actual compiled record and
Cargo build receipts, private Std's 110 files, tool/configuration pins, selected
native MIR/test inputs, all translated targets, all proof trees and run log.
Independent audit recomputes hashes and verifies complete coverage. The seven
negative and five type inputs are independently captured, never overwritten.
For missing-control-free, `generated/missing-control-free-executed-run-proof.sh`
records the pre-change launcher actually used; its current launcher snapshot
contains the subsequent checker-order change and is not claimed to have run.

For a fresh replay, restore `inputs/repository/` at a repository root, then
copy archive `probe/` into this exact relative probe directory. Restore the
pinned installation/private Std/config from captured inputs and installation
manifest; binaries are pinned by digest and are not archive payloads. Activate
`/workspace/bytes-proof-tools/activate.sh`, clear `verif/` and backend caches,
regenerate native inputs with `capture-native.sh`, then run `run-proof.sh`
elevated (Why3 socket requirement). The launcher regenerates build outputs,
requires one current Cargo build fingerprint and exact compiled record,
checks all source/MIR routes before proving all targets with one concurrent
prover process (sequential solver fallback),
1024 MiB and sc-drf disabled. Absolute live Cargo paths in build receipts are
observations of the captured run; replay regenerates them, not stale-path
inputs. Archive-only audits compare captured bytes and hashes instead.
