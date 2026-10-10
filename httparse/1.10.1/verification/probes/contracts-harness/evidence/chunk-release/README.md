# Chunk release-profile proof snapshot

These generated artifacts were copied immediately after the uncached release
proof and before further Creusot translation overwrote the active `verif/`
outputs. The run compiled the harness in Cargo's `release` profile, so
`cfg!(debug_assertions)` selected the false branch.

Command, from `verification/probes/contracts-harness`:

```sh
source /workspace/proof-tools/activate.sh
/workspace/rust-crate-proofs/httparse/1.10.1/run-proof.bash \
  cargo creusot --simple-triggers=false prove --why3session --no-cache \
  verif/httparse_contracts_harness_rlib/chunk/step_chunk_size.coma \
  verif/httparse_contracts_harness_rlib/chunk/parse_chunk_size.coma \
  -- --offline --release
```

Z3 4.15.3 proved `step_chunk_size` (33 VCs) and `parse_chunk_size` (9 VCs).
The adjacent `.proof.json` files record the generated goal/prover results; the
`.coma` files record the exact translated release-profile bodies.
