# HeaderMap capacity leaves

`src/header/map_capacity.rs` contains two private helpers used directly by the
production `HeaderMap` implementation. `usable_capacity` computes the primary
entry count for the load factor, and `checked_raw_capacity` computes
`requested + requested / 3` with checked overflow. The proof harness includes
that source file by path, so it verifies the same bodies the runtime calls.

From this directory, run:

```sh
../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove \
  --no-cache -- --locked --offline
```

On the x86_64 target with Creusot `0.11.0-dev` and
`creusot-std 0.11.0-dev`, the proof passed:

```text
Library verif.http_map_capacity_proof_rlib.map_capacity.checked_raw_capacity: ✔ (2)
Library verif.http_map_capacity_proof_rlib.map_capacity.usable_capacity: ✔ (1)
```

All three generated VCs passed with Z3 `4.15.3`; no local trusted contract was
added. This is an isolated helper proof. The `HeaderMap::to_raw_capacity`
wrapper that maps `None` to `MaxSizeReached`, all callers, and the hash table,
multimap, iterator, unsafe-pointer, and drop behavior remain unproved. The
crate's ordinary source check was also run from the crate root:

```sh
./scripts/run-proof.sh cargo check --locked --offline
```

It exited successfully after this extraction. Neither check is a successful
crate-level Creusot proof. The full-runtime translation blockers and their
current diagnostics are in [`TOOL_BLOCKERS.md`](../../TOOL_BLOCKERS.md).
