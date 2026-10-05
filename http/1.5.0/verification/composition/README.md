# Request and Response source-module proof

> **Checkpoint note (2026-10-05):** This page contains earlier run counts and
> limitations. The current accepted frontier, including fixed-method shortcut
> and conditional `and_then` results, is in
> [the HTTP checkpoint](../CHECKPOINT_2026-10-05.md) and the exact manifests
> under `evidence/`. Treat the older counts below as historical.

This harness includes the production `src/request.rs` and `src/response.rs`
modules by path. Its `http_runtime` dependency is the actual HTTP 1.5.0 crate;
the imported head-component types are therefore real HTTP values, not local
stand-ins. The harness is intended to prove field accessors and body/parts
composition while leaving those dependency representations opaque.

Run the normal source check and Creusot target from this directory:

```sh
../../scripts/run-proof.sh cargo check --locked --offline
RUSTFLAGS="--cfg http_composition_leaf" ../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache -- --locked --offline
```

This is a leaf composition harness. A successful harness proof does not count
as an integrated proof of the complete HTTP crate. Builder operations that call
other HTTP methods, formatting/dynamic error APIs, and iterator/raw-pointer
behavior remain separate proof targets.

The latest Why3 run proved the exact body VCs for these 30 local source APIs:

- `Request<T>`: `from_parts`, `method`, `method_mut`, `uri`, `uri_mut`,
  `version`, `version_mut`, `headers`, `headers_mut`, `extensions`,
  `extensions_mut`, `body`, `body_mut`, `into_body`, `into_parts`, and `map`.
- `Response<T>`: `from_parts`, `status`, `status_mut`, `version`, `version_mut`,
  `headers`, `headers_mut`, `extensions`, `extensions_mut`, `body`, `body_mut`,
  `into_body`, `into_parts`, and `map`.

Each listed getter/mutator/parts body has one successful own-body VC. Each
`map` has two successful VCs, including the `FnOnce` contract relay. Those are
32 successful target VCs total. The harness run as a whole still exits
unsuccessfully: builder `body`/`and_then` VCs and the derived `Debug` refinement
VCs fail, so this is a set of accepted leaf bodies rather than an integrated
composition proof. `new`, `Default`, builder APIs, and formatting have no exact
functional body contract here and are not included in the count.

The proof command supplies the `http_composition_leaf` cfg to omit the derived
`Clone` impls on `Request`, `Response`, and their `Parts`, plus the
`Request<()>`/`Response<()>` builder entrypoint impls. Creusot currently ICEs
on the derived unit-field `Clone`, and Why3 rejects generated theories for the
builder shortcuts that inline external `Method` constants. The runtime build
retains these original APIs. The harness depends on the exact published 1.5.0
crate archive for opaque component types; the local Request/Response source
bodies for parts construction, field accessors, body accessors, and `map`
remain the targets under verification.

Before adding this leaf cfg, Why3 reported a parser error in
`request/impl_Request_unit/connect.coma:98`. The generated constant was
`const_CONNECT: t_Method = ... { ERROR_UNBOUND_f0 = Connect }`, sourced from
the archived Method representation when translating the convenience
`Request::connect` body. This is evidence about that builder path's external
constant translation, not evidence against the field-accessor bodies retained
in this target.
