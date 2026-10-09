# AO: original owner retires before its surviving promoted child

Development increment after audited AN `5f3d20a3005b727ace827685d0601d791d180f78`.
The complete original bytes 1.11.1 architecture remains **NOT ADMITTED**.

The native witness constructs a nonempty Box-backed Bytes, clones it once,
drops the original at inner scope exit, reads the surviving child, saves the
returned Vec, and drops the child on normal return. The proof includes both
native tag parities. Native execution tests corroborate the client behavior;
they do not establish allocator parity or prove the Rust memory model.

The distinguishing transition consumes the complete original PromotionScope,
including its owned pointer history and root resources. It returns only a
DetachedScope containing an affine observation cursor. The existing proved
State::on_release body seals the actual root recovery authority into private
state. The last child's release and final Acquire recover that authority and
perform both physical frees. No recovery capability, root core, pointer
permission, or private state is hidden inside the surviving scope. Its logical
observations do not extract resources. The nonfinal helper is stated for an
actual ledger length greater than one; the client derives its two actual owners.

The first diagnostic positive run passes 125 targets / 983 prover leaves /
0 null and 0 structural leaves. It deliberately skips the correspondence gate
and is not canonical admission. The restored full source/native checker passes; all 31 main and 52 native
structural controls reject. Seven semantic controls leave real null leaves,
respectively: omitted root recovery publication 2, original Drop 1, survivor
Drop 1, cursor handoff 5, final Acquire 2, payload free 2, control free 2.
The three generic feature controls also affect retained Shared bodies and
are single defects, not independent extra coverage. Missing free controls
use uninhabitable conjure paths; they are proof-sensitivity evidence rather
than native event-absence theorems.

Five frontend controls reject three duplicate consumptions with E0382,
survivor retirement during its live read with E0502/E0505, and incompatible
Drop adapter calls with E0061. Frontend archives exclude proof tasks and all
solver caches. The restored feature-free all-target gate now passes 125/983/0 with
correspondence status 0, no features/exclusions/diagnostic/source controls.
Canonical archive `5cf0ffb4ae25dd3d8cb757677a5a43032e926c0eac4dbe3822a9c482206238b9`
has 968 hashed members, including 110 private-Std files and all four actual
Cargo compiled input artifacts plus their receipt. Independent canonical
archive reconstruction passed both checkers and all 31/52 controls, archive
member/source/Std/tool/compiled-input hashes and five frontend snapshots. This paragraph is updated
after capture; executable proof/checker inputs remain the captured versions. See evidence
and TCB.md.

The inherited AN prefix and support sources remain byte-pinned. Generic
physical, synchronization, provenance, callback erasure and normal Drop
compiler boundaries are explicit. Root retirement and surviving-child ownership,
lastness and reclamation laws are proved bodies, not new trusted contracts.

Concurrency, CAS losers, unwind, allocation failure, arbitrary escaping clients,
other original APIs and build configurations remain open. A successful AO gate
does not imply full-crate verification. After each audited published increment,
ask Astra 「次何するのがいい？」 and execute the next recommendation.

The pinned tools live outside evidence archives. Captures include source,
configuration, tool hashes, private Std, native MIR and actual Cargo compiled
inputs. Why3 executes under the common lock, one prover and 1024 MiB, with
sc-drf disabled. Archive replay of location-bound Cargo OUT_DIR is documented
rather than represented as a portable standalone toolchain.
