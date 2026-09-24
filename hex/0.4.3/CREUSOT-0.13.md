# Creusot 0.13.0 core codec verification

Scope: hex 0.4.3, `--no-default-features`, `encode_to_slice` and
`decode_to_slice`. Codec implementations, preconditions, postconditions and
loop invariants are unchanged. The former local FromIteratorSpec trait and
trusted formatter model are now defined in this crate.

## Reproduce without Unix sockets

Install Creusot v0.13.0 (318615be), its Rust nightly-2026-06-22 toolchain,
Why3 at 54c92f96bb0711d6e991c18f10bfbc08d90d028b, and Z3 4.15.3.
Use the unmodified upstream Why3 source and the Creusot 0.13.0 prelude.
Put cargo-creusot, why3, and z3 on PATH, then run:

```sh
python3 verify-core.py
```

`WHY3`, `Z3`, `CREUSOT_PRELUDE`, and `WHY3_Z3_DRIVER` may select explicit
executable, prelude-directory and driver paths. `CREUSOT_DATA_HOME` has the same
meaning as in Creusot. The default driver is `z3_4_12`.

The script translates a fresh no-default-features build, exports every generated
Coma file through Why3's standard `split_vc` transformation and Z3 driver, then
runs Z3 directly on every exported SMT problem. Only exit code zero and an
`unsat` response count as success. No socket server, transport modification,
proof cache, or why3find session is used.

Observed result: **167/167 obligations proved, from 20 Coma files**.
`target/core-proof/results.json` records each problem hash and solver result;
the adjacent files contain exported SMT problems and export logs.

The generic AsRef bridge and formatting/generic adapters retain their explicit
trusted boundaries. The report concerns the two slice codec APIs only.
Alloc, serde and other feature combinations are not verified under 0.13.0;
`verify-all.bash` remains the older multi-feature workflow and is not the command
for this report.

`cargo-proofs 0.2.0` currently expects a native why3find `proof.json` session.
This direct-SMT workflow does not produce or impersonate one, so recording it
requires support for this evidence format or a separate run uploader.
