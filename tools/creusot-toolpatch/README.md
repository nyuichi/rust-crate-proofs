# Creusot proof tool compatibility patch

This directory packages a narrowly scoped Creusot backend fix and two Why3
overlay patches needed by the itoa proof work on the pinned Creusot 0.11.0-dev
toolchain. It is intended to be committed at
`tools/creusot-toolpatch/` in the repository. It does not change crate source,
verification contracts, or formatter trust annotations.

## Changes and trust boundary

- `patches/creusot-narrowcast-backend.patch` lowers an unsigned Rust narrowing
  cast to its exact mathematical remainder modulo `2^target_width`, then to the
  target unsigned type. It leaves other casts unchanged and emits proof
  assertions for the range facts. It adds only the `int.ComputerDivision`
  prelude dependency required by the modulo term.
- `patches/why3-bv64-narrowcast-driver.patch` activates Why3's standard total
  `int2bv 64` encoding for the BV64 integer conversion.
- `patches/why3-bv128-lsr-trigger.patch` adds a BV128-only lemma with an
  explicit trigger for the existing logical right-shift equation. The lemma
  is proved from the unchanged generic Why3 axiom by `clear_but`; no axiom,
  trusted contract, or range precondition is added or removed.

The backend patch covers executable unsigned casts. It does not make narrowing
casts in logical contracts total. The supplied high-half witness keeps the
contract in mathematical division form and verifies the actual Rust
`(n >> 64) as u64` body.

## Pins

Exact source, Why3, Why3find, Rust, and solver revisions and the environment
used for the captured proofs are in `records/versions.md`. The patches apply
to Creusot commit `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc` and Why3 commit
`2c0f2992af85f82f3eda0f158dcf10e62e0db875`.

## Build and verify

The scripts default to the temporary tool installations used during this work;
the absolute locations are defaults for reproducing these runs, not required
installation paths. Override `RUSTUP_HOME`, `RUSTUP_TOOLCHAIN`, `CARGO_HOME`,
`CREUSOT_DATA_HOME`, `WHY3_BASE_DATA`, `WHY3_BIN`, `CREUSOT_RUSTC`,
`WHY3CONFIG`, `WHY3DATA`, `CARGO_TARGET_DIR`, `LD_LIBRARY_PATH`,
`XDG_CACHE_HOME`, `XDG_CONFIG_HOME`, and `PATH` to use another local
installation. `run-verify-all.sh` also accepts `RUST_CRATE_PROOFS_ROOT`; the
build and patch-check scripts accept source/tool-data paths as positional
arguments, and the baseline/verify wrappers accept a repository root. Temporary
build and overlay directories use `/tmp`. No script reads, sets, or changes
`HOME`.

Rebuild the patched compiler, validate patch applicability, prove the derived
Why3 lemma, and run the narrow-cast / high-half / shift witnesses:

```sh
tools/creusot-toolpatch/scripts/check-patch-application.sh
tools/creusot-toolpatch/scripts/build-creusot-rustc.sh
tools/creusot-toolpatch/scripts/prove-derived-lsr-lemma.sh
tools/creusot-toolpatch/scripts/run-narrowcast-witness.sh
tools/creusot-toolpatch/scripts/run-high-half-witness.sh
tools/creusot-toolpatch/scripts/run-ediv-shift-witness.sh
```

Replay the archived Phase 2 source commit in both proof configurations:

```sh
tools/creusot-toolpatch/scripts/run-phase2-baseline.sh /workspace/rust-crate-proofs
```

Run the repository's official `itoa/1.0.18/verify-all.bash` against the
current worktree with the patched compiler and a freshly generated Why3
overlay:

```sh
tools/creusot-toolpatch/scripts/run-verify-all.sh /workspace/rust-crate-proofs
```

The wrapper selects the patched `CREUSOT_RUSTC`, sets the pinned nightly and
tool paths, sets `CARGO_NET_OFFLINE=true`, creates an isolated `WHY3DATA`
overlay from the pinned Why3 data tree, and removes only that generated
overlay at exit. Pass `WHY3DATA` explicitly to inspect or preserve a chosen
overlay. The official `verify-all.bash` itself is unchanged and runs its
default and `--all-features` proof configurations.

The recorded Phase 2 replay passed 70 proof units and 192 goals in each
configuration. The witness source and compact `.coma` / proof JSON results are
under `witnesses/` and `proofs/`. The separate Phase 4 arithmetic bodies are
not claimed as complete by this tooling package.
