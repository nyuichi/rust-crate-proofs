# HeaderMap capacity and index leaves

> **Checkpoint note (2026-10-05):** The helper results below remain useful,
> but the final paragraph predates the selected actual-source HeaderMap and
> conditional `IterMut::next_unsafe` runs. Current scope and open obligations
> are summarized in [the HTTP checkpoint](../CHECKPOINT_2026-10-05.md).

This target includes the actual private implementations from
`src/header/map_capacity.rs` and `src/header/map_index.rs`. The helpers prove
independently of the full `HeaderMap` table and iterator bodies.

Run the fresh proof from this directory with the shared HTTP proof runner:

```sh
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache -- --locked --offline
```

The 2026-10-05 replay proved 15 source targets across 30 solver leaves:

- `usable_capacity` and `checked_raw_capacity` (3 leaves total);
- `HashValue::clone`, `Pos::clone`, and `Pos::{new,is_some,is_none,resolve}`;
- `HashValue` equality body and refinement, plus its derived `Eq` helper body;
- the manual `HashValue` `DebugTuple` body and its refinement to the standard
  append-preservation contract;
- `desired_pos` exact bit-mask result and upper bound, using Creusot's existing
  `bitwise_proof` encoding;
- `probe_distance` exact cyclic-distance arithmetic (8 leaves).

`Pos::none` translates with zero solver obligations, so it is not counted as a
proved body. The `Debug` contract proves preservation of existing formatter
output as a prefix; it does not specify exact debug text. The earlier derived
Debug refinement left one leaf open; after replacing it with the runtime-
equivalent manual `DebugTuple` body and an append-preservation postcondition,
the fresh body proof (4 VCs) and refinement (2 VCs) both pass. Exact source
fingerprints, goal names, and hashed `.coma`/`proof.json` files are recorded in
[`evidence/manifest.json`](evidence/manifest.json). The rejected derived-body
attempt is retained under `evidence/attempts/`.

`probe_distance` assumes a power-of-two mask, `mask < MAX_SIZE`, an in-range
current slot, and an arithmetic bound. These are private caller invariants and
have not been proved at every `HeaderMap` call site. The helper evidence does
not verify insertion/removal, capacity growth, iterators, ownership of buckets,
or the integrated map implementation. The selected actual-source map API
leaves have a separate target in `verification/header-map-api`; its current
translation run stops in the generic insertion macro on `HeaderName`/`K`
`DeepModel` and `PartialEqModel` bounds, before reaching those leaf proofs.

The production `RawLinks` permission experiment was reverted; it was not
verified and is not counted as proof evidence. A crate-root normal check passed
after that rollback, but a later shared URI edit currently makes it stop on the
test-only `URI_CHARS` table being unused outside tests. A whole-source Creusot
translation also reports in-flight `Method`, `Status`, and URI model
diagnostics; those failures are not counted as map helper counterexamples.
