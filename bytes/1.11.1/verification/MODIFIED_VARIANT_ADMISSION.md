# Production variant admission record

Target: bytes 1.11.1, `verified,std`, stock pinned Creusot, x86_64,
normal-return scoped threads. Status: implementation/proof pending, NOT ADMITTED.
See MODIFIED_VARIANT_SPEC.md; this record does not complete arbitrary sharing,
mutable APIs or all feature/configuration obligations.

## Changed premise and bounded question

D05 permits reopening only on an actual representation/interface change.
The production entry now replaces implicit Clone(&self)/callback ownership with
Owner::share_pair(&mut self), borrowed ReadHandle views, an explicit CloseContext,
consuming ReadHandle::close and consuming Owner::close. Actual source lives in
`src/verified/`; canonical physical bridges are included via
`src/verified_ownership.rs`. This is the user-selected route 1, not an annotation
or wrapper retry of the old API. No original Clone/Deref/Drop proof is inferred.

Question: can one compatible representation establish actual byte values,
sharing across real thread boundaries, read access through peer retirement,
receipt-derived exactly-one recovery and unconditional consuming parent cleanup?

Positive checks: native empty/spare-capacity/read-bound cases; one integrated
production translation/proof with every generated proof leaf closed; formal byte
result and unique-final-observer contracts; actual affine capabilities reach B3;
no new bytes protocol trust; exact source/configuration/archive correspondence.
Negative checks: missing Acquire must reject peer synchronization; missing reader
fraction/receipt must prevent final cleanup, including an empty reader; caller
contracts and Rust borrows must reject unmatched/missing completion where tested.
Different negative checks establish different claims (Rust typing versus VCs).

Stop policy: classify frontend errors separately from VCs, preserve failures,
review the interface after two equivalent VC failures, restructure at the third.
Do not increase timeouts or add wrappers to evade unchanged frozen decisions.
Astra is the architecture implementer; Luna audits resources/source/evidence.

## Required body and representation boundaries

| Actual boundary | Required relation |
|---|---|
| Owner::new -> B1/FrozenOwner | Input Vec model becomes initialized physical slots; Vec owner consumed |
| Owner::share_pair -> B4/lifetime split | Both real slices equal input contents; half anchor plus two quarters of same lifetime |
| ReadHandle::read | Index-bounded result is that actual input byte |
| ReadHandle::close -> native retirement | Unique ticket consumed; receipt last flag matches actual Release RMW; nonlast returns no peer fractions |
| Final retirement -> Acquire | Actual peer fractions extracted through deferred acquire witness only after native fence |
| Consuming context finish | Matched authoritative receipt fragments derive XOR and completed retirement |
| Owner::close -> B3 | All fractions reunited, full lifetime ended, actual full region/recovery consumed once |
| Scoped callers | Real spawn/join contracts compose byte and cleanup postconditions; peer-retirement witness keeps another reader live |

The atomic primitive is unchanged: src/verified/atomic.rs and
verification/probes/weak-native-publication/src/primitive.rs have SHA256
`dc94853b3009ccedce53d4f1edb41f85ab9b317e6cc651ec815d6b5f8259968e`.
The existing B1/B3/B4 physical contracts, stock thread contracts, synthetic
lifetime/resource algebra and weak-memory primitive assumptions remain explicit
TCB. Their adequacy is not proved by counting this variant's VCs. Mutable B4 is
not exercised in this admission; D01 remains frozen and a later exclusive model
must reject its preserved counterexample.

## Results

Pending. Entry routing regression alone: original native default passes1256 tests
in17 suites and original no_std check passes. See modified-variant-evidence/
entry-regression.json. These are compatibility observations, not variant proof.

### First production proof (saved failure)

Ghost-token representation translates. Native tests pass (48 scoped lifecycle
cases plus the primitive race test). The first complete prover run exits1 with
48 Coma/proof JSON files and3 null leaves: root allocation_ops deallocate_u8,
reallocate_u8, and Owner::close at14/15 split obligations. All other results are
component/body results, not admission. The root allocation_ops declaration was
then excluded from the modified entry: canonical raw_vec already invokes that
native helper inside the existing physical primitive boundary. This does not
move a bytes protocol into trust or remove its body obligations. Owner cleanup
is still required.

Archive `runs/failure/production-first/evidence.tar.gz` SHA256
`01fbeb3b3ee405ed73ee17066ec16e8083c5af8b200df92eb0a20806c48f0a56`: root
independently verified all121 member hashes,48 Coma files,48 proof JSON and3
null leaves (including tactic children arrays). The captured source precedes
subsequent repairs; it must not be advertised as matching the repaired source.

### Concurrent lifecycle positive (46 files)

The native root-helper wiring fix plus stock PositiveReal::ext_eq at the full
lifetime reunion close the previous failures. The equality lemma has a stock
body and introduces no project trust. Actual production verified,std proves46
files, zero null leaves, including the read-value/XOR scoped caller and the
unconditional consuming B3 cleanup. The exact matching native run passes.

Positive archive SHA256
`da680207c7f979330b2e1f7ca63d8fb8881c72a9543a1583a2d924bc88e3571c`, independently
audited117 members,46 Coma/46 JSON and zero null leaves. Source snapshot is the
concurrent witness before the ordered caller was added. This is a validated
increment; admission remains pending ordered peer retirement and negatives.
The low-level Owner/ReadHandle/CloseContext/Closed types are crate-private, so
external callers cannot mix ghost-only receipts or bypass their helper contracts.
