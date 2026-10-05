# Message core constructor slice

This harness includes `src/message.rs` at its crate root so Creusot checks the
same Request/Response/Header definitions and the actual constructor bodies used
by `httparse`. Production uses `include!("message.rs")` at crate root to
preserve the public `type_name` paths.

The Request and Response constructors specify that every parsed scalar field
starts as `None` and that the returned mutable header slice has the same
sequence as the supplied slice. Header fields remain unrestricted beyond their
Rust types. `empty_header_value_caller` reads the actual `EMPTY_HEADER` constant
and specifies that its byte value is empty. Two representative callers consume
the constructor contracts.

The fresh isolated proof passed all 9 VCs with Z3 4.15.3: Request::new (1),
Response::new (1), the representative constructor callers (2 each), Header's
proof-only Clone body (1), and the `EMPTY_HEADER` value accessor (2, including
the constant initializer). The Why3 profile used one prover and a 1000 MiB
limit via the target-local proof wrapper.

The empty header name model remains open. An earlier probe with
`result.name@.len() == 0` did not prove because the standard-library `str::view`
was opaque. Replacing that condition with `result.name == ""` caused the
Creusot translator to ICE with `Unsupported literal` at the string literal.
This failed probe remains an explicit open item; no trusted string axiom was
added. Inspecting the current standard string model confirmed that `str::view`
is intentionally opaque. The runtime constant still sets `name: ""` exactly.

The original runtime `Header` `Debug`, `Eq`, and `PartialEq` implementations are
preserved in non-Creusot builds and remain outside this slice. Request and
Response `Debug`, `Eq`, and `PartialEq` derives are also excluded in proof
builds. Header's proof build derives `Copy`, `Clone`, and `DeepModel`; only the
Clone body is a proved target here. Its runtime formatter is retained under
`cfg(not(creusot))`. `EMPTY_HEADER.value` changes from `b""` to `&[]`, the same
empty byte slice; Creusot cannot translate the former literal directly.

This constructor checkpoint is partial: constructor and caller bodies plus the
byte value of `EMPTY_HEADER` are proved, while the `EMPTY_HEADER` name model and
derived/explicit trait implementations remain open. The crate-wide
full-verification gate stays open.

Run from this directory:

```sh
bash verify.sh translate
bash verify.sh prove
```

`verify.sh prove` runs a fresh `cargo creusot` translation only, checks that
each expected Coma target is nonempty, queries the selected `creusot` Why3
package, and then invokes `why3find prove --no-cache -s -j 1` through the
target-local shared proof runner. The runner validates the explicit Why3
profile before starting the solver. Before translation the script cleans only
this harness package in its explicit local `target/` directory and removes
prior Coma outputs recursively from this harness's verification target;
afterward it requires the Request and Response constructor bodies, Header clone
body, and all three callers in its mandatory manifest to be nonempty. Additional
Coma outputs are allowed.

Proof result: passed, 9 VCs, `bash verification/probes/message-core/verify.sh prove`
from the `httparse/1.10.1` directory.

The harness imports `ensures` unconditionally because the shared source uses
that attribute in both native and Creusot builds. Native offline checks passed
for default features and `--no-default-features`; both emitted only the existing
`creusot-std` unstable-auto-trait warning. This import-only correction leaves the
Creusot cfg, specifications, and function bodies unchanged. The proof was not
rerun, so the existing 9-VC artifacts were left untouched. The revised
split-phase runner has only passed shell syntax and diff checks; it has not yet
been run to create fresh proof evidence.
