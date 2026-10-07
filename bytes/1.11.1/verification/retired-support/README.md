# Private Creusot std support for bytes 1.11.1

`scripts/prepare-verified-std.py` prepares a bytes-only copy of the pinned
normalized `creusot-std 0.13.0` Cargo registry package. Set `BYTES_TOOL_ROOT`
or activate the bytes toolchain so the installer can derive it from
`CARGO_HOME`. The private package is placed at
`$BYTES_TOOL_ROOT/bytes-verified-std`; it never patches the installed source
checkout or registry package.

The installer checks the stock package and all nine affected files against
`manifest.json`, copies the package into a staging directory under the tool root,
and applies these normalized registry-relative patches in order:

1. `alloc-capacity.patch` — alloc-gated Vec, Box, String, conversion and slice
   contract exposure, including generic Vec capacity and fallible reservation
   contracts.
2. `address-model.patch` — the opaque numeric Vec base-address observer and
   its `as_ptr` / `as_mut_ptr` contracts.
3. `copy-slot.patch` — the generic `try_spawn_with_slot` standard-library
   wrapper.

It validates all nine final hashes and the candidate package-tree digest before
atomically publishing the private copy. Reruns succeed only when that copy still
matches the reviewed candidate; stale or conflicting contents are left intact
and reported as an error.

The installer creates or extends the ignored crate-local
`.cargo/config.toml` with a `[patch.crates-io]` override for `creusot-std`.
Existing unrelated TOML is preserved. A different `creusot-std` override, invalid
TOML, or a generated config path that is not Git-ignored is rejected without
replacing user content.

The capacity and base-address observers are opaque generic library contracts.
They expose no sequence-based allocation identity, pointer permission,
provenance identity, injectivity, or allocation-liveness rule. The capacity
contracts make no allocator-totality promise. `try_spawn_with_slot` is a generic
trusted std boundary with `F: Copy`; it does not add a bytes-specific ownership
or refcount theorem. Existing total-correctness assumptions on infallible std
externs remain inherited; this overlay adds no termination attributes.

This support layer is specific to the bytes 1.11.1 modified variant. It is not
installed into Cargo globally and has no effect on other crates.
