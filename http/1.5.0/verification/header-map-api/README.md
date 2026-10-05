# HeaderMap actual-source API leaves

> **Checkpoint note (2026-10-05):** This page records the earlier
> seven-method/Vec-bound run, plus later Entry accessor and conditional
> `IterMut::next_unsafe` results and the actual `try_insert_entry` helper.
> The IterMut nested split arity has been independently checked. The public
> Iterator graph proof and two generic `len` layout obligations remain open. See
> [the HTTP checkpoint](../CHECKPOINT_2026-10-05.md) and the run manifests.

This harness includes the production `src/header/map.rs`,
`src/header/map_capacity.rs`, `src/header/name.rs`, `src/byte_str.rs`, and
`src/bytes_model.rs`. It uses the published HTTP 1.5.0 `HeaderValue` as an
opaque field type. The harness-only `http_map_api_leaf` feature gates unrelated
entry/insertion, raw-removal, iterator, formatter, and adapter families so the
selected methods remain the actual production bodies. Normal HTTP builds do
not enable this feature.

The selected methods are `HeaderMap::new`, `Default::default`, `len`,
`keys_len`, `is_empty`, `clear`, and `capacity`. The fresh run proves 47 of 49
solver leaves. `new`, `default`, `keys_len`, `is_empty`, `clear`, and `capacity`
pass all 20 leaves. `len` proves both addition-overflow obligations, but two
generic element-size preconditions remain open.

The actual `HeaderMap::len` body calls a nontrusted `cfg(creusot)` helper that
derives the allocation bound through the existing slice permission and
`PtrLive` invariant. The helper's own postcondition VC passes; its five other
VCs are calls/specifications for the existing permission chain. The four Vec
consumer probes contribute four own body VCs and seven supporting
call/specification VCs. This adds no new Vec specification. The two remaining obligations are to establish
`size_of_logic::<Bucket<T>>() > 0` and
`size_of_logic::<ExtraValue<T>>() > 0` for arbitrary `T`. Current generic ADT
size modeling does not connect a live stored element or positive Vec length to
those layout facts when `T` may be uninhabited. The harness adds no map
invariant or blanket nonzero-size assumption. In the actual `HeaderMap::len`
artifact, `vc_len_T` contains six successful own body leaves and two open
leaves for those helper preconditions; four separate call/specification VCs
pass. Thus 47/49 is the full selected target closure, not an own-body count.

Across all 12 targets, the goal-name audit separates 19 own-body leaves (17
proved, two open in `HeaderMap::len`) from 30 supporting, callee, or
elimination leaves (all proved). The selected seven actual `HeaderMap` methods
account for 14 own-body leaves; the remaining five own-body leaves come from
the permission helper and four Vec probes.

This is a selected actual-source method run, not an integrated proof of the
complete `HeaderMap` implementation. The helper and its calls are
`cfg(creusot)`-only; normal runtime code is unchanged.

## Entry accessor checkpoint

The focused archive at
[`evidence/run-2026-10-05-entry-accessors/manifest.json`](evidence/run-2026-10-05-entry-accessors/manifest.json)
records a fresh named-profile emission and proof for these six actual-source
method bodies:

| Methods | Own body leaves | Vec index support leaves |
|---|---:|---:|
| `VacantEntry::{key,into_key}` | 2 | 0 |
| `OccupiedEntry::{key,get,get_mut,into_mut}` | 4 | 4 |
| Total | 6 | 4 |

All 10 leaves passed. Each target's COMA-derived root set has the same arity
as its proof JSON: the two vacant projections each have one direct own task;
each occupied projection has one direct own task and one `Vec<Bucket<T>>`
index/index-mut support task. The proof trees are flat, with no nested or
unresolved child nodes.

The occupied accessors require only `entry.index < entry.map.entries.len()`.
Their postconditions connect the result to the selected storage slot. This
proves the conditional indexed projection bodies; it does not prove that a
selected slot is occupied, that it matches a lookup key, or that a valid
`OccupiedEntry` was produced by the map's public lookup/entry path. The
`http_map_entry_api_leaf` feature is enabled only in the verification harness
alongside `http_map_api_leaf`; the normal crate configuration is unchanged.

## Insertion helper checkpoint

The actual private `HeaderMap::try_insert_entry` body is recorded in
[`evidence/run-2026-10-05-try-insert-entry/manifest.json`](evidence/run-2026-10-05-try-insert-entry/manifest.json).
Its exact Coma task has one own body root and three Vec/error-constructor
support roots; all four pass, and the no-preprocess Why3 printer emitted those
same four roots. The postcondition specifies the bounded append/error result
and frames `mask`, `indices`, `extra_values`, and `danger`.

This helper result does not establish that its callers update the index table
correctly, nor does it prove lookup, insertion reachability, or the global
Robin Hood table invariant. The archive uses the named actual-source
`http_map_api_leaf` + `http_map_entry_api_leaf` profile. The earlier commands
below reproduce the seven-method run; use the exact commands in the manifest
for this separate helper snapshot.

## Lookup-loop helper checkpoint

