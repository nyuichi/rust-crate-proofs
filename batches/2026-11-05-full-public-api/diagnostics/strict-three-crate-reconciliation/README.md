# Strict three-crate API reconciliation (original ledgers preserved)

This snapshot audits exact versions endian-type0.2.0, nohash-hasher0.2.0 and
normalize-line-endings0.3.0 against existing nightly2026-09-17 rustdoc JSON,
expanded source, original manifests, and later evidence. It does not change any
proof, source artifact, published report or API claim. Existing inventory files
remain unchanged; input-inventory-hashes.json pins them. Raw rows, raw impls,
source aliases/spans and proposed scoped type substitutions are retained in
strict-rows.json. This analysis is not a new compiler canonicalization schema.

## Corrected counts and their meaning

| Crate/configuration | Preserved ledger | Ordinary signature candidates | Extra consumer nightly gate required | Known inapplicable | Derived std excluded | Applicable candidates including gated |
|---|---:|---:|---:|---:|---:|---:|
| endian default/all |174|126|2|0|46|128|
| nohash std default/all, debug/release |199|159|30|10 alias specializations|0|189|
| nohash no-default, debug/release |50|42|8|0|0|50|
| normalize default/all |86|65|15|6 unmet extra bounds|0|80|

These are signature/impl candidates **under their retained type, lifetime and
callback bounds**, not proof that every method applies to every T or arbitrary
callback. summary.json separately records constraint-bearing candidate counts:
endian22; nohash std151/no_std28; normalize77. Universal satisfaction of those
constraints is not asserted. The six known false normalize extra bounds are
separated explicitly; other generic instantiation obligations may remain open.
A nightly compiler alone does not enable unstable-library feature gates. All
Cargo features is not the same as all consumer #![feature(...)] configurations.

### nohash alias correction:199→189, not193

IntMap/IntSet fix their hasher to BuildHasherDefault<NoHashHasher<key>>. Old alias
projection indiscriminately renamed HashMap/HashSet self types, including impls
specialized to RandomState. Ten such records are not APIs on these aliases:
new, with_capacity, From<array> for each alias (six ordinary cases), plus new_in
and with_capacity_in for each alias (four allocator-feature cases). Enabling
allocator_api cannot repair the distinct hasher type. All ten exact local IDs,
raw owners/trait args, alias definitions/configurations and mismatch reasons are
in inapplicable-alias-exact-rows.json. The old rows are retained, not deleted.

The interim193 estimate removed only the first six; inspecting the allocator
specializations found four more.189 is the final applicable-candidate count.
The historical sequence is recorded explicitly rather than silently replacing
one claimed final denominator with another. Small compile-only fixtures confirm
valid alias default/with_hasher calls and expected E0599/E0277 rejection of all
six ordinary wrong-alias calls on concrete u64 keys; no RNG/unsafe execution or
heavy build was used. The four gated cases are rejected structurally by the same
explicit RandomState self-type mismatch, without enabling a new nightly API.

### Blanket binder instantiation is not payload conversion

Synthetic rustdoc blanket rows specialize `for` to a concrete type but retain
blanket_impl's template binder. Reusing its name T as the wrapper's payload T
is incorrect. For example, old `<NoHashHasher<T> as From<T>>::from` means
**reflexive From<Self>**, not construction from the payload. Correct display is
`<NoHashHasher<T> as From<NoHashHasher<T>>>::from`; Borrow/BorrowMut similarly
borrow Self, not payload T. Into/TryFrom/TryInto constraints also need the
concrete Self substitution even when displayed API text stays the same.

blanket-identity-corrections.json and strict-rows.json retain exact raw patterns,
scoped binder→concrete-self terms, alias hasher/allocator bindings, instantiated
trait args and constraints. Substitution is one-step, so target payload T inside
a concrete Self term is not recursively captured by the unrelated std binder T.
Raw template parameters and target-source contexts remain separate. Alias
bindings are restricted to generic names actually present in the impl owner,
not blindly applied to unrelated method generics. More elaborate bound solving
remains conditional; these analysis displays are not used to alter public keys.

