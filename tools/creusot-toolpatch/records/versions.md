# Pinned versions and run provenance

## Source and toolchain

| Component | Version / revision |
| --- | --- |
| Crate source for Phase 2 replay | `cf60e6f7f7d53abe66565774bc58c582b6932f53` (`prove decimal pair table correspondence`) |
| Creusot driver source | `437d3d8d00b8114d7a3b4f7b8738d594a395f5bc` (`0.11.0-dev`) |
| Rust toolchain | `nightly-2026-02-27`; `rustc 1.95.0-nightly (6a979b3e3 2026-02-26)`, `rustc-dev` and `llvm-tools` |
| Why3 | `1.8.2+git`, pinned commit `2c0f2992af85f82f3eda0f158dcf10e62e0db875` |
| Why3find | `v1.2.0+dev`, pinned commit `3a98fc320b9cbf2e71860da1c8dc188a966eee96` |
| Creusot library provenance in repo README | `7a48f5a5b1cb15a11c4e744568ca187331a30025`; the repository contains the vendored `0.11.0-dev` sources in-tree |
| Z3 | `4.15.3` |
| CVC5 | `1.3.1` |
| CVC4 | `1.8` |
| Alt-Ergo | `2.6.2` |

The patched `creusot-rustc` used for this record is
`/workspace/proof-tools/targets/creusot-cast-compdiv/debug/creusot-rustc`, built
by `scripts/build-creusot-rustc.sh`. The backend patch adds the explicit
`int.ComputerDivision` dependency for cast modulo terms.
The backend patch applies to the Creusot source revision above. Why3 overlay
patches apply to a Why3 data tree from the pinned Why3 revision.

## Environment used for the recorded checks

```sh
export RUSTUP_HOME=/tmp/rustup-home
export CARGO_HOME=/tmp/cargo-home
export CREUSOT_DATA_HOME=/tmp/creusot-data
export XDG_CACHE_HOME=/tmp/creusot-cache
export XDG_CONFIG_HOME=/tmp/why3-capture-config
export WHY3CONFIG=/tmp/why3-capture-config/creusot/why3.conf
export WHY3_BASE_DATA=/workspace/proof-tools/opamroot/creusot-v0.11/share/why3
# The scripts create a fresh patched WHY3DATA overlay from this pinned tree.
export CREUSOT_RUSTC=/workspace/proof-tools/targets/creusot-cast-compdiv/debug/creusot-rustc
export PATH=/tmp/creusot-data/bin:/tmp/cargo-home/bin:/workspace/proof-tools/opamroot/creusot-v0.11/bin:$PATH
export LD_LIBRARY_PATH=/tmp/rustup-home/toolchains/nightly-2026-02-27-x86_64-unknown-linux-gnu/lib:/tmp/local-ocaml/usr/lib/x86_64-linux-gnu
export CARGO_NET_OFFLINE=true
```

The baseline used a fresh pid-suffixed `CARGO_TARGET_DIR`. Its full output is
in `proofs/baseline/phase2-cf60e6f-compdiv-verify.log`. Both configurations
passed: 70 proof units and 192 goals each.

## Exact proof commands

The Why3 overlay is reconstructed with
`./scripts/prepare-why3-overlay.sh`; its source is the pinned Why3 data tree.
Set `WHY3_BASE_DATA` to another installation's `share/why3` directory if
needed. Derived library lemma:

```sh
why3 prove -C "$WHY3CONFIG" \
  -L "$WHY3DATA/stdlib" \
  -P 'Z3,4.15.3' -t 10 \
  -a 'clear_but to_uint_lsr' \
  "$WHY3DATA/stdlib/bv.mlw" \
  -T BV_Gen_Triggered -G to_uint_lsr_triggered
```

Narrow-cast and generic shift witnesses are run by the scripts in `scripts/`;
those first rerun the lemma above, use `--no-cache`, and have separate Cargo
target directories. The Phase 2 crate replay is `./verify-all.bash` from the
archived `itoa/1.0.18` directory. That script runs default and
`--all-features`, both with `CARGO_NET_OFFLINE=true`.
