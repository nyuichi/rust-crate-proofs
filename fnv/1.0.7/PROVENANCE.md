# fnv 1.0.7 provenance

**Verification status: complete.**

This source tree is copied from the crate published on crates.io as `fnv`
version `1.0.7`. The published archive has SHA-256 checksum
`3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1`.

The archive records the immutable upstream revision
`4b4784ebfd3332dc61f0640764d6f1140e03a9ab`, which is the commit referenced by
the annotated upstream tag `v1.0.7`:

https://github.com/servo/rust-fnv/tree/4b4784ebfd3332dc61f0640764d6f1140e03a9ab

The public API, runtime behavior, and upstream MIT and Apache-2.0 license files
are preserved. The Rust source includes Creusot specifications, a recursive
FNV-1a fold, its append lemma, and proof annotations added by this repository.

The proof checks arithmetic and indexing safety and establishes that
`FnvHasher::write` updates the hasher's old abstract state to the specified
64-bit FNV-1a fold over the complete input slice, including bitvector XOR and
wrapping multiplication semantics. The fold append lemma connects each loop
iteration to the complete byte sequence. Contracts also specify `Default`,
`with_key`, and `finish`.

The upstream tuple field remains private. `FnvHasher` instead exposes an opaque
logical `u64` view and has an explicit type invariant: every 64-bit bit pattern
is a valid FNV-1a state. This preserves representation hiding while allowing
clients to use every operational contract. The standard-library hash map and
hash set names are type aliases rather than new nominal types and contain no
fnv implementation of their own.

## Completion status

This is a complete functional proof of every crate-owned executable public API.
There are no `#[trusted]` declarations. `FnvBuildHasher`, `FnvHashMap`, and
`FnvHashSet` are type aliases to standard-library generic types and introduce no
additional implementation body in this crate.

`./verify-all.bash` checks both supported feature configurations:

- `--no-default-features` (`no_std`);
- `--all-features` (default `std`).

Both configurations prove 8 files. The upstream test vector passes in both;
the std configuration additionally passes 2 documentation tests.

Generated Why3 and Cargo build artifacts are intentionally not tracked. Run
`./verify-all.bash` to regenerate and check them.

## Creusot 0.13.0 migration (2026-09-25)

The crate now pins crates.io `creusot-std =0.13.0` and Rust
`nightly-2026-06-22` in a crate-local toolchain file. No Rust source or contracts
were changed for this migration. `verify-all.bash` uses the 0.13 CLI syntax
and disables the proof cache. Native Why3/why3find sessions prove 8 files
per checked feature configuration.

Environment: macOS aarch64, Why3 `54c92f96`, why3find `eab37557`,
Z3 4.16.0 and CVC5 1.3.1. For publication, run from this crate directory:

```sh
cargo proofs run -- cargo creusot --no-cache -- --no-default-features
cargo proofs publish --dry-run
```

The source and lockfile must first be committed and pushed. Recording requires
the cargo-proofs Creusot 0.13 version-command and Coma method-span adapter fix.
Published claims are `panic_contract` only; see `proofs.toml` for the exact
assumptions and limits. Functional specifications do not become separate
functional-correctness claims.

The existing verification fork changes the upstream iterator loop into an
index-based while loop; this is not a byte-for-byte upstream implementation.
The FNV XOR/wrapping-multiply operations are preserved.