## provided_trait_methods versus actual overrides

Rustdoc provided_trait_methods lists trait defaults even when an impl overrides
them. The original generator already checks actual method names first. Nohash
std snapshots have23 shadowed declarations across10 impls (including ten native
Hasher writes and std collection overrides); no_std has its own recorded subset.
These defaults are not extra executable methods and are **not subtracted again**
from199/50. shadowed-provided-methods.json preserves raw impl lists, actual items,
and checks that no inherited-default duplicate was emitted for those overrides.
Endian/normalize snapshots have no such overlaps in the checked records.

## Derive and non-callable distinctions

Only automatically_derived standard/core/alloc trait impls are excluded:
endian46 callable-ledger entries. Handwritten macro-generated std trait methods
remain included. Across these exact configurations no nonstandard-derived
callable was found; any such row would remain in scope rather than be discarded
merely for a derive attribute. Types, aliases, associated types and marker impls
remain separately inventoried and are not added to callable-function counts.

Normalize's rev/rposition/cycle/copied/cloned/__iterator_get_unchecked have
unsatisfied original opaque Self/Item bounds. Their six rows are preserved as
not callable, not counted among80 candidates. nth_back is not supplied by an
original DoubleEndedIterator impl and is not manufactured by adding an adapter.
The new bounded nth/count/last and position/find evidence does not change that.

## Existing claims and supplementary evidence

machine-claims-unaffected.json independently binds every existing14/66/1 mapping
to exactly one crate-local manual/macro source row, with exact typed identity,
spans and generic bounds. Wrong aliases and external blanket rows are unused.
Configured legacy nominal keys and full typed Rust identities are kept separate,
with explicit spelling normalization; this is source binding, not a claim those
strings were text-identical. Original source/manifest archive equality also
passes for all three source artifacts. No machine claim is removed or added.

- endian66 retains the original bitwise/explicit scalar-model From premises;
  its40 concrete integer byte APIs have separate all-input Kani evidence, while
  generic slice6 only have integer-specialization evidence. External16 (including
  gated2) remain separate unproved std boundaries, with corrected blanket types.
- nohash14 remains Default/Clone/Hasher12. Debug has u8/fixed-writer Kani in
  four std/no_std profiles; clone_from has u64 publicly reachable-state Kani,
  with original-public-observer and read-only-copy-observer stages distinguished,
  plus separate Lean default-model composition. These partial domains do not
  establish arbitrary T/callbacks or close external174 applicable candidates.
- normalize1 remains the existing **adapted translation/finite-list** mapped
  contract, not an original arbitrary-iterator proof. Separate original constructor
  initialization and finite0..4 Kani next/default/search supplements are retained
  per row without multiplying independent machine claims. All inherited APIs
  outside those finite predicate/iterator domains remain independently open.

Rust source/compiler versions and std boundaries of each supplemental packet
remain authoritative; Kani's2026-08-21 std bodies are not automatically proofs
of all2026-09-17 or other std versions represented by the inventory.

## Reproduce and publication effect

`python3 reconcile.py` regenerates the new view from pinned unchanged inputs;
`python3 check-machine-bindings.py` rechecks the14/66/1 exact source bindings.
The compile-only fixture and its positive/expected-negative logs are retained.
Existing proof bodies/publications are unchanged. PUBLISHER-HANDOFF.md identifies
where old counts need a Git-ledger correction; inspected report artifacts have
no199/184 remaining-count prose needing a proof rerun solely for this correction.

## Collection-scope addendum

Counts189/50/80/128 reconcile the preserved input ledgers. Those input commands lack an explicit --document-hidden-items option; these counts are not established as an exhaustive catalogue of hidden/nightly APIs. Hidden and private filters differ. The later base16 inventory explicitly enables hidden-item documentation, so its population must not be silently compared as identical. No new collection/build is claimed here; existing14/66/1 source bindings and theorem domains are unchanged. Exact evidence and the append-only correction are in ../base16ct-strict-reconciliation/THREE-CRATE-COLLECTION-SCOPE-ADDENDUM.md.
