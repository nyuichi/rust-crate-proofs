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