The fresh actual-source proof for [`HeaderMap::find_with_hash`] records one
own body leaf and eight call/specification support leaves, all successful, in
[`evidence/run-2026-10-05-find-with-hash/manifest.json`](evidence/run-2026-10-05-find-with-hash/manifest.json).
Its preconditions require the map's storage/table representation predicate
`header_map_find_ready` and a nonempty map. A found result carries a bounded
probe and bucket index, matching bucket/slot/input hashes, and the actual
`HeaderName`/query `PartialEqModel` relation. `None` remains unconstrained.

This is a one-helper result. It does not prove that constructors or mutations
establish `header_map_find_ready`, lookup's absent-key completeness, or the
public `get`/`get_mut`/`get_all`/`contains_key` wrappers. Those remain separate
lookup and map-invariant work. The Creusot definition retains the native
generic equality body and adds only the model type bounds needed to translate
that existing comparison; the normal build keeps its original generic
signature. The named `http_map_find_api_leaf` profile excludes unrelated
map families only in the verification harness.

## Lookup/getter body batch on current source

The accepted resumed run is recorded in
[`evidence/run-2026-10-05-map-lookup-getters-resume/manifest.json`](evidence/run-2026-10-05-map-lookup-getters-resume/manifest.json).
It freshly emitted 13 COMAs from `map.rs` SHA
`ca22bfb5673eba84281bf3b1738e831a15b94cfcb99b0f8e16cab4fa46f9ce6e` and
`name.rs` SHA `8c4633906a8ecd6e30e7eb75f8ace00c9b9f89d5e5f4c325f7e6b10a4eee94d6`,
then proved them through the shared HTTP wrapper. All 43 direct Why3 roots
passed. The independent no-prover task print matched every proof JSON root;
Astra separately accepted the batch as conditional modular body/refinement
evidence. Its report and task audit are in the run directory.

The batch covers seven `HeaderMap` bodies (`find`, `find_with_hash`, `get`,
`get2`, `get_mut`, `get_all`, and `contains_key`) and three sealed key adapters
(`HeaderName`, `&HeaderName`, and `&str`), plus their three trait-refinement
checks. The root audit separates 10 actual body roots, 3 refinement roots, and
30 imported support stubs. Those 30 roots are literal `true` stubs, not proofs
of their callees.

The contracts remain conditional and narrow. Each lookup body requires
`header_map_find_ready`; `find_with_hash` also requires a nonempty entries
vector. `find` and `find_with_hash` establish bounds and the slot/bucket/hash
and key-model relations only for `Some`; `None` carries no absence theorem.
The public getters have no functional result postcondition: they establish local
call, index, and type-invariant obligations. `get_mut` exports no map frame or
readiness-preservation postcondition. The sealed adapters establish only an
in-range index for `Some`.

The proof of `find` consumes an abstract `hash_elem_using` summary whose result
is an arbitrary `HashValue`; the selected run does not prove actual hash
semantics. Its summary has no false precondition, so this does not make the
selected caller proof vacuous. The `&str` adapter consumes the actual
`HdrName::from_bytes` parse/callback contract, which this map batch does not
reprove. Astra found no selected-context impossible precondition or false HTTP
axiom. The `RandomState` and `String::as_str` frontend warnings are from
unselected code, while `BuildHasher::build_hasher` and `Hasher::finish` mark the
unproved hash-helper body boundary.

The WIP `append_value` exact-append/frame contract and the map insertion,
mutation, growth, and actual-Map composition work remain unproved. This batch
does not establish that constructors or mutations create
`header_map_find_ready`, or prove slot coverage, Robin Hood probe order,
key/hash coherence, or `None iff absent`. The named leaf profile only changes
verification-only cfg branches; the native public API is unchanged.

The prior handoff text referred to an untracked partial batch directory that
was omitted from the recovery archive. Its proof JSON and COMAs were not
available to revalidate, so its partial-pass counts are not evidence. The fresh
run above is the source-linked record for this resumed work.

## Reproduce

From this directory, emit current Coma after a package-scoped clean, then prove
the exact generated targets:

```sh
source /workspace/proof-tools/activate.sh
export CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http
cargo clean -p http-header-map-api-proof
RUSTFLAGS='--cfg http_map_api_leaf' cargo creusot --simple-triggers=false -- \
  --features http_map_api_leaf --locked --offline
RUSTFLAGS='--cfg http_map_api_leaf' ../../scripts/run-proof.sh \
  cargo creusot --simple-triggers=false prove --no-cache \
  'verif/http_header_map_api_proof_rlib/header/map/allocation_len_bound.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_HeaderValue/new.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_Default_for_HeaderMap_T/default.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/len.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/keys_len.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/is_empty.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/clear.coma' \
  'verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/capacity.coma' \
  'verif/http_header_map_api_proof_rlib/vec_bounds/generic_vec_len.coma' \
  'verif/http_header_map_api_proof_rlib/vec_bounds/sum_nonzero_vec_lengths.coma' \
  'verif/http_header_map_api_proof_rlib/vec_bounds/u8_vec_len.coma' \
  'verif/http_header_map_api_proof_rlib/vec_bounds/unit_vec_len.coma' \
  -- --features http_map_api_leaf --locked --offline
```

Current `.coma` files, `proof.json` files, exact goal counts, and source/artifact
fingerprints are preserved in
[`evidence/run-2026-10-05-live-bound/manifest.json`](evidence/run-2026-10-05-live-bound/manifest.json).
